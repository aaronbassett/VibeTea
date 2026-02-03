# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-03 (Phase 4: Supabase persistence)

## Test Framework

### Rust/Server and Monitor (Updated for Phase 4)

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)] mod tests` inline | In use |
| Integration | Rust built-in + wiremock | `tests/` directory with MockServer | In use (Phase 4) |
| E2E | Not selected | TBD | Not started |

### Running Tests

#### Rust/Server and Monitor (Phase 4)

| Command | Purpose |
|---------|---------|
| `cargo test` | Run all tests in the workspace |
| `cargo test --lib` | Run library unit tests only |
| `cargo test --test '*'` | Run integration tests only |
| `cargo test -- --nocapture` | Run tests with println output |
| `cargo test -- --test-threads=1` | Run tests sequentially (prevents env var interference) |
| `cargo test -p vibetea-monitor persistence` | Run persistence module tests |
| `cargo test --test sender_recovery_test` | Run sender recovery integration tests |
| `cargo test -p vibetea-server unsafe_mode` | Run unsafe mode integration tests |

## Test Organization

### Rust/Monitor Directory Structure (Phase 4 Update)

```
monitor/
├── src/
│   ├── config.rs               # Config module with inline tests
│   ├── error.rs                # Error module with inline tests
│   ├── types.rs                # Types module with inline tests
│   ├── watcher.rs              # File watching implementation
│   ├── parser.rs               # JSONL parser implementation
│   ├── privacy.rs              # Privacy pipeline with 38 inline unit tests
│   ├── crypto.rs               # Ed25519 crypto with 14 inline unit tests (Phase 6)
│   ├── sender.rs               # HTTP sender with 8 inline unit tests (Phase 6)
│   ├── persistence.rs          # Event batching and persistence (Phase 4) NEW
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint (CLI)
└── tests/
    ├── privacy_test.rs         # Integration tests for privacy compliance (17 tests)
    ├── sender_recovery_test.rs # Sender retry logic with wiremock (Phase 4) NEW
    └── (more integration tests to be created)
```

### Rust/Server Directory Structure

```
server/
├── src/
│   ├── config.rs               # Config module with inline tests (12 tests)
│   ├── error.rs                # Error module with inline tests (18+ tests)
│   ├── types.rs                # Types module with inline tests (10+ tests)
│   ├── routes.rs               # HTTP routes implementation
│   ├── auth.rs                 # Ed25519 signature verification
│   ├── broadcast.rs            # Event broadcasting
│   ├── rate_limit.rs           # Rate limiting logic
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint
└── tests/
    └── unsafe_mode_test.rs     # Integration test for unsafe auth mode
```

## Test Patterns

### Integration Tests (Rust) - Persistence Module (Phase 4)

Larger tests using wiremock to test event batching and retry logic:

#### Example from `monitor/tests/sender_recovery_test.rs`

```rust
//! Integration tests for sender recovery behavior.
//!
//! These tests verify that the sender correctly handles error scenarios
//! and recovers gracefully, particularly around oversized events.

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a test event with a small payload.
fn create_small_event() -> Event {
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

/// Creates an oversized event that exceeds MAX_CHUNK_SIZE (900KB).
fn create_oversized_event() -> Event {
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

/// Creates a sender configured to use the mock server with fast retry policy.
fn create_test_sender(server_url: &str) -> Sender {
    let config = SenderConfig::new(server_url.to_string(), "test-monitor".to_string(), 100)
        .with_retry_policy(RetryPolicy::fast_for_tests());
    Sender::new(config, Crypto::generate())
}

// =============================================================================
// Recovery Tests
// =============================================================================

/// Verifies that normal events are sent successfully after an oversized event
/// receives a 413 response.
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

/// Verifies that multiple oversized events don't accumulate and block the sender.
#[tokio::test]
async fn test_multiple_oversized_events_all_skipped() {
    let mock_server = MockServer::start().await;

    // All oversized chunks get 413
    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(413).set_body_string("Payload too large"))
        .mount(&mock_server)
        .await;

    let mut sender = create_test_sender(&mock_server.uri());

    // Queue only oversized events
    sender.queue(create_oversized_event());
    sender.queue(create_oversized_event());
    sender.queue(create_oversized_event());

    // Flush should succeed - all 413s are handled gracefully
    let result = sender.flush().await;
    assert!(result.is_ok(), "Flush should succeed: {:?}", result);
    assert!(sender.is_empty(), "Buffer should be empty after flush");
}

/// Verifies normal operation when no oversized events are present.
#[tokio::test]
async fn test_normal_events_flush_successfully() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut sender = create_test_sender(&mock_server.uri());

    // Queue only normal events
    for _ in 0..10 {
        sender.queue(create_small_event());
    }

    let result = sender.flush().await;
    assert!(result.is_ok(), "Flush should succeed: {:?}", result);
    assert!(sender.is_empty(), "Buffer should be empty after flush");
}

