//! HTTP sender for VibeTea Monitor.
//!
//! This module handles sending events to the VibeTea server with:
//!
//! - Connection pooling via reqwest
//! - Event buffering (1000 events max, FIFO eviction)
//! - Exponential backoff retry (1s → 60s max, ±25% jitter)
//! - Rate limit handling (429 with Retry-After header)
//!
//! # Example
//!
//! ```no_run
//! use vibetea_monitor::sender::{Sender, SenderConfig};
//! use vibetea_monitor::crypto::Crypto;
//! use vibetea_monitor::types::{Event, EventType, EventPayload, SessionAction};
//! use std::path::Path;
//! use uuid::Uuid;
//!
//! #[tokio::main]
//! async fn main() {
//!     let crypto = Crypto::load(Path::new("/home/user/.vibetea")).unwrap();
//!     let config = SenderConfig::new(
//!         "https://vibetea.fly.dev".to_string(),
//!         "my-monitor".to_string(),
//!         1000,
//!     );
//!
//!     let mut sender = Sender::new(config, crypto);
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
//!     sender.send(event).await.unwrap();
//! }
//! ```

use std::collections::VecDeque;
use std::time::Duration;

use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, RETRY_AFTER};
use reqwest::{Client, StatusCode};
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::crypto::Crypto;
use crate::types::Event;

/// Initial retry delay in seconds.
const INITIAL_RETRY_DELAY_SECS: u64 = 1;

/// Maximum retry delay in seconds.
const MAX_RETRY_DELAY_SECS: u64 = 60;

/// Jitter factor (±25%).
const JITTER_FACTOR: f64 = 0.25;

/// Default buffer capacity.
const DEFAULT_BUFFER_SIZE: usize = 1000;

/// Maximum number of retry attempts before giving up on a batch.
const MAX_RETRY_ATTEMPTS: u32 = 10;

/// HTTP request timeout.
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Errors that can occur during event sending.
#[derive(Error, Debug)]
pub enum SenderError {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Server returned an error status.
    #[error("server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    /// Authentication failed (401).
    #[error("authentication failed: invalid signature or source ID")]
    AuthFailed,

    /// Rate limited (429).
    #[error("rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    /// Buffer is full and oldest events were evicted.
    #[error("buffer overflow: {evicted_count} events evicted")]
    BufferOverflow { evicted_count: usize },

    /// Maximum retry attempts exceeded.
    #[error("max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid header value (source_id or signature contains invalid characters).
    #[error("invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
}

/// Configuration for the sender.
#[derive(Debug, Clone)]
pub struct SenderConfig {
    /// Server URL (e.g., `https://vibetea.fly.dev`).
    pub server_url: String,

    /// Source ID for this monitor.
    pub source_id: String,

    /// Maximum number of events to buffer.
    pub buffer_size: usize,
}

impl SenderConfig {
    /// Creates a new sender configuration.
    #[must_use]
    pub fn new(server_url: String, source_id: String, buffer_size: usize) -> Self {
        Self {
            server_url,
            source_id,
            buffer_size,
        }
    }

    /// Creates a configuration with default buffer size.
    #[must_use]
    pub fn with_defaults(server_url: String, source_id: String) -> Self {
        Self::new(server_url, source_id, DEFAULT_BUFFER_SIZE)
    }
}

/// HTTP event sender with buffering and retry logic.
pub struct Sender {
    config: SenderConfig,
    crypto: Crypto,
    client: Client,
    buffer: VecDeque<Event>,
    current_retry_delay: Duration,
}

