//! Debounce utility for coalescing rapid events.
//!
//! This module provides a debouncer that delays processing events until a
//! specified duration has passed since the last event for a given key. This
//! is particularly useful for file system events where rapid consecutive
//! changes should be coalesced into a single event.
//!
//! # Architecture
//!
//! The debouncer uses a background task that maintains a map of pending events
//! keyed by a user-defined key type (typically a file path). When events arrive:
//!
//! 1. The event replaces any existing pending event for that key
//! 2. A timer is reset for that key
//! 3. When the timer expires (no new events for the debounce duration), the
//!    final event value is emitted
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use std::time::Duration;
//! use tokio::sync::mpsc;
//! use vibetea_monitor::utils::debounce::Debouncer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (output_tx, mut output_rx) = mpsc::channel(100);
//!     let debouncer = Debouncer::new(Duration::from_millis(100), output_tx);
//!
//!     // Send multiple events for the same key
//!     let path = PathBuf::from("/test/file.rs");
//!     debouncer.send(path.clone(), "event1".to_string()).await.unwrap();
//!     debouncer.send(path.clone(), "event2".to_string()).await.unwrap();
//!     debouncer.send(path.clone(), "event3".to_string()).await.unwrap();
//!
//!     // After the debounce duration, only "event3" will be emitted
//!     if let Some((key, value)) = output_rx.recv().await {
//!         assert_eq!(key, path);
//!         assert_eq!(value, "event3");
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, trace, warn};

/// Default debounce interval in milliseconds.
pub const DEFAULT_DEBOUNCE_MS: u64 = 100;

/// Error type for debouncer operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebouncerError {
    /// The debouncer's input channel has been closed.
    ChannelClosed,
}

impl std::fmt::Display for DebouncerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelClosed => write!(f, "debouncer channel closed"),
        }
    }
}

impl std::error::Error for DebouncerError {}

/// A pending event waiting for its debounce timer to expire.
#[derive(Debug)]
struct PendingEvent<V> {
    /// The event value to emit when the timer expires.
    value: V,
    /// When this event should be emitted (after debounce delay).
    deadline: Instant,
}

/// A debouncer that coalesces rapid events by key.
///
/// Events are held until a configurable duration has passed since the last
/// event for that key, at which point the final value is emitted.
///
/// # Type Parameters
///
/// * `K` - The key type used to group events (e.g., `PathBuf` for file paths)
/// * `V` - The value type for events
///
/// # Thread Safety
///
/// The debouncer is designed for use with Tokio async code. It maintains a
/// background task that processes the debounce logic, communicating via
/// channels for thread safety.
#[derive(Debug)]
pub struct Debouncer<K, V>
where
    K: Clone + Eq + Hash + Send + 'static,
    V: Clone + Send + 'static,
{
    /// Channel for sending events to the background task.
    input_tx: mpsc::Sender<(K, V)>,
    /// Handle to the background task (kept for cleanup).
    #[allow(dead_code)]
    task_handle: tokio::task::JoinHandle<()>,
}

