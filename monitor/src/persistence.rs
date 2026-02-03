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

use reqwest::Client;
use thiserror::Error;

use crate::config::PersistenceConfig;
use crate::crypto::Crypto;
use crate::types::Event;

/// Maximum number of events per batch (per FR-002).
const MAX_BATCH_SIZE: usize = 1000;

/// HTTP request timeout in seconds.
const REQUEST_TIMEOUT_SECS: u64 = 30;

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
    /// This method is a no-op stub for now. Implementation will be added
    /// in a later task.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to queue for persistence
    #[allow(unused_variables)]
    pub fn queue(&mut self, event: Event) {
        // Stub: implementation coming in later task
    }

    /// Flushes all buffered events to the Supabase edge function.
    ///
    /// This method is a stub that returns `Ok(0)` for now. Implementation
    /// will be added in a later task.
    ///
    /// # Returns
    ///
    /// The number of events successfully sent.
    ///
    /// # Errors
    ///
    /// Returns `PersistenceError` if:
    /// - The HTTP request fails
    /// - JSON serialization fails
    /// - The server returns an authentication error (401)
    /// - The server returns an error status
    /// - Maximum retry attempts are exceeded
    pub async fn flush(&mut self) -> Result<usize, PersistenceError> {
        // Stub: implementation coming in later task
        Ok(0)
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_flush_returns_zero_for_stub() {
        let mut batcher = EventBatcher::new(create_test_config(), create_test_crypto());

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
}
