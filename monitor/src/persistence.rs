//! Event batching and persistence for Supabase edge function.
//!
//! This module handles batching and sending events to a Supabase edge function
//! for historic storage and activity heatmap visualization.
//!
//! # Design
//!
//! - **Best-effort**: Persistence does not block real-time event flow. If the
//!   edge function is unavailable, events are buffered and retried later.
//!
//! - **Batching**: Events are collected in a buffer and sent periodically or
//!   when the buffer reaches [`MAX_BATCH_SIZE`] events (whichever comes first).
//!
//! - **Retry behavior**: Failed submissions use exponential backoff with a
//!   configurable retry limit. After exceeding the limit, the batch is dropped
//!   and the failure count is reset to prevent unbounded memory growth.
//!
//! # Example
//!
//! ```no_run
//! use vibetea_monitor::persistence::{EventBatcher, PersistenceError};
//! use vibetea_monitor::config::PersistenceConfig;
//! use vibetea_monitor::crypto::Crypto;
//! use vibetea_monitor::types::{Event, EventType, EventPayload, SessionAction};
//! use uuid::Uuid;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), PersistenceError> {
//!     let config = PersistenceConfig {
//!         supabase_url: "https://xyz.supabase.co/functions/v1".to_string(),
//!         batch_interval_secs: 60,
//!         retry_limit: 3,
//!     };
//!     let crypto = Crypto::generate();
//!     let mut batcher = EventBatcher::new(config, crypto);
//!
//!     let event = Event::new(
//!         "my-monitor".to_string(),
//!         EventType::Session,
//!         EventPayload::Session {
//!             session_id: Uuid::new_v4(),
//!             action: SessionAction::Started,
//!             project: "my-project".to_string(),
//!         },
//!     );
//!
//!     batcher.queue(event);
//!     let sent = batcher.flush().await?;
//!     println!("Sent {} events", sent);
//!     Ok(())
//! }
//! ```

use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{Client, StatusCode};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::PersistenceConfig;
use crate::crypto::Crypto;
use crate::types::Event;

/// Maximum number of events per batch (per FR-002).
const MAX_BATCH_SIZE: usize = 1000;

/// HTTP request timeout in seconds.
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Initial retry delay in milliseconds (per FR-015).
const INITIAL_RETRY_DELAY_MS: u64 = 1000;

/// Multiplier for exponential backoff (per FR-015).
const RETRY_BACKOFF_MULTIPLIER: u64 = 2;

/// Errors that can occur during event persistence.
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization failed.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Authentication failed (401 response).
    #[error("authentication failed: invalid signature")]
    AuthFailed,

    /// Server returned an error status.
    #[error("server error: {status} - {message}")]
    ServerError {
        /// HTTP status code.
        status: u16,
        /// Error message from server.
        message: String,
    },

    /// Maximum retry attempts exceeded.
    #[error("max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded {
        /// Number of attempts made.
        attempts: u8,
    },

    /// Invalid header value.
    #[error("invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
}

/// Batches events and sends them to the Supabase ingest edge function.
///
/// The batcher collects events in a buffer and sends them either:
/// - When [`flush`](Self::flush) is called explicitly
/// - When the buffer reaches [`MAX_BATCH_SIZE`] events
///
/// # Thread Safety
///
/// This struct is not thread-safe. For concurrent access, wrap it in
/// appropriate synchronization primitives (e.g., `Mutex`).
pub struct EventBatcher {
    /// Persistence configuration.
    config: PersistenceConfig,

    /// Cryptographic context for signing requests.
    crypto: Crypto,

    /// Buffered events awaiting transmission.
    buffer: Vec<Event>,

    /// HTTP client with connection pooling.
    client: Client,

    /// Count of consecutive failed flush attempts.
    consecutive_failures: u8,
}

impl EventBatcher {
    /// Creates a new event batcher with the given configuration and crypto context.
    ///
    /// # Arguments
    ///
    /// * `config` - Persistence configuration (URL, batch interval, retry limit)
    /// * `crypto` - Cryptographic context for signing requests
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::persistence::EventBatcher;
    /// use vibetea_monitor::config::PersistenceConfig;
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let config = PersistenceConfig {
    ///     supabase_url: "https://example.supabase.co/functions/v1".to_string(),
    ///     batch_interval_secs: 60,
    ///     retry_limit: 3,
    /// };
    /// let crypto = Crypto::generate();
    /// let batcher = EventBatcher::new(config, crypto);
    ///
    /// assert!(batcher.is_empty());
    /// ```
    #[must_use]
    pub fn new(config: PersistenceConfig, crypto: Crypto) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .pool_max_idle_per_host(5)
            .build()
            .expect("failed to create HTTP client");

        Self {
            config,
            crypto,
            buffer: Vec::with_capacity(MAX_BATCH_SIZE),
            client,
            consecutive_failures: 0,
        }
    }

