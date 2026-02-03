# Contract: Sender Metrics Interface

**Module**: `monitor/src/sender.rs` (modification)
**Date**: 2026-02-03

## Overview

Extension to the existing Sender module to expose observable metrics for TUI consumption (FR-025).

## Interface

### SenderMetrics Struct

```rust
/// Observable metrics from the sender (FR-025)
#[derive(Debug, Clone, Copy, Default)]
pub struct SenderMetrics {
    /// Events currently queued in the send buffer
    pub queued: usize,
    /// Total events successfully sent to server
    pub sent: u64,
    /// Total events that failed to send
    pub failed: u64,
}

impl SenderMetrics {
    /// Total events processed (sent + failed)
    pub fn total(&self) -> u64 {
        self.sent + self.failed
    }
}
```

### Sender Extension

Add to existing `Sender` struct:

```rust
impl Sender {
    /// Get current metrics snapshot
    ///
    /// Returns a point-in-time snapshot of sender statistics.
    /// This method is cheap to call and does not block.
    pub fn metrics(&self) -> SenderMetrics {
        SenderMetrics {
            queued: self.buffer.len(),
            sent: self.sent_count,
            failed: self.failed_count,
        }
    }

    /// Subscribe to metrics updates
    ///
    /// Returns a channel receiver that emits metrics updates
    /// after each send attempt (success or failure).
    pub fn subscribe_metrics(&self) -> mpsc::UnboundedReceiver<SenderMetrics> {
        // Implementation detail: clone the existing subscriber channel
        self.metrics_tx.subscribe()
    }
}
```

### Internal Changes

Add counters to Sender struct:

```rust
pub struct Sender {
    config: SenderConfig,
    crypto: Crypto,
    client: Client,
    buffer: VecDeque<Event>,
    current_retry_delay: Duration,
    // NEW: Metrics tracking
    sent_count: u64,
    failed_count: u64,
    metrics_tx: broadcast::Sender<SenderMetrics>,
}

impl Sender {
    pub fn new(config: SenderConfig, crypto: Crypto) -> Self {
        let (metrics_tx, _) = broadcast::channel(16);
        Self {
            config,
            crypto,
            client: Client::new(),
            buffer: VecDeque::with_capacity(config.buffer_size),
            current_retry_delay: Duration::from_secs(INITIAL_RETRY_DELAY_SECS),
            sent_count: 0,
            failed_count: 0,
            metrics_tx,
        }
    }

    pub async fn send(&mut self, event: Event) -> Result<(), SenderError> {
        match self.try_send(&event).await {
            Ok(()) => {
                self.sent_count += 1;
                self.notify_metrics();
                Ok(())
            }
            Err(e) => {
                self.failed_count += 1;
                self.notify_metrics();
                Err(e)
            }
        }
    }

    fn notify_metrics(&self) {
        // Best-effort notification, ignore if no subscribers
        let _ = self.metrics_tx.send(self.metrics());
    }
}
```

## Usage in TUI

### Polling Approach

```rust
// In TUI event loop
loop {
    tokio::select! {
        // ... other event handling

        // Poll metrics periodically or on demand
        _ = tick_interval.tick() => {
            let metrics = sender.metrics();
            app_state.update_stats(EventStats {
                total: metrics.total(),
                sent: metrics.sent,
                failed: metrics.failed,
                queued: metrics.queued,
            });
        }
    }
}
```

### Subscription Approach

```rust
// Alternative: subscribe to metrics channel
let mut metrics_rx = sender.subscribe_metrics();

loop {
    tokio::select! {
        // ... other event handling

        Ok(metrics) = metrics_rx.recv() => {
            app_state.update_stats(EventStats {
                total: metrics.total(),
                sent: metrics.sent,
                failed: metrics.failed,
                queued: metrics.queued,
            });
        }
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initial_state() {
        let sender = create_test_sender();
        let metrics = sender.metrics();

        assert_eq!(metrics.queued, 0);
        assert_eq!(metrics.sent, 0);
        assert_eq!(metrics.failed, 0);
        assert_eq!(metrics.total(), 0);
    }

    #[tokio::test]
    async fn test_metrics_after_queue() {
        let mut sender = create_test_sender();

        sender.queue(create_test_event());
        sender.queue(create_test_event());

        let metrics = sender.metrics();
        assert_eq!(metrics.queued, 2);
    }

    #[tokio::test]
    async fn test_metrics_subscription() {
        let mut sender = create_test_sender();
        let mut rx = sender.subscribe_metrics();

        // Simulate send (mock server response)
        sender.sent_count += 1;
        sender.notify_metrics();

        let metrics = rx.recv().await.unwrap();
        assert_eq!(metrics.sent, 1);
    }
}
```

## Backwards Compatibility

This is a non-breaking addition to the existing Sender API:
- New `metrics()` method added
- Optional `subscribe_metrics()` method added
- Internal struct fields added (private)
- No changes to existing public API