/// Verifies that server errors (5xx) still trigger retry/error behavior.
/// Only 413 is treated as "skip and continue".
#[tokio::test]
async fn test_server_error_still_fails_flush() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
        .mount(&mock_server)
        .await;

    let mut sender = create_test_sender(&mock_server.uri());
    sender.queue(create_small_event());

    // 500 errors should still cause flush to fail (after retries)
    let result = sender.flush().await;
    assert!(result.is_err(), "Flush should fail on 500 error");
}
```

Key patterns for Phase 4 integration tests:
1. **Helper functions**: Reusable test event creation
2. **wiremock MockServer**: Lightweight HTTP mocking without external services
3. **Async tests with #[tokio::test]**: Test async code patterns
4. **Response templates**: Match requests and return configured responses
5. **Scenario-based tests**: Test recovery paths (413, 500, timeout)
6. **Buffer verification**: Check buffer state after operations

### Unit Tests (Rust) - Persistence Module (Phase 4)

Persistence module will include inline tests for:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size_limit() {
        let mut batcher = EventBatcher::new(test_config(), Crypto::generate());

        // Add events up to MAX_BATCH_SIZE
        for _ in 0..MAX_BATCH_SIZE {
            let event = create_test_event();
            let needs_flush = batcher.queue(event);
            if _ == MAX_BATCH_SIZE - 1 {
                assert!(needs_flush, "Should signal flush at capacity");
            } else {
                assert!(!needs_flush, "Should not signal flush before capacity");
            }
        }

        assert_eq!(batcher.buffer_len(), MAX_BATCH_SIZE);
    }

    #[test]
    fn test_buffer_overflow_evicts_oldest() {
        let mut batcher = EventBatcher::new(test_config(), Crypto::generate());

        // Fill to capacity
        for i in 0..MAX_BATCH_SIZE {
            let mut event = create_test_event();
            event.id = format!("evt_{:020}", i);
            batcher.queue(event);
        }

        // Add one more - oldest should be evicted
        let mut event = create_test_event();
        event.id = "evt_00000000000000000001".to_string();
        batcher.queue(event);

        assert_eq!(batcher.buffer_len(), MAX_BATCH_SIZE);
        // Verify oldest event was removed
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::default();
        let delay1 = calculate_retry_delay(1, &policy);
        let delay2 = calculate_retry_delay(2, &policy);
        let delay3 = calculate_retry_delay(3, &policy);

        // Each should be roughly 2x the previous
        assert!(delay2.as_millis() > delay1.as_millis());
        assert!(delay3.as_millis() > delay2.as_millis());
    }

    #[test]
    fn test_jitter_within_bounds() {
        let policy = RetryPolicy {
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            max_attempts: 3,
            jitter_factor: 0.25,
        };

        for _ in 0..100 {
            let delay = calculate_retry_delay(1, &policy);
            let millis = delay.as_millis() as u64;
            // Should be 1000 ±25% = 750-1250
            assert!(millis >= 750 && millis <= 1250, "Jitter out of bounds: {}", millis);
        }
    }

    #[test]
    fn test_retry_policy_validation() {
        // Test clamping of invalid values
        let invalid = RetryPolicy {
            initial_delay_ms: 0,
            max_delay_ms: -1000, // Invalid
            max_attempts: 0,
            jitter_factor: 2.5,   // Invalid (>1.0)
        };

        let validated = invalid.validated();
        assert!(validated.initial_delay_ms >= 1);
        assert!(validated.max_delay_ms >= validated.initial_delay_ms);
        assert!(validated.max_attempts >= 1);
        assert!(validated.jitter_factor >= 0.0 && validated.jitter_factor <= 1.0);
    }
}
```