    /// Adds an event to the buffer for later transmission.
    ///
    /// If the buffer is at capacity ([`MAX_BATCH_SIZE`]), the oldest event is
    /// evicted to make room for the new event (FIFO eviction). This should
    /// rarely occur since the method returns `true` when the buffer becomes
    /// full, allowing callers to trigger a flush before overflow.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to queue for persistence
    ///
    /// # Returns
    ///
    /// Returns `true` if the buffer has reached [`MAX_BATCH_SIZE`] events
    /// after adding this event, indicating that a flush should be triggered.
    /// Returns `false` if the buffer has room for more events.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::persistence::EventBatcher;
    /// use vibetea_monitor::config::PersistenceConfig;
    /// use vibetea_monitor::crypto::Crypto;
    /// use vibetea_monitor::types::{Event, EventType, EventPayload, SessionAction};
    /// use uuid::Uuid;
    ///
    /// let config = PersistenceConfig {
    ///     supabase_url: "https://example.supabase.co/functions/v1".to_string(),
    ///     batch_interval_secs: 60,
    ///     retry_limit: 3,
    /// };
    /// let mut batcher = EventBatcher::new(config, Crypto::generate());
    ///
    /// let event = Event::new(
    ///     "monitor".to_string(),
    ///     EventType::Session,
    ///     EventPayload::Session {
    ///         session_id: Uuid::new_v4(),
    ///         action: SessionAction::Started,
    ///         project: "my-project".to_string(),
    ///     },
    /// );
    ///
    /// let needs_flush = batcher.queue(event);
    /// assert!(!needs_flush); // Buffer not full with just one event
    /// assert_eq!(batcher.buffer_len(), 1);
    /// ```
    pub fn queue(&mut self, event: Event) -> bool {
        // If buffer is at capacity, evict oldest to make room (shouldn't happen often)
        if self.buffer.len() >= MAX_BATCH_SIZE {
            warn!("Persistence buffer overflow, dropping oldest event");
            self.buffer.remove(0);
        }

        self.buffer.push(event);
        self.buffer.len() >= MAX_BATCH_SIZE
    }

