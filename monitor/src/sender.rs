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

/// Maximum payload size per request (slightly under 1MB to leave room for headers/overhead).
const MAX_CHUNK_SIZE: usize = 900 * 1024;

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
    /// Events are sent in chunks that fit within the server's body size limit.
    /// On success, the buffer is cleared. On failure, remaining events stay
    /// in the buffer for later retry.
    ///
    /// # Errors
    ///
    /// Returns `SenderError` if a chunk cannot be sent after all retries.
    pub async fn flush(&mut self) -> Result<(), SenderError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Chunk events to stay under server's body size limit
        let events: Vec<Event> = self.buffer.iter().cloned().collect();
        let chunks = self.chunk_events(&events);

        debug!(
            total_events = events.len(),
            chunks = chunks.len(),
            "Flushing events in chunks"
        );

        for chunk in chunks {
            self.send_batch(&chunk).await?;
        }

        // Clear buffer on success
        self.buffer.clear();
        self.reset_retry_delay();

        Ok(())
    }

    /// Chunks events into batches that fit within the size limit.
    ///
    /// Events larger than `MAX_CHUNK_SIZE` are placed in their own chunk with a
    /// warning logged, as they may fail to send. The server should reject payloads
    /// over its limit, and the retry logic will eventually drop them.
    fn chunk_events(&self, events: &[Event]) -> Vec<Vec<Event>> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut current_size = 2; // Start with "[]" for empty array

        for event in events {
            // Estimate serialized size (actual JSON may be slightly different)
            let event_size = serde_json::to_string(event)
                .map(|s| s.len())
                .unwrap_or(1000);

            // Check if single event exceeds chunk size
            if event_size > MAX_CHUNK_SIZE {
                warn!(
                    event_id = %event.id,
                    event_size = event_size,
                    max_size = MAX_CHUNK_SIZE,
                    "Event exceeds maximum chunk size, placing in separate chunk"
                );
                // Flush current chunk first if non-empty
                if !current_chunk.is_empty() {
                    chunks.push(std::mem::take(&mut current_chunk));
                    current_size = 2;
                }
                // Put oversized event in its own chunk
                chunks.push(vec![event.clone()]);
                continue;
            }

            // Account for comma separator
            let separator_size = if current_chunk.is_empty() { 0 } else { 1 };

            if current_size + separator_size + event_size > MAX_CHUNK_SIZE && !current_chunk.is_empty() {
                // Start a new chunk
                chunks.push(std::mem::take(&mut current_chunk));
                current_size = 2;
            }

            current_chunk.push(event.clone());
            current_size += event_size + if current_chunk.len() > 1 { 1 } else { 0 };
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
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
                        StatusCode::PAYLOAD_TOO_LARGE => {
                            // Log and skip this chunk - don't let oversized events block
                            // subsequent valid events. The chunk_events function already
                            // isolates oversized events into their own chunks.
                            warn!(
                                events = events.len(),
                                "Payload too large (413), dropping oversized chunk"
                            );
                            return Ok(());
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

    #[test]
    fn test_chunk_events_small_batch() {
        let sender = create_test_sender();
        let events: Vec<Event> = (0..5).map(|_| create_test_event()).collect();

        let chunks = sender.chunk_events(&events);

        // Small batch should be a single chunk
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 5);
    }

    #[test]
    fn test_chunk_events_empty() {
        let sender = create_test_sender();
        let events: Vec<Event> = vec![];

        let chunks = sender.chunk_events(&events);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_events_splits_large_batch() {
        let sender = create_test_sender();

        // Create events with large context to force chunking
        let large_context = "x".repeat(10000); // 10KB per event context
        let events: Vec<Event> = (0..200).map(|_| {
            Event::new(
                "test-monitor".to_string(),
                EventType::Tool,
                EventPayload::Tool {
                    session_id: Uuid::new_v4(),
                    tool: "Read".to_string(),
                    status: crate::types::ToolStatus::Completed,
                    context: Some(large_context.clone()),
                    project: Some("test".to_string()),
                },
            )
        }).collect();

        let chunks = sender.chunk_events(&events);

        // Should have multiple chunks
        assert!(chunks.len() > 1, "Expected multiple chunks, got {}", chunks.len());

        // Total events should match
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, 200);

        // Each chunk should serialize to under the limit
        for chunk in &chunks {
            let size = serde_json::to_string(chunk).unwrap().len();
            assert!(size <= MAX_CHUNK_SIZE + 1000, "Chunk too large: {} bytes", size);
        }
    }

    #[test]
    fn test_chunk_events_handles_oversized_single_event() {
        let sender = create_test_sender();

        // Create an event larger than MAX_CHUNK_SIZE (900KB)
        // Each character in JSON string takes ~1 byte plus escaping overhead
        let oversized_context = "x".repeat(MAX_CHUNK_SIZE + 1000);
        let oversized_event = Event::new(
            "test-monitor".to_string(),
            EventType::Tool,
            EventPayload::Tool {
                session_id: Uuid::new_v4(),
                tool: "Read".to_string(),
                status: crate::types::ToolStatus::Completed,
                context: Some(oversized_context),
                project: Some("test".to_string()),
            },
        );

        // Verify the event is actually oversized
        let event_size = serde_json::to_string(&oversized_event).unwrap().len();
        assert!(
            event_size > MAX_CHUNK_SIZE,
            "Test event should be larger than MAX_CHUNK_SIZE, got {} bytes",
            event_size
        );

        // Mix oversized event with normal events
        let normal_event = create_test_event();
        let events = vec![
            normal_event.clone(),
            oversized_event.clone(),
            normal_event.clone(),
        ];

        let chunks = sender.chunk_events(&events);

        // Should have at least 2 chunks: one for normal events, one for oversized
        assert!(
            chunks.len() >= 2,
            "Expected at least 2 chunks, got {}",
            chunks.len()
        );

        // Total events should match
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, 3, "All events should be included");

        // Find the chunk with the oversized event
        let oversized_chunk = chunks.iter().find(|c| {
            c.len() == 1 && {
                let size = serde_json::to_string(&c[0]).unwrap().len();
                size > MAX_CHUNK_SIZE
            }
        });
        assert!(
            oversized_chunk.is_some(),
            "Oversized event should be in its own chunk"
        );
    }

    #[test]
    fn test_chunk_events_oversized_only() {
        let sender = create_test_sender();

        // Create only oversized events
        let oversized_context = "y".repeat(MAX_CHUNK_SIZE + 500);
        let events: Vec<Event> = (0..3)
            .map(|_| {
                Event::new(
                    "test-monitor".to_string(),
                    EventType::Tool,
                    EventPayload::Tool {
                        session_id: Uuid::new_v4(),
                        tool: "Write".to_string(),
                        status: crate::types::ToolStatus::Completed,
                        context: Some(oversized_context.clone()),
                        project: Some("test".to_string()),
                    },
                )
            })
            .collect();

        let chunks = sender.chunk_events(&events);

        // Each oversized event should be in its own chunk
        assert_eq!(
            chunks.len(),
            3,
            "Each oversized event should be in its own chunk"
        );

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(
                chunk.len(),
                1,
                "Chunk {} should contain exactly 1 event",
                i
            );
        }
    }
}