Key unit test patterns:
1. **Buffer management**: Size limits, eviction, state tracking
2. **Retry policy**: Exponential backoff, jitter bounds, validation
3. **Edge cases**: Empty buffers, max values, negative inputs
4. **Deterministic tests**: Use `RetryPolicy::fast_for_tests()` for predictable timing

## Mocking Strategy (Phase 4)

### Rust/Monitor - Persistence Tests

Persistence tests use **wiremock** for HTTP mocking:

```rust
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, body};

// Setup mock server
let mock_server = MockServer::start().await;

// Match POST /events and respond with 200
Mock::given(method("POST"))
    .and(path("/events"))
    .respond_with(ResponseTemplate::new(200))
    .mount(&mock_server)
    .await;

// Use mock server URL in tests
let sender = create_test_sender(&mock_server.uri());
```

Key advantages:
1. **No external services**: Mock server runs in-process
2. **Request matching**: Match on method, path, headers, body
3. **Multiple responses**: Mount multiple mocks with `.up_to_n_times()`
4. **Status codes**: Easy to test error conditions (413, 500)

## Coverage Requirements (Phase 4 Update)

### Targets

| Metric | Target | Status |
|--------|--------|--------|
| Line coverage | 80% | Phase 4: Increasing with persistence tests |
| Branch coverage | 75% | Phase 4: New retry logic branches |
| Function coverage | 80% | Phase 4: EventBatcher, PersistenceError |

### New Coverage Areas (Phase 4)

- `persistence.rs`: EventBatcher queue, flush, retry logic
- `sender.rs` retry paths: Success, timeout, 413, 500 responses
- `RetryPolicy` validation: Jitter clamping, delay exponential growth
- Chunk splitting: MAX_CHUNK_SIZE boundary handling

## Test Categories (Phase 4 Update)

### Persistence Tests (New for Phase 4)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `test_oversized_event_does_not_block_normal_events` | 413 handling | wiremock | Phase 4 |
| `test_multiple_oversized_events_all_skipped` | Batch cleanup | wiremock | Phase 4 |
| `test_normal_events_flush_successfully` | Happy path | wiremock | Phase 4 |
| `test_server_error_still_fails_flush` | 5xx handling | wiremock | Phase 4 |
| `test_batch_size_limit` | MAX_BATCH_SIZE | unit | Phase 4 |
| `test_buffer_overflow_evicts_oldest` | FIFO eviction | unit | Phase 4 |
| `test_retry_policy_exponential_backoff` | Backoff math | unit | Phase 4 |
| `test_jitter_within_bounds` | Jitter validation | unit | Phase 4 |

### Smoke Tests (Critical Path - Phase 4 Update)

Tests that must pass before any deploy:

| Test | Purpose | Location |
|------|---------|----------|
| Config tests | Configuration loading | `server/src/config.rs`, `monitor/src/config.rs` |
| Error tests | Error type safety | `server/src/error.rs` |
| Privacy tests | Constitution I compliance | `monitor/src/privacy.rs` + `privacy_test.rs` |
| Crypto tests | Ed25519 operations | `monitor/src/crypto.rs` |
| Sender tests | Event buffering | `monitor/src/sender.rs` + `sender_recovery_test.rs` |
| **Persistence tests** | **Batch/retry logic** | **`monitor/src/persistence.rs` + `sender_recovery_test.rs`** |

---

*This document describes HOW to test for Phase 4 (Supabase persistence). Update when testing strategy changes.*