    /// Returns `true` if the buffer has reached [`MAX_BATCH_SIZE`] events.
    ///
    /// This can be used to check whether a flush should be triggered without
    /// adding a new event.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::persistence::EventBatcher;
    /// use vibetea_monitor::config::PersistenceConfig;
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let config = PersistenceConfig {
    ///     supabase_url: "https://example.supabase.co/functions/v1".to_string(),
    ///     batch_interval_secs: 60,
    ///     retry_limit: 3,
    /// };
    /// let batcher = EventBatcher::new(config, Crypto::generate());
    ///
    /// assert!(!batcher.needs_flush()); // Empty buffer doesn't need flush
    /// ```
    #[must_use]
    pub fn needs_flush(&self) -> bool {
        self.buffer.len() >= MAX_BATCH_SIZE
    }

    /// Flushes all buffered events to the Supabase edge function.
    ///
    /// Serializes the buffered events to JSON, signs the payload using Ed25519,
    /// and sends it to the ingest endpoint with authentication headers.
    ///
    /// # Retry Behavior (FR-015)
    ///
    /// On transient failures (server errors, network issues), this method retries
    /// with exponential backoff (1s, 2s, 4s delays). After exceeding the configured
    /// retry limit, the batch is dropped and a warning is logged to prevent
    /// unbounded memory growth.
    ///
    /// Authentication errors (401) are NOT retried, as they indicate a
    /// configuration issue that requires human intervention.
    ///
    /// # Returns
    ///
    /// The number of events successfully sent. Returns `Ok(0)` if the buffer
    /// is empty (no request is made).
    ///
    /// # Errors
    ///
    /// Returns `PersistenceError` if:
    /// - The HTTP request fails
    /// - JSON serialization fails
    /// - The server returns an authentication error (401) - not retried
    /// - The server returns an error status - retried until limit
    /// - Maximum retry attempts are exceeded - batch is dropped
    pub async fn flush(&mut self) -> Result<usize, PersistenceError> {
        self.flush_with_delay(INITIAL_RETRY_DELAY_MS).await
    }

    /// Internal flush implementation with configurable initial delay for testing.
    async fn flush_with_delay(&mut self, initial_delay_ms: u64) -> Result<usize, PersistenceError> {
        if self.buffer.is_empty() {
            debug!("flush called with empty buffer, skipping");
            return Ok(0);
        }

        let events: Vec<Event> = self.buffer.clone();
        let event_count = events.len();
        let mut delay_ms = initial_delay_ms;

        debug!(event_count = event_count, "flushing event buffer");

        loop {
            match self.send_batch(&events).await {
                Ok(count) => {
                    self.buffer.clear();
                    self.consecutive_failures = 0;
                    info!(event_count = count, "successfully flushed events");
                    return Ok(count);
                }
                Err(PersistenceError::AuthFailed) => {
                    // Auth failures are not retryable - they require config changes
                    self.consecutive_failures += 1;
                    error!(
                        consecutive_failures = self.consecutive_failures,
                        "authentication failed, not retrying"
                    );
                    return Err(PersistenceError::AuthFailed);
                }
                Err(e) => {
                    self.consecutive_failures += 1;

                    if self.consecutive_failures >= self.config.retry_limit {
                        // Drop the batch and reset to prevent unbounded memory growth
                        warn!(
                            attempts = self.consecutive_failures,
                            events = event_count,
                            "max retries exceeded, dropping batch"
                        );
                        let attempts = self.consecutive_failures;
                        self.buffer.clear();
                        self.consecutive_failures = 0;
                        return Err(PersistenceError::MaxRetriesExceeded { attempts });
                    }

                    debug!(
                        attempt = self.consecutive_failures,
                        delay_ms = delay_ms,
                        error = %e,
                        "batch send failed, will retry"
                    );

                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = delay_ms.saturating_mul(RETRY_BACKOFF_MULTIPLIER);
                }
            }
        }
    }

    /// Sends a batch of events to the Supabase ingest endpoint.
    ///
    /// Signs the JSON payload with Ed25519 and includes authentication headers.
    ///
    /// # Arguments
    ///
    /// * `events` - Slice of events to send
    ///
    /// # Returns
    ///
    /// The number of events sent on success.
    ///
    /// # Errors
    ///
    /// Returns `PersistenceError` if the request fails or server returns an error.
    async fn send_batch(&self, events: &[Event]) -> Result<usize, PersistenceError> {
        let count = events.len();

        // Serialize events to JSON
        let body = serde_json::to_string(events)?;

        // Sign the JSON body
        let signature = self.crypto.sign(body.as_bytes());

        // Get source_id from first event
        let source_id = &events[0].source;

        // Build headers
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("X-Source-ID", HeaderValue::from_str(source_id)?);
        headers.insert("X-Signature", HeaderValue::from_str(&signature)?);

        // Build URL for ingest endpoint
        let url = format!("{}/ingest", self.config.supabase_url);

        debug!(
            url = %url,
            source_id = %source_id,
            event_count = count,
            "sending batch to ingest endpoint"
        );

        // Send request
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .body(body)
            .send()
            .await?;

        // Handle response
        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => Ok(count),
            StatusCode::UNAUTHORIZED => Err(PersistenceError::AuthFailed),
            status => {
                let message = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "unknown error".to_string());
                Err(PersistenceError::ServerError {
                    status: status.as_u16(),
                    message,
                })
            }
        }
    }

    /// Returns the number of events currently buffered.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::persistence::EventBatcher;
    /// use vibetea_monitor::config::PersistenceConfig;
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let config = PersistenceConfig {
    ///     supabase_url: "https://example.supabase.co/functions/v1".to_string(),
    ///     batch_interval_secs: 60,
    ///     retry_limit: 3,
    /// };
    /// let batcher = EventBatcher::new(config, Crypto::generate());
    /// assert_eq!(batcher.buffer_len(), 0);
    /// ```
    #[must_use]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the buffer contains no events.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::persistence::EventBatcher;
    /// use vibetea_monitor::config::PersistenceConfig;
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let config = PersistenceConfig {
    ///     supabase_url: "https://example.supabase.co/functions/v1".to_string(),
    ///     batch_interval_secs: 60,
    ///     retry_limit: 3,
    /// };
    /// let batcher = EventBatcher::new(config, Crypto::generate());
    /// assert!(batcher.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns a reference to the persistence configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::persistence::EventBatcher;
    /// use vibetea_monitor::config::PersistenceConfig;
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let config = PersistenceConfig {
    ///     supabase_url: "https://example.supabase.co/functions/v1".to_string(),
    ///     batch_interval_secs: 60,
    ///     retry_limit: 3,
    /// };
    /// let batcher = EventBatcher::new(config, Crypto::generate());
    /// assert_eq!(batcher.config().batch_interval_secs, 60);
    /// ```
    #[must_use]
    pub fn config(&self) -> &PersistenceConfig {
        &self.config
    }
}

/// Manages event persistence with timer-based and capacity-based flushing.
///
/// The `PersistenceManager` wraps an [`EventBatcher`] and provides an async runtime
/// that flushes events to the Supabase edge function when either:
/// - The configurable interval elapses (default: 60 seconds)
/// - The batch reaches [`MAX_BATCH_SIZE`] events (1000)
///
/// Whichever condition occurs first triggers a flush.
///
/// # Example
///
/// ```no_run
/// use vibetea_monitor::persistence::PersistenceManager;
/// use vibetea_monitor::config::PersistenceConfig;
/// use vibetea_monitor::crypto::Crypto;
/// use vibetea_monitor::types::{Event, EventType, EventPayload, SessionAction};
/// use uuid::Uuid;
///
/// #[tokio::main]
/// async fn main() {
///     let config = PersistenceConfig {
///         supabase_url: "https://xyz.supabase.co/functions/v1".to_string(),
///         batch_interval_secs: 60,
///         retry_limit: 3,
///     };
///     let crypto = Crypto::generate();
///     let (manager, sender) = PersistenceManager::new(config, crypto);
///
///     // Spawn the manager to run in the background
///     let handle = tokio::spawn(manager.run());
///
///     // Send events through the channel
///     let event = Event::new(
///         "my-monitor".to_string(),
///         EventType::Session,
///         EventPayload::Session {
///             session_id: Uuid::new_v4(),
///             action: SessionAction::Started,
///             project: "my-project".to_string(),
///         },
///     );
///     sender.send(event).await.unwrap();
///
///     // Drop sender to trigger shutdown
///     drop(sender);
///     handle.await.unwrap();
/// }
/// ```
pub struct PersistenceManager {
    /// The event batcher for buffering and sending events.
    batcher: EventBatcher,

    /// Receiver for incoming events.
    receiver: mpsc::Receiver<Event>,
}

impl PersistenceManager {
    /// Creates a new persistence manager and returns the event sender.
    ///
    /// # Arguments
    ///
    /// * `config` - Persistence configuration (URL, batch interval, retry limit)
    /// * `crypto` - Cryptographic context for signing requests
    ///
    /// # Returns
    ///
    /// A tuple containing the manager and an `mpsc::Sender<Event>` for sending events.
    /// The channel capacity is set to `MAX_BATCH_SIZE * 2` to provide backpressure.
    #[must_use]
    pub fn new(config: PersistenceConfig, crypto: Crypto) -> (Self, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel(MAX_BATCH_SIZE * 2);
        let batcher = EventBatcher::new(config, crypto);
        (
            Self {
                batcher,
                receiver: rx,
            },
            tx,
        )
    }

    /// Runs the persistence manager, processing events and flushing periodically.
    ///
    /// This method runs until the sender is dropped (signaling shutdown). On shutdown,
    /// any remaining buffered events are flushed before returning.
    ///
    /// # Flush Triggers
    ///
    /// Events are flushed when either:
    /// - The configured batch interval elapses (and buffer is not empty)
    /// - The buffer reaches [`MAX_BATCH_SIZE`] events
    ///
    /// Whichever condition occurs first triggers the flush.
    ///
    /// # Error Handling
    ///
    /// Flush errors are logged but do not cause the manager to exit. The manager
    /// continues running and will retry failed batches according to the configured
    /// retry policy.
    pub async fn run(mut self) {
        let interval_secs = self.batcher.config().batch_interval_secs;
        let interval = Duration::from_secs(interval_secs);
        let mut timer = tokio::time::interval(interval);

        // Skip missed ticks to avoid bursts of flushes after delays
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // First tick fires immediately, skip it
        timer.tick().await;

        info!(
            interval_secs = interval_secs,
            max_batch_size = MAX_BATCH_SIZE,
            "persistence manager started"
        );

        loop {
            tokio::select! {
                // Receive events from the channel
                maybe_event = self.receiver.recv() => {
                    match maybe_event {
                        Some(event) => {
                            debug!(event_id = %event.id, "received event for persistence");
                            let needs_flush = self.batcher.queue(event);

                            if needs_flush {
                                debug!(
                                    buffer_size = self.batcher.buffer_len(),
                                    "buffer full, triggering immediate flush"
                                );
                                if let Err(e) = self.batcher.flush().await {
                                    warn!(error = %e, "persistence flush failed (buffer full)");
                                }
                            }
                        }
                        None => {
                            // Channel closed - sender dropped, initiate shutdown
                            info!("event channel closed, shutting down persistence manager");
                            break;
                        }
                    }
                }

                // Periodic timer tick
                _ = timer.tick() => {
                    if !self.batcher.is_empty() {
                        debug!(
                            buffer_size = self.batcher.buffer_len(),
                            "timer tick, flushing events"
                        );
                        if let Err(e) = self.batcher.flush().await {
                            warn!(error = %e, "periodic persistence flush failed");
                        }
                    }
                }
            }
        }

        // Flush remaining events on shutdown
        if !self.batcher.is_empty() {
            info!(
                remaining_events = self.batcher.buffer_len(),
                "flushing remaining events before shutdown"
            );
            if let Err(e) = self.batcher.flush().await {
                warn!(error = %e, "final persistence flush failed");
            }
        }

        info!("persistence manager stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventPayload, EventType, SessionAction};
    use uuid::Uuid;

    fn create_test_config() -> PersistenceConfig {
        PersistenceConfig {
            supabase_url: "https://test.supabase.co/functions/v1".to_string(),
            batch_interval_secs: 60,
            retry_limit: 3,
        }
    }

    fn create_test_crypto() -> Crypto {
        Crypto::generate()
    }

    fn create_test_event() -> Event {
        Event::new(
            "test-monitor".to_string(),
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        )
    }

    fn create_test_event_with_id(id: &str) -> Event {
        Event {
            id: id.to_string(),
            source: "test-monitor".to_string(),
            timestamp: chrono::Utc::now(),
            event_type: EventType::Session,
            payload: EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        }
    }

    #[test]
    fn test_new_creates_empty_batcher() {
        let batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        assert!(batcher.is_empty());
        assert_eq!(batcher.buffer_len(), 0);
        assert_eq!(batcher.consecutive_failures, 0);
    }

    #[test]
    fn test_buffer_len_returns_zero_for_new_batcher() {
        let batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        assert_eq!(batcher.buffer_len(), 0);
    }

    #[test]
    fn test_is_empty_returns_true_for_new_batcher() {
        let batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        assert!(batcher.is_empty());
    }

    #[tokio::test]
    async fn test_flush_returns_zero_when_empty() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        // Flush with empty buffer should return 0 without making a request
        let result = batcher.flush().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_max_batch_size_constant() {
        // Verify the constant is set per FR-002
        assert_eq!(MAX_BATCH_SIZE, 1000);
    }

    #[test]
    fn test_request_timeout_constant() {
        assert_eq!(REQUEST_TIMEOUT_SECS, 30);
    }

    #[test]
    fn test_persistence_error_display() {
        let http_err = PersistenceError::AuthFailed;
        assert_eq!(
            http_err.to_string(),
            "authentication failed: invalid signature"
        );

        let server_err = PersistenceError::ServerError {
            status: 500,
            message: "Internal error".to_string(),
        };
        assert_eq!(server_err.to_string(), "server error: 500 - Internal error");

        let retry_err = PersistenceError::MaxRetriesExceeded { attempts: 3 };
        assert_eq!(
            retry_err.to_string(),
            "max retries exceeded after 3 attempts"
        );
    }

    #[test]
    fn test_queue_adds_event_to_buffer() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());
        let event = create_test_event();
        let event_id = event.id.clone();

        batcher.queue(event);

        assert_eq!(batcher.buffer_len(), 1);
        assert!(!batcher.is_empty());
        assert_eq!(batcher.buffer[0].id, event_id);
    }

    #[test]
    fn test_queue_returns_false_when_buffer_not_full() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        // Queue a single event
        let needs_flush = batcher.queue(create_test_event());
        assert!(!needs_flush);

        // Queue more events but stay under MAX_BATCH_SIZE
        for _ in 0..10 {
            let needs_flush = batcher.queue(create_test_event());
            assert!(!needs_flush);
        }
        assert_eq!(batcher.buffer_len(), 11);
    }

    #[test]
    fn test_queue_returns_true_when_buffer_full() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        // Fill buffer to MAX_BATCH_SIZE - 1
        for _ in 0..(MAX_BATCH_SIZE - 1) {
            let needs_flush = batcher.queue(create_test_event());
            assert!(!needs_flush);
        }
        assert_eq!(batcher.buffer_len(), MAX_BATCH_SIZE - 1);

        // The 1000th event should trigger full buffer
        let needs_flush = batcher.queue(create_test_event());
        assert!(needs_flush);
        assert_eq!(batcher.buffer_len(), MAX_BATCH_SIZE);
    }

    #[test]
    fn test_queue_evicts_oldest_when_over_capacity() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        // Add first event with known ID
        let first_event = create_test_event_with_id("evt_first_event_id_12345");
        batcher.queue(first_event);

        // Add second event with known ID
        let second_event = create_test_event_with_id("evt_second_event_id_1234");
        batcher.queue(second_event);

        // Fill remaining slots to capacity
        for i in 2..MAX_BATCH_SIZE {
            batcher.queue(create_test_event_with_id(&format!(
                "evt_event_number_{:08}",
                i
            )));
        }
        assert_eq!(batcher.buffer_len(), MAX_BATCH_SIZE);

        // Verify first event is still there
        assert_eq!(batcher.buffer[0].id, "evt_first_event_id_12345");
        // Verify second event is in position 1
        assert_eq!(batcher.buffer[1].id, "evt_second_event_id_1234");

        // Queue one more - should evict the first event
        let new_event = create_test_event_with_id("evt_new_overflow_event_");
        let needs_flush = batcher.queue(new_event);

        // Buffer should still be at max capacity
        assert_eq!(batcher.buffer_len(), MAX_BATCH_SIZE);
        assert!(needs_flush);

        // First event should now be the second event we added
        assert_eq!(batcher.buffer[0].id, "evt_second_event_id_1234");

        // Last event should be our new overflow event
        assert_eq!(
            batcher.buffer[MAX_BATCH_SIZE - 1].id,
            "evt_new_overflow_event_"
        );
    }

    #[test]
    fn test_needs_flush_returns_false_when_empty() {
        let batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        assert!(!batcher.needs_flush());
    }

    #[test]
    fn test_needs_flush_returns_false_when_partially_full() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        // Add some events but not enough to fill
        for _ in 0..500 {
            batcher.queue(create_test_event());
        }

        assert!(!batcher.needs_flush());
    }

    #[test]
    fn test_needs_flush_returns_true_when_full() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        // Fill buffer to capacity
        for _ in 0..MAX_BATCH_SIZE {
            batcher.queue(create_test_event());
        }

        assert!(batcher.needs_flush());
    }

    #[test]
    fn test_queue_multiple_events_increments_buffer() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

        batcher.queue(create_test_event());
        assert_eq!(batcher.buffer_len(), 1);

        batcher.queue(create_test_event());
        assert_eq!(batcher.buffer_len(), 2);

        batcher.queue(create_test_event());
        assert_eq!(batcher.buffer_len(), 3);
    }

    #[test]
    fn test_invalid_header_error_display() {
        // Create an invalid header value to test the error type
        let result = HeaderValue::from_str("invalid\nheader");
        assert!(result.is_err());

        // Verify the error can be converted to PersistenceError
        let persistence_err: PersistenceError = result.unwrap_err().into();
        assert!(persistence_err.to_string().contains("invalid header value"));
    }
}

