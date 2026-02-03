//! Integration tests for VIBETEA_UNSAFE_NO_AUTH=true mode.
//!
//! These tests verify that when unsafe mode is enabled:
//! - POST /events accepts requests without X-Signature header
//! - GET /ws accepts connections without token parameter
//! - Invalid signatures and tokens are ignored (not validated)
//!
//! # Warning
//!
//! Unsafe mode should NEVER be used in production. It completely disables
//! authentication and is intended only for local development and testing.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::time::timeout;
use tower::ServiceExt;
use uuid::Uuid;

use vibetea_server::config::Config;
use vibetea_server::routes::{create_router, AppState};
use vibetea_server::types::{Event, EventPayload, EventType, SessionAction};

// ============================================================================
// Test Helpers
// ============================================================================

/// Creates a configuration with unsafe_no_auth enabled.
fn unsafe_config() -> Config {
    Config {
        public_keys: HashMap::new(),
        subscriber_token: None,
        port: 0, // Will be overridden when binding
        unsafe_no_auth: true,
    }
}

/// Creates a test event for use in POST /events requests.
fn create_test_event(source: &str) -> Event {
    Event {
        id: format!("evt_test{}", Uuid::new_v4().simple()),
        source: source.to_string(),
        timestamp: Utc::now(),
        event_type: EventType::Session,
        payload: EventPayload::Session {
            session_id: Uuid::new_v4(),
            action: SessionAction::Started,
            project: "test-project".to_string(),
        },
    }
}

/// Spawns a test server on a random available port.
/// Returns the socket address and a handle to abort the server.
async fn spawn_test_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    // Bind to port 0 to get a random available port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    (addr, handle)
}

// ============================================================================
// POST /events Tests - Without Signature
// ============================================================================

/// Test that POST /events accepts requests without X-Signature header
/// when VIBETEA_UNSAFE_NO_AUTH=true.
#[tokio::test]
async fn post_events_without_signature_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                // No X-Signature header
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "POST /events should accept requests without X-Signature in unsafe mode"
    );
}

/// Test that POST /events accepts batch events without signature in unsafe mode.
#[tokio::test]
async fn post_events_batch_without_signature_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let events = vec![
        create_test_event("test-monitor"),
        create_test_event("test-monitor"),
        create_test_event("test-monitor"),
    ];
    let body = serde_json::to_string(&events).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "POST /events should accept batch events without signature in unsafe mode"
    );
}

// ============================================================================
// POST /events Tests - Invalid Signature Ignored
// ============================================================================

/// Test that invalid X-Signature is ignored in unsafe mode.
/// The server should accept the request even with a completely invalid signature.
#[tokio::test]
async fn post_events_with_invalid_signature_ignored_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .header("X-Signature", "completely-invalid-not-even-base64!!!")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "POST /events should ignore invalid X-Signature in unsafe mode"
    );
}

/// Test that a malformed base64 signature is ignored in unsafe mode.
#[tokio::test]
async fn post_events_with_malformed_base64_signature_ignored_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .header("X-Signature", "not===valid===base64")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "POST /events should ignore malformed base64 signature in unsafe mode"
    );
}

/// Test that a valid-looking but wrong signature is ignored in unsafe mode.
#[tokio::test]
async fn post_events_with_wrong_signature_ignored_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let body = serde_json::to_string(&event).unwrap();

    // Valid base64, valid length for Ed25519 signature, but wrong signature
    let fake_signature = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        [0u8; 64], // Ed25519 signatures are 64 bytes
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .header("X-Signature", &fake_signature)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "POST /events should ignore wrong signature in unsafe mode"
    );
}

// ============================================================================
// POST /events - Source ID Still Required
// ============================================================================

/// Test that X-Source-ID header is still required even in unsafe mode.
/// This ensures events can be properly attributed to their source.
#[tokio::test]
async fn post_events_still_requires_source_id_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                // No X-Source-ID header
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "POST /events should still require X-Source-ID even in unsafe mode"
    );
}

/// Test that an empty X-Source-ID is rejected even in unsafe mode.
#[tokio::test]
async fn post_events_rejects_empty_source_id_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "POST /events should reject empty X-Source-ID even in unsafe mode"
    );
}

// ============================================================================
// POST /events - Event Validation Still Works
// ============================================================================

/// Test that invalid JSON is still rejected in unsafe mode.
#[tokio::test]
async fn post_events_still_validates_json_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .body(Body::from("this is not valid json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "POST /events should still validate JSON in unsafe mode"
    );
}

