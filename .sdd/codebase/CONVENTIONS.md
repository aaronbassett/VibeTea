# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-03 (Phase 4: Supabase persistence)

## Code Style

### Formatting Tools

| Tool | Configuration | Command |
|------|---------------|---------|
| Prettier (TypeScript/Client) | `.prettierrc` | `npm run format` |
| ESLint (TypeScript/Client) | `eslint.config.js` | `npm run lint` |
| rustfmt (Rust/Server/Monitor) | Default settings | `cargo fmt` |
| clippy (Rust/Server/Monitor) | Default lints | `cargo clippy` |
| Deno fmt (Supabase Edge Functions) | Default settings | `deno fmt` |

### Style Rules

#### Rust/Server/Monitor (Phase 4 focus)

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 4 spaces | rustfmt default |
| Strings | Double quotes | `"string"` |
| Line length | 100 chars (soft) | rustfmt respects natural breaks |
| Comments | `//! ` for module docs, `///` for item docs | Doc comments on all public items |
| Module docs | Every module with overview and examples | `//! Event persistence with exponential backoff` |

## Naming Conventions

### Rust/Server/Monitor (Phase 4 focus)

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `persistence.rs`, `config.rs` |
| Module constants | SCREAMING_SNAKE_CASE | `MAX_BATCH_SIZE`, `INITIAL_RETRY_DELAY_MS` |
| Tests | Inline `#[cfg(test)] mod tests` | `sender_recovery_test.rs`, `privacy_test.rs` |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `EventBatcher`, `PersistenceConfig`, `RetryPolicy` |
| Enums | PascalCase | `PersistenceError`, `EventType`, `ToolStatus` |
| Error variants | PascalCase | `AuthFailed`, `MaxRetriesExceeded`, `ServerError` |
| Functions | snake_case | `new()`, `queue()`, `flush()`, `validated()` |
| Methods | snake_case | `.with_retry_policy()`, `.is_extension_allowed()` |
| Test functions | `test_` prefix | `test_oversized_event_does_not_block()` |

## Error Handling

### Rust/Monitor - Persistence Module (Phase 4)

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Persistence errors | `#[derive(Error)]` enum with variants | `monitor/src/persistence.rs` |
| Max retries exceeded | Struct variant with field | `MaxRetriesExceeded { attempts: u8 }` |
| Server errors | Struct variant with status/message | `ServerError { status: u16, message: String }` |
| Auth failures | Simple variant | `AuthFailed` |
| HTTP errors | Automatic conversion | `#[from] reqwest::Error` |
| JSON errors | Automatic conversion | `#[from] serde_json::Error` |

Example from `monitor/src/persistence.rs`:

```rust
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
```

### Logging Conventions

Structured logging using the `tracing` crate:

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Failures that affect operation | `error!("Batch submission failed after {attempts} retries", attempts)` |
| warn | Recoverable issues or unsafe modes | `warn!("Persistence buffer overflow, dropping oldest event")` |
| info | State changes and milestones | `info!("Batch of {count} events submitted successfully", count)` |
| debug | Diagnostic information | `debug!("Retry policy configured: initial={ms}ms, max={max}ms", ms, max)` |

## Common Patterns (Phase 4 Focus)

### Event Batching Pattern

Batch events efficiently for persistence:

```rust
/// Buffers events in memory until batch interval or size limit.
pub struct EventBatcher {
    config: PersistenceConfig,
    crypto: Crypto,
    buffer: Vec<Event>,
    client: Client,
    consecutive_failures: u8,
}

impl EventBatcher {
    /// Adds an event to the buffer.
    /// Returns true if buffer is at capacity and should be flushed.
    pub fn queue(&mut self, event: Event) -> bool {
        if self.buffer.len() >= MAX_BATCH_SIZE {
            warn!("Persistence buffer overflow, dropping oldest event");
            self.buffer.remove(0);
        }
        self.buffer.push(event);
        self.buffer.len() >= MAX_BATCH_SIZE
    }

    /// Sends buffered events to the edge function.
    /// Uses exponential backoff with jitter on failure.
    pub async fn flush(&mut self) -> Result<usize, PersistenceError> {
        // Implementation with retry logic
    }
}
```

### Retry Policy Pattern

Configure exponential backoff with validation:

```rust
/// Retry configuration with exponential backoff.
pub struct RetryPolicy {
    pub initial_delay_ms: u64,      // 1000
    pub max_delay_ms: u64,          // 60000
    pub max_attempts: u32,          // Limit before giving up
    pub jitter_factor: f64,         // ±25% variance
}

impl RetryPolicy {
    /// Creates a retry policy optimized for fast tests.
    pub fn fast_for_tests() -> Self {
        Self {
            initial_delay_ms: 1,
            max_delay_ms: 5,
            max_attempts: 3,
            jitter_factor: 0.0,  // No jitter for deterministic tests
        }
    }

    /// Validates and clamps values to acceptable ranges.
    pub fn validated(mut self) -> Self {
        // Clamp jitter factor to 0.0..=1.0
        self.jitter_factor = if self.jitter_factor.is_finite() {
            self.jitter_factor.clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Ensure positive delays
        self.initial_delay_ms = self.initial_delay_ms.max(1);
        self.max_delay_ms = self.max_delay_ms.max(self.initial_delay_ms);

        // Ensure at least one attempt
        self.max_attempts = self.max_attempts.max(1);

        self
    }
}
```