/// Integration tests using wiremock for HTTP mocking.
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::types::{EventPayload, EventType, SessionAction};
    use uuid::Uuid;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_crypto() -> Crypto {
        Crypto::generate()
    }

    fn create_test_event_with_source(source: &str) -> Event {
        Event::new(
            source.to_string(),
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        )
    }

    fn create_config_with_url(url: &str) -> PersistenceConfig {
        PersistenceConfig {
            supabase_url: url.to_string(),
            batch_interval_secs: 60,
            retry_limit: 3,
        }
    }

    #[tokio::test]
    async fn test_flush_sends_request_with_correct_headers() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Set up mock to verify headers
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .and(header("Content-Type", "application/json"))
            .and(header("X-Source-ID", "test-monitor"))
            .and(header_exists("X-Signature"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Create batcher with mock server URL
        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        // Queue an event and flush
        batcher.queue(create_test_event_with_source("test-monitor"));
        let result = batcher.flush().await;

        // Verify success
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_flush_clears_buffer_on_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        // Queue multiple events
        batcher.queue(create_test_event_with_source("monitor"));
        batcher.queue(create_test_event_with_source("monitor"));
        batcher.queue(create_test_event_with_source("monitor"));
        assert_eq!(batcher.buffer_len(), 3);

        // Flush should clear buffer
        let result = batcher.flush().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert!(batcher.is_empty());
        assert_eq!(batcher.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_flush_returns_error_on_401() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        let result = batcher.flush().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PersistenceError::AuthFailed));

        // Buffer should NOT be cleared on error
        assert_eq!(batcher.buffer_len(), 1);
        assert_eq!(batcher.consecutive_failures, 1);
    }

    #[tokio::test]
    async fn test_flush_returns_error_on_500() {
        // With retry logic, a persistent 500 error will exhaust retries.
        // Use retry_limit=1 to test the basic error handling path.
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Use retry_limit=1 so it fails immediately without retrying
        let config = create_config_with_retry_limit(&mock_server.uri(), 1);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // Use fast delay for testing
        let result = batcher.flush_with_delay(1).await;
        assert!(result.is_err());

        // With retry logic, this becomes MaxRetriesExceeded after retry_limit=1
        match result.unwrap_err() {
            PersistenceError::MaxRetriesExceeded { attempts } => {
                assert_eq!(attempts, 1);
            }
            other => panic!("Expected MaxRetriesExceeded, got {:?}", other),
        }

        // Buffer should be cleared after max retries (batch dropped per FR-015)
        assert!(batcher.is_empty());
        assert_eq!(batcher.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_flush_does_nothing_when_empty() {
        let mock_server = MockServer::start().await;

        // Should NOT receive any requests
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        // Flush empty buffer
        let result = batcher.flush().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_flush_accepts_201_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));
        let result = batcher.flush().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert!(batcher.is_empty());
    }

    #[tokio::test]
    async fn test_flush_accepts_202_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(202))
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));
        let result = batcher.flush().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert!(batcher.is_empty());
    }

    #[tokio::test]
    async fn test_flush_resets_consecutive_failures_on_success() {
        // Test that after a retry eventually succeeds, consecutive_failures is reset.
        // We set up 2 failures followed by success within the retry limit.
        let mock_server = MockServer::start().await;

        // First 2 requests fail, 3rd succeeds
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .up_to_n_times(2)
            .expect(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Use retry_limit=5 to allow recovery
        let config = create_config_with_retry_limit(&mock_server.uri(), 5);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // Flush should eventually succeed after retries
        let result = batcher.flush_with_delay(1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        // consecutive_failures should be reset to 0 on success
        assert_eq!(batcher.consecutive_failures, 0);
        assert!(batcher.is_empty());
    }

    #[tokio::test]
    async fn test_flush_sends_correct_json_body() {
        let mock_server = MockServer::start().await;

        // Use a custom matcher to capture and verify the body
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        let event = create_test_event_with_source("my-source");
        let event_id = event.id.clone();
        batcher.queue(event);

        let result = batcher.flush().await;
        assert!(result.is_ok());

        // Verify request was made
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        // Verify body is valid JSON array
        let body: Vec<serde_json::Value> =
            serde_json::from_slice(&requests[0].body).expect("body should be valid JSON array");
        assert_eq!(body.len(), 1);
        assert_eq!(body[0]["id"], event_id);
        assert_eq!(body[0]["source"], "my-source");
    }

    #[tokio::test]
    async fn test_flush_signature_is_valid_base64() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = create_config_with_url(&mock_server.uri());
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));
        batcher.flush().await.unwrap();

        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        // Get signature header
        let signature = requests[0]
            .headers
            .get("X-Signature")
            .expect("X-Signature header should exist")
            .to_str()
            .unwrap();

        // Verify it's valid base64
        use base64::prelude::*;
        let decoded = BASE64_STANDARD.decode(signature);
        assert!(decoded.is_ok(), "Signature should be valid base64");
        assert_eq!(
            decoded.unwrap().len(),
            64,
            "Ed25519 signature should be 64 bytes"
        );
    }

    // =========================================================================
    // FR-015: Retry Logic Tests
    // =========================================================================

    /// Helper to create config with custom retry limit and fast delays for testing.
    fn create_config_with_retry_limit(url: &str, retry_limit: u8) -> PersistenceConfig {
        PersistenceConfig {
            supabase_url: url.to_string(),
            batch_interval_secs: 60,
            retry_limit,
        }
    }

    #[tokio::test]
    async fn test_flush_retries_on_server_error() {
        let mock_server = MockServer::start().await;

        // Mock returns 500 twice, then succeeds on third attempt
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .up_to_n_times(2)
            .expect(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 5);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // Use fast delay (1ms) for testing
        let result = batcher.flush_with_delay(1).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert!(batcher.is_empty());
        assert_eq!(batcher.consecutive_failures, 0);

        // Verify 3 requests were made (2 failures + 1 success)
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 3);
    }

    #[tokio::test]
    async fn test_flush_respects_retry_limit() {
        let mock_server = MockServer::start().await;

        // Mock always returns 500
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .expect(3) // Should be called exactly 3 times (retry_limit = 3)
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 3);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // Use fast delay (1ms) for testing
        let result = batcher.flush_with_delay(1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PersistenceError::MaxRetriesExceeded { attempts } => {
                assert_eq!(attempts, 3);
            }
            other => panic!("Expected MaxRetriesExceeded, got {:?}", other),
        }

        // Verify exactly 3 requests were made
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 3);
    }

    #[tokio::test]
    async fn test_flush_does_not_retry_on_401() {
        let mock_server = MockServer::start().await;

        // Mock returns 401 - should NOT be retried
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1) // Should only be called once
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 5);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // Use fast delay (1ms) for testing
        let result = batcher.flush_with_delay(1).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PersistenceError::AuthFailed));

        // Buffer should NOT be cleared on auth error
        assert_eq!(batcher.buffer_len(), 1);
        assert_eq!(batcher.consecutive_failures, 1);

        // Verify only 1 request was made (no retries)
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn test_flush_clears_buffer_after_max_retries() {
        let mock_server = MockServer::start().await;

        // Mock always returns 500
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 3);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        // Queue multiple events
        batcher.queue(create_test_event_with_source("monitor"));
        batcher.queue(create_test_event_with_source("monitor"));
        batcher.queue(create_test_event_with_source("monitor"));
        assert_eq!(batcher.buffer_len(), 3);

        // Use fast delay (1ms) for testing
        let result = batcher.flush_with_delay(1).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PersistenceError::MaxRetriesExceeded { .. }
        ));

        // Buffer should be cleared after max retries (batch dropped)
        assert!(batcher.is_empty());
    }

    #[tokio::test]
    async fn test_flush_resets_failures_after_max_retries() {
        let mock_server = MockServer::start().await;

        // Mock always returns 500 for first batch
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 3);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // First flush should fail after max retries
        let result = batcher.flush_with_delay(1).await;
        assert!(result.is_err());

        // consecutive_failures should be reset to 0 after dropping batch
        assert_eq!(batcher.consecutive_failures, 0);

        // Now queue another event and reset mock to succeed
        mock_server.reset().await;
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        batcher.queue(create_test_event_with_source("monitor"));
        let result = batcher.flush_with_delay(1).await;

        // Second flush should succeed without accumulated failures affecting it
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_flush_exponential_backoff_timing() {
        let mock_server = MockServer::start().await;

        // Mock returns 500 for first 3 attempts, then succeeds
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .up_to_n_times(3)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 5);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));

        // Use 10ms initial delay to verify exponential backoff
        // Expected delays: 10ms, 20ms, 40ms = ~70ms minimum total
        let start = std::time::Instant::now();
        let result = batcher.flush_with_delay(10).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Allow some tolerance for test execution time
        // Minimum: 10 + 20 + 40 = 70ms of sleep time
        assert!(
            elapsed.as_millis() >= 60,
            "Expected at least 60ms elapsed, got {}ms",
            elapsed.as_millis()
        );
    }

    #[tokio::test]
    async fn test_flush_retry_with_configurable_limit() {
        let mock_server = MockServer::start().await;

        // Test with retry_limit = 1 (no retries after first failure)
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 1);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));
        let result = batcher.flush_with_delay(1).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PersistenceError::MaxRetriesExceeded { attempts } => {
                assert_eq!(attempts, 1);
            }
            other => panic!("Expected MaxRetriesExceeded, got {:?}", other),
        }

        // Only 1 request should be made with retry_limit = 1
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn test_flush_succeeds_on_last_retry() {
        let mock_server = MockServer::start().await;

        // Mock returns 500 for first 2 attempts, succeeds on 3rd (retry_limit = 3)
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = create_config_with_retry_limit(&mock_server.uri(), 3);
        let mut batcher = EventBatcher::new(config, create_test_crypto());

        batcher.queue(create_test_event_with_source("monitor"));
        let result = batcher.flush_with_delay(1).await;

        // Should succeed on the 3rd attempt (just before hitting limit)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
        assert!(batcher.is_empty());
        assert_eq!(batcher.consecutive_failures, 0);
    }
}

