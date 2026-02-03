//! Integration tests for sender recovery behavior.
//!
//! These tests verify that the sender correctly handles error scenarios
//! and recovers gracefully, particularly around oversized events.

use uuid::Uuid;
use vibetea_monitor::crypto::Crypto;
use vibetea_monitor::sender::{RetryPolicy, Sender, SenderConfig};
use vibetea_monitor::types::{Event, EventPayload, EventType, ToolStatus};
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
///
/// This test ensures one oversized event doesn't block the entire sender buffer.
#[tokio::test]
async fn test_oversized_event_does_not_block_normal_events() {
    let mock_server = MockServer::start().await;

    // Track how many successful requests we receive
    // First request: oversized chunk -> 413
    // Second request: normal chunk -> 200
    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(413).set_body_string("Payload too large"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut sender = create_test_sender(&mock_server.uri());

    // Queue events: [normal, oversized, normal]
    // The oversized event should be isolated in its own chunk
    sender.queue(create_small_event());
    sender.queue(create_oversized_event());
    sender.queue(create_small_event());

    // Flush should succeed overall - the 413 for the oversized chunk is handled
    // gracefully and doesn't prevent the normal events from being sent
    let result = sender.flush().await;
    assert!(result.is_ok(), "Flush should succeed: {:?}", result);

    // Verify buffer is cleared
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

    // Verify buffer is cleared even though all events were rejected
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

/// Verifies that server errors (5xx) still trigger retry/error behavior,
/// only 413 is treated as "skip and continue".
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