### Async Test Helpers Pattern

Helper functions for test setup and teardown:

```rust
// sender_recovery_test.rs
fn create_test_event() -> Event {
    Event::new(
        "test-monitor".to_string(),
        EventType::Tool,
        EventPayload::Tool {
            session_id: Uuid::new_v4(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("small.rs".to_string()),
            project: Some("test-project".to_string()),
        },
    )
}

fn create_oversized_event() -> Event {
    // Create context larger than 900KB to trigger oversized handling
    let oversized_context = "x".repeat(950_000);
    Event::new(
        "test-monitor".to_string(),
        EventType::Tool,
        EventPayload::Tool {
            session_id: Uuid::new_v4(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some(oversized_context),
            project: Some("test-project".to_string()),
        },
    )
}

fn create_test_sender(server_url: &str) -> Sender {
    let config = SenderConfig::new(server_url.to_string(), "test-monitor".to_string(), 100)
        .with_retry_policy(RetryPolicy::fast_for_tests());
    Sender::new(config, Crypto::generate())
}

// Integration test using wiremock
#[tokio::test]
async fn test_oversized_event_does_not_block_normal_events() {
    let mock_server = MockServer::start().await;

    // First request: oversized chunk -> 413
    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(413).set_body_string("Payload too large"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second request: normal chunk -> 200
    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut sender = create_test_sender(&mock_server.uri());

    // Queue events: [normal, oversized, normal]
    sender.queue(create_small_event());
    sender.queue(create_oversized_event());
    sender.queue(create_small_event());

    // Flush should succeed overall
    let result = sender.flush().await;
    assert!(result.is_ok(), "Flush should succeed: {:?}", result);
    assert!(sender.is_empty(), "Buffer should be empty after flush");
}
```

Key patterns for Phase 4:
1. **Test helpers**: Extract common setup into reusable functions
2. **Mock servers**: Use wiremock for HTTP testing without external dependencies
3. **Error responses**: Test both success (200) and error (413, 500) cases
4. **Buffer state**: Verify buffer is empty after flush
5. **Event composition**: Use builder pattern for complex test data

### Chunked Sending Pattern (Phase 4)

Split large payloads into multiple requests:

```rust
const MAX_CHUNK_SIZE: usize = 900 * 1024;  // 900KB per request

/// Splits events into chunks that fit within MAX_CHUNK_SIZE.
fn chunk_events(events: Vec<Event>) -> Vec<Vec<Event>> {
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    let mut current_size = 0;

    for event in events {
        let event_size = serde_json::to_string(&event)
            .map(|s| s.len())
            .unwrap_or(1000);

        if !current_chunk.is_empty() && current_size + event_size > MAX_CHUNK_SIZE {
            chunks.push(current_chunk);
            current_chunk = Vec::new();
            current_size = 0;
        }

        current_chunk.push(event);
        current_size += event_size;
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}
```

### Exponential Backoff with Jitter Pattern

Implement backoff for retry logic:

```rust
/// Calculate next retry delay with exponential backoff and jitter.
fn calculate_retry_delay(attempt: u32, policy: &RetryPolicy) -> Duration {
    // Exponential: initial_delay * 2^(attempt-1)
    let base_delay_ms = policy.initial_delay_ms
        .saturating_mul(2u64.pow(attempt.saturating_sub(1)))
        .min(policy.max_delay_ms);

    // Add jitter: ±(jitter_factor * base_delay)
    if policy.jitter_factor == 0.0 {
        Duration::from_millis(base_delay_ms)
    } else {
        let jitter_range = (base_delay_ms as f64 * policy.jitter_factor) as i64;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_range..=jitter_range);
        let final_delay_ms = (base_delay_ms as i64 + jitter).max(1) as u64;
        Duration::from_millis(final_delay_ms)
    }
}

// Usage in sender
async fn flush_with_retry(&mut self) -> Result<usize> {
    for attempt in 1..=self.config.retry_policy.max_attempts {
        match self.send_batch().await {
            Ok(count) => {
                self.consecutive_failures = 0;
                return Ok(count);
            }
            Err(e) if attempt >= self.config.retry_policy.max_attempts => {
                return Err(e);
            }
            Err(_) => {
                let delay = calculate_retry_delay(attempt, &self.config.retry_policy);
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

## Module Documentation Standard (Phase 4)

Every module includes detailed documentation:

```rust
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
//!     // ... use batcher
//!     Ok(())
//! }
//! ```
```

Key module documentation patterns:
1. **First line**: One-line summary
2. **Design section**: High-level architecture and decision rationale
3. **Example code**: Practical usage patterns with `no_run` for external examples
4. **Key concepts**: Links to important types and constants using `[`Type`]`

## Git Conventions

### Commit Messages (Phase 4)

Format: `type(scope): description`

Phase 4 examples:
- `feat(monitor): implement batch interval timer`
- `feat(monitor): implement retry logic with exponential backoff`
- `feat(monitor): implement signed batch submission`
- `feat(monitor): implement event buffering with max 1000 events`
- `feat(monitor): scaffold persistence module`
- `feat(monitor): add persistence configuration`
- `test(monitor): add persistence integration tests with wiremock`

---

*This document defines HOW to write code for Phase 4 (Supabase persistence). Update when conventions change.*