/// Tests for PersistenceManager timer-based batching (FR-002).
#[cfg(test)]
mod persistence_manager_tests {
    use super::*;
    use crate::types::{EventPayload, EventType, SessionAction};
    use std::time::Duration;
    use uuid::Uuid;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_crypto() -> Crypto {
        Crypto::generate()
    }

    fn create_test_event_with_source(source: &str) -> Event {
        Event::new(
            source.to_string(),
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        )
    }

    #[test]
    fn test_persistence_manager_new_creates_channel() {
        let config = PersistenceConfig {
            supabase_url: "https://test.supabase.co/functions/v1".to_string(),
            batch_interval_secs: 60,
            retry_limit: 3,
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Channel should be open
        assert!(!sender.is_closed());

        // Manager's batcher should be empty
        assert!(manager.batcher.is_empty());
    }

    #[test]
    fn test_persistence_manager_config_accessor() {
        let config = PersistenceConfig {
            supabase_url: "https://test.supabase.co/functions/v1".to_string(),
            batch_interval_secs: 120,
            retry_limit: 5,
        };
        let (manager, _sender) = PersistenceManager::new(config, create_test_crypto());

        assert_eq!(manager.batcher.config().batch_interval_secs, 120);
        assert_eq!(manager.batcher.config().retry_limit, 5);
    }

    #[tokio::test]
    async fn test_persistence_manager_flushes_on_full_buffer() {
        let mock_server = MockServer::start().await;

        // Mock expects exactly 1 batch request
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Use a very long interval so timer doesn't trigger
        let config = PersistenceConfig {
            supabase_url: mock_server.uri(),
            batch_interval_secs: 3600, // 1 hour - won't trigger
            retry_limit: 3,
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Spawn manager
        let handle = tokio::spawn(manager.run());

        // Send exactly MAX_BATCH_SIZE events to trigger immediate flush
        for _ in 0..MAX_BATCH_SIZE {
            sender
                .send(create_test_event_with_source("test-monitor"))
                .await
                .unwrap();
        }

        // Give time for the flush to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Drop sender to trigger shutdown
        drop(sender);

        // Wait for manager to finish
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("manager should shut down within timeout")
            .expect("manager task should complete successfully");

        // Verify exactly 1 batch was sent
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        // Verify batch contained MAX_BATCH_SIZE events
        let body: Vec<serde_json::Value> = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body.len(), MAX_BATCH_SIZE);
    }

    #[tokio::test]
    async fn test_persistence_manager_flushes_on_interval() {
        let mock_server = MockServer::start().await;

        // Mock expects at least 1 batch request
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1..)
            .mount(&mock_server)
            .await;

        // Use 1 second interval (minimum allowed)
        let config = PersistenceConfig {
            supabase_url: mock_server.uri(),
            batch_interval_secs: 1,
            retry_limit: 3,
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Spawn manager
        let handle = tokio::spawn(manager.run());

        // Send a few events (far below MAX_BATCH_SIZE)
        for _ in 0..5 {
            sender
                .send(create_test_event_with_source("test-monitor"))
                .await
                .unwrap();
        }

        // Wait for timer to trigger (1 second + buffer)
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Drop sender to trigger shutdown
        drop(sender);

        // Wait for manager to finish
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("manager should shut down within timeout")
            .expect("manager task should complete successfully");

        // Verify at least 1 batch was sent by timer
        let requests = mock_server.received_requests().await.unwrap();
        assert!(
            !requests.is_empty(),
            "Expected at least 1 request from timer flush"
        );

        // Verify batch contained 5 events
        let body: Vec<serde_json::Value> = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body.len(), 5);
    }