/// Test that invalid event structure is rejected in unsafe mode.
#[tokio::test]
async fn post_events_still_validates_event_structure_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    // Valid JSON but missing required fields
    let invalid_event = json!({
        "id": "evt_test123",
        // missing source, timestamp, type, payload
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .body(Body::from(invalid_event.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "POST /events should still validate event structure in unsafe mode"
    );
}

// ============================================================================
// GET /ws Tests - Without Token
// ============================================================================

/// Test that GET /ws initiates WebSocket upgrade without token in unsafe mode.
///
/// Note: This test verifies the HTTP upgrade response. Full WebSocket
/// communication testing would require a WebSocket client library.
#[tokio::test]
async fn get_ws_upgrade_without_token_in_unsafe_mode() {
    let (addr, handle) = spawn_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ws", addr))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();

    // WebSocket upgrade should succeed (101) or be accepted for upgrade
    // Note: reqwest doesn't fully support WebSocket, so we might get different
    // status codes depending on how the connection is handled
    let status = response.status();
    assert!(
        status == StatusCode::SWITCHING_PROTOCOLS || status.is_success(),
        "GET /ws should accept WebSocket upgrade without token in unsafe mode, got {}",
        status
    );

    handle.abort();
}

// ============================================================================
// GET /ws Tests - Invalid Token Ignored
// ============================================================================

/// Test that GET /ws with invalid token still upgrades in unsafe mode.
#[tokio::test]
async fn get_ws_with_invalid_token_ignored_in_unsafe_mode() {
    let (addr, handle) = spawn_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ws?token=completely-invalid-token", addr))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();

    let status = response.status();
    assert!(
        status == StatusCode::SWITCHING_PROTOCOLS || status.is_success(),
        "GET /ws should ignore invalid token in unsafe mode, got {}",
        status
    );

    handle.abort();
}

/// Test that GET /ws with empty token still upgrades in unsafe mode.
#[tokio::test]
async fn get_ws_with_empty_token_ignored_in_unsafe_mode() {
    let (addr, handle) = spawn_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}/ws?token=", addr))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();

    let status = response.status();
    assert!(
        status == StatusCode::SWITCHING_PROTOCOLS || status.is_success(),
        "GET /ws should ignore empty token in unsafe mode, got {}",
        status
    );

    handle.abort();
}

// ============================================================================
// GET /ws Tests - Filters Still Work
// ============================================================================

/// Test that WebSocket filter parameters still work in unsafe mode.
#[tokio::test]
async fn get_ws_with_filters_works_in_unsafe_mode() {
    let (addr, handle) = spawn_test_server().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}/ws?source=my-monitor&type=session&project=my-project",
            addr
        ))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .send()
        .await
        .unwrap();

    let status = response.status();
    assert!(
        status == StatusCode::SWITCHING_PROTOCOLS || status.is_success(),
        "GET /ws with filters should work in unsafe mode, got {}",
        status
    );

    handle.abort();
}

// ============================================================================
// Health Endpoint - No Auth Required Regardless of Mode
// ============================================================================

/// Test that /health endpoint works in unsafe mode (should always work).
#[tokio::test]
async fn health_endpoint_works_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /health should work in unsafe mode"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let health: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(health["status"], "ok");
}

// ============================================================================
// Event Broadcasting Works in Unsafe Mode
// ============================================================================

/// Test that events are properly broadcast in unsafe mode.
#[tokio::test]
async fn events_are_broadcast_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);

    // Subscribe to events before sending
    let mut receiver = state.broadcaster.subscribe();

    let app = create_router(state);

    let event = create_test_event("test-monitor");
    let event_id = event.id.clone();
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Verify event was broadcast
    let received_event = timeout(Duration::from_millis(100), async {
        receiver.recv().await.unwrap()
    })
    .await
    .expect("Should receive broadcast event");

    assert_eq!(
        received_event.id, event_id,
        "Broadcast event should match sent event"
    );
}

/// Test that multiple events are broadcast in order in unsafe mode.
#[tokio::test]
async fn multiple_events_broadcast_in_order_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);

    let mut receiver = state.broadcaster.subscribe();

    let app = create_router(state);

    let events: Vec<Event> = (0..3)
        .map(|i| {
            let mut event = create_test_event("test-monitor");
            event.id = format!("evt_order_test_{:03}", i);
            event
        })
        .collect();

    let event_ids: Vec<String> = events.iter().map(|e| e.id.clone()).collect();
    let body = serde_json::to_string(&events).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "test-monitor")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    // Verify all events were broadcast in order
    for expected_id in event_ids {
        let received = timeout(Duration::from_millis(100), async {
            receiver.recv().await.unwrap()
        })
        .await
        .expect("Should receive all broadcast events");

        assert_eq!(
            received.id, expected_id,
            "Events should be broadcast in order"
        );
    }
}

// ============================================================================
// Any Source ID Accepted in Unsafe Mode
// ============================================================================

/// Test that any source ID is accepted in unsafe mode (no public key required).
#[tokio::test]
async fn any_source_id_accepted_in_unsafe_mode() {
    let config = unsafe_config();
    let state = AppState::new(config);
    let app = create_router(state);

    // Use a source ID that would normally require a registered public key
    let event = create_test_event("unregistered-monitor-xyz");
    let body = serde_json::to_string(&event).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "unregistered-monitor-xyz")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "Any source ID should be accepted in unsafe mode (no public key lookup)"
    );
}

// ============================================================================
// Rate Limiting Still Works in Unsafe Mode
// ============================================================================

/// Test that rate limiting still applies in unsafe mode.
#[tokio::test]
async fn rate_limiting_still_works_in_unsafe_mode() {
    use vibetea_server::broadcast::EventBroadcaster;
    use vibetea_server::rate_limit::RateLimiter;

    let config = unsafe_config();
    let state = AppState::with_components(
        config,
        EventBroadcaster::new(),
        RateLimiter::new(1.0, 1), // Very restrictive: 1 request allowed
    );
    let app = create_router(state);

    let event = create_test_event("rate-test-source");
    let body = serde_json::to_string(&event).unwrap();

    // First request should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "rate-test-source")
                .body(Body::from(body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::ACCEPTED,
        "First request should succeed"
    );

    // Second immediate request should be rate limited
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("Content-Type", "application/json")
                .header("X-Source-ID", "rate-test-source")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Rate limiting should still apply in unsafe mode"
    );
}