impl<K, V> Debouncer<K, V>
where
    K: Clone + Eq + Hash + Send + std::fmt::Debug + 'static,
    V: Clone + Send + 'static,
{
    /// Creates a new debouncer with the specified interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - How long to wait after the last event before emitting
    /// * `output_tx` - Channel for emitting debounced events
    ///
    /// # Returns
    ///
    /// A new `Debouncer` instance with a running background task.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    /// use vibetea_monitor::utils::debounce::Debouncer;
    ///
    /// let (tx, rx) = mpsc::channel(100);
    /// let debouncer: Debouncer<String, i32> = Debouncer::new(
    ///     Duration::from_millis(150),
    ///     tx,
    /// );
    /// ```
    #[must_use]
    pub fn new(interval: Duration, output_tx: mpsc::Sender<(K, V)>) -> Self {
        let (input_tx, input_rx) = mpsc::channel(1000);

        let task_handle = tokio::spawn(async move {
            run_debounce_loop(interval, input_rx, output_tx).await;
        });

        Self {
            input_tx,
            task_handle,
        }
    }

    /// Creates a new debouncer with the default interval (100ms).
    ///
    /// # Arguments
    ///
    /// * `output_tx` - Channel for emitting debounced events
    ///
    /// # Returns
    ///
    /// A new `Debouncer` instance with a running background task.
    #[must_use]
    pub fn with_default_interval(output_tx: mpsc::Sender<(K, V)>) -> Self {
        Self::new(Duration::from_millis(DEFAULT_DEBOUNCE_MS), output_tx)
    }

    /// Sends an event to be debounced.
    ///
    /// If an event for the same key is already pending, its value is replaced
    /// and the timer is reset. The final value will be emitted after the
    /// debounce interval passes with no new events for that key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to group this event with (e.g., file path)
    /// * `value` - The event data
    ///
    /// # Errors
    ///
    /// Returns `DebouncerError::ChannelClosed` if the background task has
    /// terminated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use std::time::Duration;
    /// use tokio::sync::mpsc;
    /// use vibetea_monitor::utils::debounce::Debouncer;
    ///
    /// # async fn example() -> Result<(), vibetea_monitor::utils::debounce::DebouncerError> {
    /// let (tx, rx) = mpsc::channel(100);
    /// let debouncer: Debouncer<PathBuf, String> = Debouncer::new(
    ///     Duration::from_millis(100),
    ///     tx,
    /// );
    ///
    /// debouncer.send(
    ///     PathBuf::from("/path/to/file"),
    ///     "event data".to_string(),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(&self, key: K, value: V) -> Result<(), DebouncerError> {
        self.input_tx
            .send((key, value))
            .await
            .map_err(|_| DebouncerError::ChannelClosed)
    }

    /// Attempts to send an event without waiting.
    ///
    /// This is useful when called from synchronous code or when you don't
    /// want to block on channel capacity.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to group this event with
    /// * `value` - The event data
    ///
    /// # Returns
    ///
    /// `true` if the event was sent successfully, `false` if the channel
    /// is full or closed.
    pub fn try_send(&self, key: K, value: V) -> bool {
        self.input_tx.try_send((key, value)).is_ok()
    }
}

/// Runs the debounce loop, processing incoming events and emitting debounced results.
async fn run_debounce_loop<K, V>(
    interval: Duration,
    mut input_rx: mpsc::Receiver<(K, V)>,
    output_tx: mpsc::Sender<(K, V)>,
) where
    K: Clone + Eq + Hash + std::fmt::Debug,
    V: Clone,
{
    let mut pending: HashMap<K, PendingEvent<V>> = HashMap::new();

    // Tick interval for checking expired events
    // Use a fraction of the debounce interval for responsiveness
    let tick_interval = std::cmp::min(interval / 4, Duration::from_millis(25));

    debug!(
        interval_ms = interval.as_millis(),
        tick_ms = tick_interval.as_millis(),
        "Starting debounce loop"
    );

    loop {
        // Calculate the next deadline we need to wake up for
        let next_deadline = pending.values().map(|p| p.deadline).min();

        tokio::select! {
            // Receive new events
            event = input_rx.recv() => {
                match event {
                    Some((key, value)) => {
                        let deadline = Instant::now() + interval;
                        trace!(key = ?key, "Received event, setting deadline");
                        pending.insert(key, PendingEvent { value, deadline });
                    }
                    None => {
                        // Input channel closed, emit remaining events and exit
                        debug!("Input channel closed, flushing remaining events");
                        flush_all_pending(&mut pending, &output_tx).await;
                        break;
                    }
                }
            }

            // Wait for the next deadline or tick
            _ = async {
                match next_deadline {
                    Some(deadline) => tokio::time::sleep_until(deadline).await,
                    None => tokio::time::sleep(tick_interval).await,
                }
            } => {
                // Check for expired events
                emit_expired_events(&mut pending, &output_tx).await;
            }
        }
    }

    debug!("Debounce loop terminated");
}

/// Emits all events whose deadlines have passed.
async fn emit_expired_events<K, V>(
    pending: &mut HashMap<K, PendingEvent<V>>,
    output_tx: &mpsc::Sender<(K, V)>,
) where
    K: Clone + Eq + Hash + std::fmt::Debug,
    V: Clone,
{
    let now = Instant::now();

    // Collect keys of expired events
    let expired_keys: Vec<K> = pending
        .iter()
        .filter(|(_, event)| event.deadline <= now)
        .map(|(key, _)| key.clone())
        .collect();

    // Emit and remove expired events
    for key in expired_keys {
        if let Some(event) = pending.remove(&key) {
            trace!(key = ?key, "Emitting debounced event");
            if let Err(e) = output_tx.send((key.clone(), event.value)).await {
                warn!(key = ?key, error = %e, "Failed to emit debounced event");
            }
        }
    }
}