    #[tokio::test]
    async fn test_persistence_manager_shuts_down_cleanly() {
        let mock_server = MockServer::start().await;

        // Mock expects 1 batch request (from shutdown flush)
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Use a very long interval so timer doesn't trigger
        let config = PersistenceConfig {
            supabase_url: mock_server.uri(),
            batch_interval_secs: 3600,
            retry_limit: 3,
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Spawn manager
        let handle = tokio::spawn(manager.run());

        // Send a few events (below threshold for immediate flush)
        for _ in 0..10 {
            sender
                .send(create_test_event_with_source("test-monitor"))
                .await
                .unwrap();
        }

        // Small delay to ensure events are received
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Drop sender to trigger shutdown
        drop(sender);

        // Wait for manager to finish (should flush remaining events)
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("manager should shut down within timeout")
            .expect("manager task should complete successfully");

        // Verify the remaining events were flushed on shutdown
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);

        let body: Vec<serde_json::Value> = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body.len(), 10);
    }

    #[tokio::test]
    async fn test_persistence_manager_handles_flush_errors_gracefully() {
        let mock_server = MockServer::start().await;

        // Mock returns 500 errors for all requests
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .mount(&mock_server)
            .await;

        // Use short retry limit to speed up test
        let config = PersistenceConfig {
            supabase_url: mock_server.uri(),
            batch_interval_secs: 3600, // Long interval
            retry_limit: 1,            // Fail quickly
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Spawn manager
        let handle = tokio::spawn(manager.run());

        // Send MAX_BATCH_SIZE events to trigger immediate flush
        for _ in 0..MAX_BATCH_SIZE {
            sender
                .send(create_test_event_with_source("test-monitor"))
                .await
                .unwrap();
        }

        // Give time for the (failing) flush
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Manager should still be running despite error
        assert!(
            !handle.is_finished(),
            "Manager should continue running after flush error"
        );

        // Drop sender to trigger shutdown
        drop(sender);

        // Manager should shut down gracefully even after errors
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("manager should shut down within timeout")
            .expect("manager task should complete successfully");
    }

    #[tokio::test]
    async fn test_persistence_manager_no_flush_when_empty() {
        let mock_server = MockServer::start().await;

        // Mock should NOT receive any requests
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&mock_server)
            .await;

        let config = PersistenceConfig {
            supabase_url: mock_server.uri(),
            batch_interval_secs: 1, // Short interval
            retry_limit: 3,
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Spawn manager
        let handle = tokio::spawn(manager.run());

        // Wait for timer to tick (but buffer is empty)
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Drop sender without sending any events
        drop(sender);

        // Wait for manager to finish
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("manager should shut down within timeout")
            .expect("manager task should complete successfully");

        // Verify no requests were made (empty buffer shouldn't trigger flush)
        let requests = mock_server.received_requests().await.unwrap();
        assert!(requests.is_empty(), "Should not flush empty buffer");
    }

    #[tokio::test]
    async fn test_persistence_manager_multiple_batches() {
        let mock_server = MockServer::start().await;

        // Mock expects 2 batch requests
        Mock::given(method("POST"))
            .and(path("/ingest"))
            .respond_with(ResponseTemplate::new(200))
            .expect(2)
            .mount(&mock_server)
            .await;

        let config = PersistenceConfig {
            supabase_url: mock_server.uri(),
            batch_interval_secs: 3600, // Long interval
            retry_limit: 3,
        };
        let (manager, sender) = PersistenceManager::new(config, create_test_crypto());

        // Spawn manager
        let handle = tokio::spawn(manager.run());

        // Send first batch (triggers immediate flush)
        for _ in 0..MAX_BATCH_SIZE {
            sender
                .send(create_test_event_with_source("test-monitor"))
                .await
                .unwrap();
        }

        // Give time for first flush
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send second batch (triggers immediate flush)
        for _ in 0..MAX_BATCH_SIZE {
            sender
                .send(create_test_event_with_source("test-monitor"))
                .await
                .unwrap();
        }

        // Give time for second flush
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Drop sender to trigger shutdown
        drop(sender);

        // Wait for manager to finish
        tokio::time::timeout(Duration::from_secs(5), handle)
            .await
            .expect("manager should shut down within timeout")
            .expect("manager task should complete successfully");

        // Verify 2 batches were sent
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
    }
}