impl Sender {
    /// Creates a new sender with the given configuration and cryptographic context.
    ///
    /// # Arguments
    ///
    /// * `config` - Sender configuration
    /// * `crypto` - Cryptographic context for signing events
    #[must_use]
    pub fn new(config: SenderConfig, crypto: Crypto) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            buffer: VecDeque::with_capacity(config.buffer_size),
            config,
            crypto,
            client,
            current_retry_delay: Duration::from_secs(INITIAL_RETRY_DELAY_SECS),
        }
    }

    /// Queues an event for sending.
    ///
    /// If the buffer is full, the oldest events are evicted to make room.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to queue
    ///
    /// # Returns
    ///
    /// The number of events evicted (0 if buffer had space).
    pub fn queue(&mut self, event: Event) -> usize {
        let mut evicted = 0;

        // Evict oldest events if buffer is full
        while self.buffer.len() >= self.config.buffer_size {
            self.buffer.pop_front();
            evicted += 1;
        }

        self.buffer.push_back(event);

        if evicted > 0 {
            warn!(evicted_count = evicted, "Buffer overflow, events evicted");
        }

        evicted
    }

    /// Returns the number of events currently in the buffer.
    #[must_use]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Sends a single event immediately without buffering.
    ///
    /// This method will retry with exponential backoff on transient failures.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to send
    ///
    /// # Errors
    ///
    /// Returns `SenderError` if the event cannot be sent after all retries.
    pub async fn send(&mut self, event: Event) -> Result<(), SenderError> {
        self.send_batch(&[event]).await
    }

    /// Flushes all buffered events to the server.
    ///
    /// Events are sent in a single batch. On success, the buffer is cleared.
    /// On failure, events remain in the buffer for later retry.
    ///
    /// # Errors
    ///
    /// Returns `SenderError` if the batch cannot be sent after all retries.
    pub async fn flush(&mut self) -> Result<(), SenderError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let events: Vec<Event> = self.buffer.iter().cloned().collect();
        self.send_batch(&events).await?;

        // Clear buffer on success
        self.buffer.clear();
        self.reset_retry_delay();

        Ok(())
    }

    /// Sends a batch of events to the server with retry logic.
    async fn send_batch(&mut self, events: &[Event]) -> Result<(), SenderError> {
        let url = format!("{}/events", self.config.server_url);
        let body = serde_json::to_string(events)?;
        let signature = self.crypto.sign(body.as_bytes());

        let mut attempts = 0;

        loop {
            attempts += 1;

            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            headers.insert(
                "X-Source-Id",
                HeaderValue::from_str(&self.config.source_id)?,
            );
            headers.insert("X-Signature", HeaderValue::from_str(&signature)?);

            debug!(
                url = %url,
                events = events.len(),
                attempt = attempts,
                "Sending event batch"
            );

            let result = self
                .client
                .post(&url)
                .headers(headers)
                .body(body.clone())
                .send()
                .await;

            match result {
                Ok(response) => {
                    let status = response.status();

                    match status {
                        StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
                            info!(events = events.len(), "Events sent successfully");
                            self.reset_retry_delay();
                            return Ok(());
                        }
                        StatusCode::UNAUTHORIZED => {
                            error!("Authentication failed");
                            return Err(SenderError::AuthFailed);
                        }
                        StatusCode::TOO_MANY_REQUESTS => {
                            let retry_after = self.parse_retry_after(&response);
                            warn!(retry_after_secs = retry_after, "Rate limited by server");

                            if attempts >= MAX_RETRY_ATTEMPTS {
                                return Err(SenderError::MaxRetriesExceeded { attempts });
                            }

                            sleep(Duration::from_secs(retry_after)).await;
                            continue;
                        }
                        _ if status.is_server_error() => {
                            let message = response.text().await.unwrap_or_default();
                            warn!(
                                status = status.as_u16(),
                                message = %message,
                                "Server error, will retry"
                            );

                            if attempts >= MAX_RETRY_ATTEMPTS {
                                return Err(SenderError::ServerError {
                                    status: status.as_u16(),
                                    message,
                                });
                            }

                            self.wait_with_backoff().await;
                            continue;
                        }
                        _ => {
                            let message = response.text().await.unwrap_or_default();
                            return Err(SenderError::ServerError {
                                status: status.as_u16(),
                                message,
                            });
                        }
                    }
                }
                Err(e) => {
                    if e.is_timeout() || e.is_connect() {
                        warn!(error = %e, "Connection error, will retry");

                        if attempts >= MAX_RETRY_ATTEMPTS {
                            return Err(SenderError::MaxRetriesExceeded { attempts });
                        }

                        self.wait_with_backoff().await;
                        continue;
                    }

                    return Err(SenderError::Http(e));
                }
            }
        }
    }

    /// Parses the Retry-After header from a 429 response.
    fn parse_retry_after(&self, response: &reqwest::Response) -> u64 {
        response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(self.current_retry_delay.as_secs())
    }

    /// Waits for the current retry delay with jitter, then increases the delay.
    async fn wait_with_backoff(&mut self) {
        let delay = self.add_jitter(self.current_retry_delay);
        debug!(delay_ms = delay.as_millis(), "Waiting before retry");
        sleep(delay).await;
        self.increase_retry_delay();
    }

    /// Adds ±25% jitter to a duration.
    fn add_jitter(&self, duration: Duration) -> Duration {
        let mut rng = rand::rng();
        let jitter_range = duration.as_secs_f64() * JITTER_FACTOR;
        let jitter = rng.random_range(-jitter_range..=jitter_range);
        let new_secs = (duration.as_secs_f64() + jitter).max(0.1);
        Duration::from_secs_f64(new_secs)
    }

    /// Doubles the retry delay up to the maximum.
    fn increase_retry_delay(&mut self) {
        let new_secs = (self.current_retry_delay.as_secs() * 2).min(MAX_RETRY_DELAY_SECS);
        self.current_retry_delay = Duration::from_secs(new_secs);
    }

    /// Resets the retry delay to the initial value.
    fn reset_retry_delay(&mut self) {
        self.current_retry_delay = Duration::from_secs(INITIAL_RETRY_DELAY_SECS);
    }

    /// Gracefully shuts down the sender, attempting to flush any remaining events.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for flush to complete
    ///
    /// # Returns
    ///
    /// The number of events that could not be sent.
    pub async fn shutdown(&mut self, timeout: Duration) -> usize {
        if self.buffer.is_empty() {
            return 0;
        }

        info!(
            buffered_events = self.buffer.len(),
            "Flushing buffer before shutdown"
        );

        let flush_future = self.flush();
        match tokio::time::timeout(timeout, flush_future).await {
            Ok(Ok(())) => 0,
            Ok(Err(e)) => {
                error!(error = %e, "Failed to flush buffer during shutdown");
                self.buffer.len()
            }
            Err(_) => {
                error!("Timeout while flushing buffer during shutdown");
                self.buffer.len()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventPayload, EventType, SessionAction};
    use uuid::Uuid;

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

    fn create_test_crypto() -> Crypto {
        Crypto::generate()
    }

    fn create_test_sender() -> Sender {
        let config = SenderConfig::new(
            "http://localhost:8080".to_string(),
            "test-monitor".to_string(),
            10, // Small buffer for testing
        );
        Sender::new(config, create_test_crypto())
    }

    #[test]
    fn test_queue_adds_events() {
        let mut sender = create_test_sender();
        assert!(sender.is_empty());

        sender.queue(create_test_event());
        assert_eq!(sender.buffer_len(), 1);

        sender.queue(create_test_event());
        assert_eq!(sender.buffer_len(), 2);
    }

    #[test]
    fn test_queue_evicts_oldest_when_full() {
        let mut sender = create_test_sender();

        // Fill buffer to capacity (10 events)
        for _ in 0..10 {
            let evicted = sender.queue(create_test_event());
            assert_eq!(evicted, 0);
        }
        assert_eq!(sender.buffer_len(), 10);

        // Add one more - should evict oldest
        let evicted = sender.queue(create_test_event());
        assert_eq!(evicted, 1);
        assert_eq!(sender.buffer_len(), 10);
    }

    #[test]
    fn test_sender_config_with_defaults() {
        let config = SenderConfig::with_defaults(
            "https://example.com".to_string(),
            "my-monitor".to_string(),
        );
        assert_eq!(config.buffer_size, DEFAULT_BUFFER_SIZE);
    }

    #[test]
    fn test_add_jitter_stays_within_bounds() {
        let sender = create_test_sender();
        let base = Duration::from_secs(10);

        // Run multiple times to test randomness bounds
        for _ in 0..100 {
            let jittered = sender.add_jitter(base);
            let secs = jittered.as_secs_f64();
            // Should be within ±25% of 10 seconds
            assert!(
                (7.5..=12.5).contains(&secs),
                "Jitter out of bounds: {}",
                secs
            );
        }
    }

    #[test]
    fn test_increase_retry_delay_doubles() {
        let mut sender = create_test_sender();
        assert_eq!(
            sender.current_retry_delay.as_secs(),
            INITIAL_RETRY_DELAY_SECS
        );

        sender.increase_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), 2);

        sender.increase_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), 4);
    }

    #[test]
    fn test_increase_retry_delay_caps_at_max() {
        let mut sender = create_test_sender();
        sender.current_retry_delay = Duration::from_secs(MAX_RETRY_DELAY_SECS);

        sender.increase_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn test_reset_retry_delay() {
        let mut sender = create_test_sender();
        sender.current_retry_delay = Duration::from_secs(30);

        sender.reset_retry_delay();
        assert_eq!(
            sender.current_retry_delay.as_secs(),
            INITIAL_RETRY_DELAY_SECS
        );
    }

    #[test]
    fn test_is_empty() {
        let mut sender = create_test_sender();
        assert!(sender.is_empty());

        sender.queue(create_test_event());
        assert!(!sender.is_empty());
    }
}