/// Flushes all pending events immediately, regardless of their deadlines.
async fn flush_all_pending<K, V>(
    pending: &mut HashMap<K, PendingEvent<V>>,
    output_tx: &mpsc::Sender<(K, V)>,
) where
    K: Clone + Eq + Hash + std::fmt::Debug,
    V: Clone,
{
    for (key, event) in pending.drain() {
        trace!(key = ?key, "Flushing pending event");
        if let Err(e) = output_tx.send((key.clone(), event.value)).await {
            warn!(key = ?key, error = %e, "Failed to flush pending event");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::time::{sleep, timeout};

    /// Helper to create a debouncer with a short interval for testing.
    fn test_debouncer<K, V>(interval_ms: u64) -> (Debouncer<K, V>, mpsc::Receiver<(K, V)>)
    where
        K: Clone + Eq + Hash + Send + std::fmt::Debug + 'static,
        V: Clone + Send + 'static,
    {
        let (tx, rx) = mpsc::channel(100);
        let debouncer = Debouncer::new(Duration::from_millis(interval_ms), tx);
        (debouncer, rx)
    }

    #[tokio::test]
    async fn test_single_event_emitted_after_interval() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        debouncer.send("key1".to_string(), 42).await.unwrap();

        // Event should be emitted after the interval
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event within timeout");

        let (key, value) = result.unwrap().unwrap();
        assert_eq!(key, "key1");
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_multiple_events_same_key_coalesced() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        // Send multiple events rapidly for the same key
        debouncer.send("key1".to_string(), 1).await.unwrap();
        debouncer.send("key1".to_string(), 2).await.unwrap();
        debouncer.send("key1".to_string(), 3).await.unwrap();

        // Only the last value should be emitted
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event within timeout");

        let (key, value) = result.unwrap().unwrap();
        assert_eq!(key, "key1");
        assert_eq!(value, 3, "Should emit the last value");

        // No more events should come
        let more = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(more.is_err(), "Should not receive additional events");
    }

    #[tokio::test]
    async fn test_different_keys_independent() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        // Send events for different keys
        debouncer.send("key1".to_string(), 100).await.unwrap();
        debouncer.send("key2".to_string(), 200).await.unwrap();
        debouncer.send("key3".to_string(), 300).await.unwrap();

        // All three should be emitted
        let mut received = Vec::new();
        for _ in 0..3 {
            let result = timeout(Duration::from_millis(200), rx.recv()).await;
            if let Ok(Some(event)) = result {
                received.push(event);
            }
        }

        assert_eq!(received.len(), 3, "Should receive 3 events");

        // Verify all keys were received
        let keys: Vec<_> = received.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"key1"));
        assert!(keys.contains(&"key2"));
        assert!(keys.contains(&"key3"));
    }

    #[tokio::test]
    async fn test_timer_reset_on_new_event() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(100);

        // Send initial event
        debouncer.send("key1".to_string(), 1).await.unwrap();

        // Wait less than the interval
        sleep(Duration::from_millis(50)).await;

        // Send another event, resetting the timer
        debouncer.send("key1".to_string(), 2).await.unwrap();

        // Wait less than the interval again
        sleep(Duration::from_millis(50)).await;

        // Send another event, resetting the timer again
        debouncer.send("key1".to_string(), 3).await.unwrap();

        // Now wait for the full interval plus some buffer
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event");

        let (_, value) = result.unwrap().unwrap();
        assert_eq!(value, 3, "Should emit the final value");
    }

    #[tokio::test]
    async fn test_path_buf_as_key() {
        let (debouncer, mut rx) = test_debouncer::<PathBuf, String>(50);

        let path = PathBuf::from("/test/file.jsonl");
        debouncer
            .send(path.clone(), "event1".to_string())
            .await
            .unwrap();
        debouncer
            .send(path.clone(), "event2".to_string())
            .await
            .unwrap();

        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok());

        let (key, value) = result.unwrap().unwrap();
        assert_eq!(key, path);
        assert_eq!(value, "event2");
    }

    #[tokio::test]
    async fn test_try_send_success() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        let sent = debouncer.try_send("key".to_string(), 42);
        assert!(sent, "try_send should succeed");

        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().1, 42);
    }

    #[tokio::test]
    async fn test_default_interval() {
        let (tx, mut rx) = mpsc::channel(100);
        let debouncer: Debouncer<String, i32> = Debouncer::with_default_interval(tx);

        debouncer.send("key".to_string(), 42).await.unwrap();

        // Default is 100ms, so wait a bit longer
        let result = timeout(Duration::from_millis(250), rx.recv()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().1, 42);
    }

    #[tokio::test]
    async fn test_flush_on_drop() {
        let (tx, mut rx) = mpsc::channel(100);
        let debouncer: Debouncer<String, i32> = Debouncer::new(
            Duration::from_millis(1000), // Long interval
            tx,
        );

        debouncer.send("key".to_string(), 42).await.unwrap();

        // Drop the debouncer (closes the input channel)
        drop(debouncer);

        // The pending event should be flushed
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok(), "Pending events should be flushed on close");

        let (key, value) = result.unwrap().unwrap();
        assert_eq!(key, "key");
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_high_frequency_events() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        // Send many events rapidly
        for i in 0..100 {
            debouncer.send("key".to_string(), i).await.unwrap();
        }

        // Only the last value should be emitted
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok());

        let (_, value) = result.unwrap().unwrap();
        assert_eq!(value, 99, "Should emit the last value in the series");

        // No more events
        let more = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(more.is_err());
    }

    #[tokio::test]
    async fn test_interleaved_events_multiple_keys() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        // Interleave events for two keys
        debouncer.send("a".to_string(), 1).await.unwrap();
        debouncer.send("b".to_string(), 10).await.unwrap();
        debouncer.send("a".to_string(), 2).await.unwrap();
        debouncer.send("b".to_string(), 20).await.unwrap();
        debouncer.send("a".to_string(), 3).await.unwrap();
        debouncer.send("b".to_string(), 30).await.unwrap();

        // Collect both events
        let mut received = HashMap::new();
        for _ in 0..2 {
            if let Ok(Some((key, value))) = timeout(Duration::from_millis(200), rx.recv()).await {
                received.insert(key, value);
            }
        }

        assert_eq!(received.len(), 2);
        assert_eq!(received.get("a"), Some(&3));
        assert_eq!(received.get("b"), Some(&30));
    }

    #[tokio::test]
    async fn test_debouncer_error_display() {
        let error = DebouncerError::ChannelClosed;
        assert_eq!(error.to_string(), "debouncer channel closed");
    }

    #[tokio::test]
    async fn test_debouncer_error_is_error_trait() {
        fn assert_error<E: std::error::Error>() {}
        assert_error::<DebouncerError>();
    }

    #[tokio::test]
    async fn test_event_not_emitted_before_interval() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(100);

        debouncer.send("key".to_string(), 42).await.unwrap();

        // Try to receive immediately - should timeout
        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err(), "Should not receive event before interval");

        // Now wait for the rest of the interval
        let result = timeout(Duration::from_millis(150), rx.recv()).await;
        assert!(result.is_ok(), "Should receive event after interval");
    }

    #[tokio::test]
    async fn test_complex_event_value() {
        #[derive(Debug, Clone, PartialEq)]
        struct ComplexEvent {
            lines: Vec<String>,
            timestamp: u64,
        }

        let (tx, mut rx) = mpsc::channel(100);
        let debouncer: Debouncer<PathBuf, ComplexEvent> =
            Debouncer::new(Duration::from_millis(50), tx);

        let path = PathBuf::from("/test/file.jsonl");
        let event = ComplexEvent {
            lines: vec!["line1".to_string(), "line2".to_string()],
            timestamp: 12345,
        };

        debouncer.send(path.clone(), event.clone()).await.unwrap();

        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_ok());

        let (key, value) = result.unwrap().unwrap();
        assert_eq!(key, path);
        assert_eq!(value, event);
    }

    #[tokio::test]
    async fn test_zero_interval_immediate_emit() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(0);

        debouncer.send("key".to_string(), 42).await.unwrap();

        // With zero interval, should emit very quickly
        let result = timeout(Duration::from_millis(100), rx.recv()).await;
        assert!(result.is_ok(), "Zero interval should emit quickly");
        assert_eq!(result.unwrap().unwrap().1, 42);
    }

    #[tokio::test]
    async fn test_channel_closed_error() {
        let (tx, rx) = mpsc::channel::<(String, i32)>(1);
        let debouncer = Debouncer::new(Duration::from_millis(50), tx);

        // Drop the receiver
        drop(rx);

        // Give the background task time to detect the closed channel
        sleep(Duration::from_millis(100)).await;

        // Send should still work (it goes to the input channel)
        // But we need to wait for the background task to try emitting
        let _ = debouncer.send("key".to_string(), 42).await;

        // The debouncer should handle the closed output channel gracefully
        // by logging a warning and continuing
    }

    #[tokio::test]
    async fn test_sequential_events_with_delay() {
        let (debouncer, mut rx) = test_debouncer::<String, i32>(50);

        // Send first event
        debouncer.send("key".to_string(), 1).await.unwrap();

        // Wait for it to be emitted
        let result1 = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().unwrap().1, 1);

        // Send second event after first is emitted
        debouncer.send("key".to_string(), 2).await.unwrap();

        // Should also be emitted
        let result2 = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().unwrap().1, 2);
    }
}
