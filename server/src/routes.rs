//! HTTP route handlers for the VibeTea server.
//!
//! This module provides the HTTP API endpoints:
//!
//! - `POST /events` - Ingest events from monitors
//! - `GET /ws` - WebSocket subscription endpoint for clients
//! - `GET /health` - Health check endpoint
//!
//! # Architecture
//!
//! All routes share application state through [`AppState`], which contains:
//! - Configuration (including auth settings)
//! - Event broadcaster for distributing events to WebSocket clients
//! - Rate limiter for protecting against abuse
//! - Server start time for uptime reporting
//!
//! # Example
//!
//! ```rust,no_run
//! use vibetea_server::routes::{create_router, AppState};
//! use vibetea_server::config::Config;
//! use vibetea_server::broadcast::EventBroadcaster;
//! use vibetea_server::rate_limit::RateLimiter;
//! use tokio::time::Instant;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config::from_env().expect("failed to load config");
//!     let state = AppState::new(config);
//!     let app = create_router(state);
//!
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Query, State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;
use tracing::{debug, error, info, trace, warn};

use crate::auth::{validate_token, verify_signature, AuthError};
use crate::broadcast::{EventBroadcaster, SubscriberFilter};
use crate::config::Config;
use crate::rate_limit::{RateLimitResult, RateLimiter};
use crate::types::{Event, EventType};

// ============================================================================
// Constants
// ============================================================================

/// Header name for the source identifier.
const HEADER_SOURCE_ID: &str = "X-Source-ID";

/// Header name for the Ed25519 signature.
const HEADER_SIGNATURE: &str = "X-Signature";

/// Header name for rate limit retry delay.
const HEADER_RETRY_AFTER: &str = "Retry-After";

/// Maximum body size for event ingestion (1 MB).
const MAX_BODY_SIZE: usize = 1024 * 1024;

// ============================================================================
// Application State
// ============================================================================

/// Shared application state for all route handlers.
///
/// This struct is wrapped in an `Arc` and cloned for each request handler,
/// enabling efficient shared access to server-wide resources.
#[derive(Clone)]
pub struct AppState {
    /// Server configuration.
    pub config: Arc<Config>,

    /// Event broadcaster for distributing events to WebSocket clients.
    pub broadcaster: EventBroadcaster,

    /// Rate limiter for protecting against abuse.
    pub rate_limiter: RateLimiter,

    /// Server start time for uptime calculation.
    pub start_time: Instant,
}

impl AppState {
    /// Creates a new application state with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration parsed from environment variables
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use vibetea_server::routes::AppState;
    /// use vibetea_server::config::Config;
    ///
    /// let config = Config::from_env().expect("failed to load config");
    /// let state = AppState::new(config);
    /// ```
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
            broadcaster: EventBroadcaster::new(),
            rate_limiter: RateLimiter::default(),
            start_time: Instant::now(),
        }
    }

    /// Creates application state with custom broadcaster and rate limiter.
    ///
    /// Useful for testing or when custom capacity/rate limits are needed.
    #[must_use]
    pub fn with_components(
        config: Config,
        broadcaster: EventBroadcaster,
        rate_limiter: RateLimiter,
    ) -> Self {
        Self {
            config: Arc::new(config),
            broadcaster,
            rate_limiter,
            start_time: Instant::now(),
        }
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &"<Config>")
            .field("broadcaster", &self.broadcaster)
            .field("rate_limiter", &self.rate_limiter)
            .field("start_time", &self.start_time)
            .finish()
    }
}

// ============================================================================
// Router
// ============================================================================

/// Creates the application router with all routes configured.
///
/// # Arguments
///
/// * `state` - Shared application state
///
/// # Returns
///
/// An axum `Router` with the following routes:
/// - `POST /events` - Event ingestion endpoint
/// - `GET /ws` - WebSocket subscription endpoint
/// - `GET /health` - Health check endpoint
///
/// # Example
///
/// ```rust,no_run
/// use vibetea_server::routes::{create_router, AppState};
/// use vibetea_server::config::Config;
///
/// let config = Config::from_env().expect("failed to load config");
/// let state = AppState::new(config);
/// let router = create_router(state);
/// ```
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/events", post(post_events))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .route("/ws", get(get_ws))
        .route("/health", get(get_health))
        .with_state(state)
}

// ============================================================================
// Error Response Types
// ============================================================================

/// JSON error response body.
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

impl ErrorResponse {
    fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: None,
        }
    }

    fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

// ============================================================================
// POST /events - Event Ingestion
// ============================================================================

/// Request body for event ingestion.
///
/// Accepts either a single event or an array of events.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EventsPayload {
    Single(Event),
    Batch(Vec<Event>),
}

impl EventsPayload {
    /// Converts the payload into a vector of events.
    fn into_events(self) -> Vec<Event> {
        match self {
            Self::Single(event) => vec![event],
            Self::Batch(events) => events,
        }
    }
}

/// POST /events - Ingest events from monitors.
///
/// # Authentication
///
/// Unless `unsafe_no_auth` is enabled, requests must include:
/// - `X-Source-ID` header: Monitor identifier
/// - `X-Signature` header: Ed25519 signature of the request body
///
/// # Rate Limiting
///
/// Requests are rate-limited per source. If the limit is exceeded,
/// returns 429 with a `Retry-After` header.
///
/// # Request Body
///
/// Accepts either a single event or an array of events as JSON.
///
/// # Responses
///
/// - `202 Accepted` - Events accepted and queued for broadcast
/// - `400 Bad Request` - Invalid event format
/// - `401 Unauthorized` - Authentication failed
/// - `429 Too Many Requests` - Rate limit exceeded
async fn post_events(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Response {
    // Extract required headers
    let source_id = match headers.get(HEADER_SOURCE_ID).and_then(|v| v.to_str().ok()) {
        Some(id) if !id.is_empty() => id,
        _ => {
            debug!("Missing or empty X-Source-ID header");
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("missing X-Source-ID header").with_code("missing_source")),
            )
                .into_response();
        }
    };

    // Authenticate if required
    if !state.config.unsafe_no_auth {
        let signature = match headers.get(HEADER_SIGNATURE).and_then(|v| v.to_str().ok()) {
            Some(sig) if !sig.is_empty() => sig,
            _ => {
                debug!(source = %source_id, "Missing or empty X-Signature header");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(
                        ErrorResponse::new("missing X-Signature header")
                            .with_code("missing_signature"),
                    ),
                )
                    .into_response();
            }
        };

        // Verify signature
        if let Err(err) = verify_signature(source_id, signature, &body, &state.config.public_keys) {
            warn!(source = %source_id, error = %err, "Signature verification failed");
            let (error_msg, error_code) = match err {
                AuthError::UnknownSource(_) => ("unknown source", "unknown_source"),
                AuthError::InvalidSignature => ("invalid signature", "invalid_signature"),
                AuthError::InvalidBase64(_) => ("invalid signature encoding", "invalid_encoding"),
                AuthError::InvalidPublicKey => ("server configuration error", "server_error"),
                AuthError::InvalidToken => ("invalid token", "invalid_token"),
            };
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new(error_msg).with_code(error_code)),
            )
                .into_response();
        }
    }

    // Check rate limit
    match state.rate_limiter.check_rate_limit(source_id).await {
        RateLimitResult::Allowed => {}
        RateLimitResult::Limited { retry_after_secs } => {
            info!(
                source = %source_id,
                retry_after = retry_after_secs,
                "Rate limit exceeded"
            );
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(HEADER_RETRY_AFTER, retry_after_secs.to_string())],
                Json(ErrorResponse::new("rate limit exceeded").with_code("rate_limited")),
            )
                .into_response();
        }
    }

    // Parse request body
    let events_payload: EventsPayload = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            debug!(source = %source_id, error = %err, "Failed to parse event payload");
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    ErrorResponse::new(format!("invalid event format: {err}"))
                        .with_code("invalid_format"),
                ),
            )
                .into_response();
        }
    };

    let events = events_payload.into_events();
    let event_count = events.len();

    // Validate that all events have matching source
    for event in &events {
        if event.source != source_id {
            warn!(
                authenticated_source = %source_id,
                event_source = %event.source,
                event_id = %event.id,
                "Event source mismatch"
            );
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    ErrorResponse::new("event source does not match authenticated source")
                        .with_code("source_mismatch"),
                ),
            )
                .into_response();
        }
    }

    // Broadcast events
    for event in events {
        trace!(
            source = %source_id,
            event_id = %event.id,
            event_type = ?event.event_type,
            "Broadcasting event"
        );
        state.broadcaster.broadcast(event);
    }

    info!(
        source = %source_id,
        event_count = event_count,
        "Events accepted and broadcast"
    );

    StatusCode::ACCEPTED.into_response()
}

// ============================================================================
// GET /ws - WebSocket Subscription
// ============================================================================

/// Query parameters for WebSocket subscription.
#[derive(Debug, Deserialize)]
pub struct WsQueryParams {
    /// Authentication token (required unless unsafe_no_auth is enabled).
    pub token: Option<String>,

    /// Filter events by source ID.
    pub source: Option<String>,

    /// Filter events by type.
    #[serde(rename = "type")]
    pub event_type: Option<EventType>,

    /// Filter events by project name.
    pub project: Option<String>,
}

impl WsQueryParams {
    /// Builds a `SubscriberFilter` from the query parameters.
    fn to_filter(&self) -> SubscriberFilter {
        let mut filter = SubscriberFilter::new();

        if let Some(ref source) = self.source {
            filter = filter.with_source(source.clone());
        }

        if let Some(event_type) = self.event_type {
            filter = filter.with_event_type(event_type);
        }

        if let Some(ref project) = self.project {
            filter = filter.with_project(project.clone());
        }

        filter
    }
}

/// GET /ws - WebSocket subscription endpoint.
///
/// # Authentication
///
/// Unless `unsafe_no_auth` is enabled, the `token` query parameter is required
/// and must match the configured subscriber token.
///
/// # Query Parameters
///
/// - `token` - Authentication token (required unless unsafe_no_auth)
/// - `source` - Filter events by source ID
/// - `type` - Filter events by type (session, activity, tool, agent, summary, error)
/// - `project` - Filter events by project name
///
/// # WebSocket Protocol
///
/// Once connected, the server sends JSON-encoded events as text messages.
/// Events are filtered according to the provided query parameters.
///
/// # Responses
///
/// - `101 Switching Protocols` - WebSocket upgrade successful
/// - `401 Unauthorized` - Invalid or missing token
async fn get_ws(
    State(state): State<AppState>,
    Query(params): Query<WsQueryParams>,
    ws: WebSocketUpgrade,
) -> Response {
    // Authenticate if required
    if !state.config.unsafe_no_auth {
        let expected_token = match &state.config.subscriber_token {
            Some(token) => token,
            None => {
                error!("Subscriber token not configured but auth is enabled");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new("server configuration error")),
                )
                    .into_response();
            }
        };

        let provided_token = match &params.token {
            Some(token) if !token.is_empty() => token,
            _ => {
                debug!("Missing or empty token in WebSocket request");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse::new("missing token").with_code("missing_token")),
                )
                    .into_response();
            }
        };

        if let Err(_err) = validate_token(provided_token, expected_token) {
            debug!("Invalid token in WebSocket request");
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("invalid token").with_code("invalid_token")),
            )
                .into_response();
        }
    }

    let filter = params.to_filter();
    info!(
        filter = ?filter,
        "WebSocket client connecting"
    );

    // Upgrade to WebSocket
    ws.on_upgrade(move |socket| handle_websocket(socket, state.broadcaster, filter))
}

/// Handles an established WebSocket connection.
///
/// Subscribes to the event broadcaster and forwards matching events to the client.
async fn handle_websocket(
    socket: axum::extract::ws::WebSocket,
    broadcaster: EventBroadcaster,
    filter: SubscriberFilter,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = broadcaster.subscribe();

    info!("WebSocket client connected");

    // Spawn a task to forward events to the client
    let forward_task = tokio::spawn(async move {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    // Check if event matches filter
                    if !filter.matches(&event) {
                        trace!(event_id = %event.id, "Event filtered out");
                        continue;
                    }

                    // Serialize and send event
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            trace!(event_id = %event.id, "Sending event to WebSocket client");
                            if let Err(err) = sender.send(Message::Text(json.into())).await {
                                debug!(error = %err, "Failed to send event to WebSocket client");
                                break;
                            }
                        }
                        Err(err) => {
                            error!(error = %err, "Failed to serialize event");
                        }
                    }
                }
                Err(RecvError::Lagged(count)) => {
                    warn!(skipped = count, "WebSocket client lagged, skipped events");
                }
                Err(RecvError::Closed) => {
                    debug!("Event broadcaster closed");
                    break;
                }
            }
        }
    });

    // Wait for client to disconnect
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => {
                debug!("WebSocket client sent close frame");
                break;
            }
            Ok(Message::Ping(data)) => {
                // axum handles pong automatically
                trace!(data_len = data.len(), "Received ping");
            }
            Ok(_) => {
                // Ignore other messages from client
            }
            Err(err) => {
                debug!(error = %err, "WebSocket error");
                break;
            }
        }
    }

    // Abort the forwarding task
    forward_task.abort();
    info!("WebSocket client disconnected");
}

// ============================================================================
// GET /health - Health Check
// ============================================================================

/// Response body for health check endpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Server status (always "ok" if responding).
    pub status: String,

    /// Number of active WebSocket connections.
    pub connections: usize,

    /// Server uptime in seconds.
    pub uptime_seconds: u64,
}

/// GET /health - Health check endpoint.
///
/// Returns server health status and statistics.
/// No authentication required.
///
/// # Response
///
/// ```json
/// {
///   "status": "ok",
///   "connections": 42,
///   "uptime_seconds": 3600
/// }
/// ```
async fn get_health(State(state): State<AppState>) -> Json<HealthResponse> {
    let uptime = state.start_time.elapsed();

    Json(HealthResponse {
        status: "ok".to_string(),
        connections: state.broadcaster.subscriber_count(),
        uptime_seconds: uptime.as_secs(),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use base64::prelude::*;
    use chrono::Utc;
    use ed25519_dalek::{Signer, SigningKey};
    use std::collections::HashMap;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::types::{EventPayload, SessionAction};

    /// Creates a test configuration with authentication disabled.
    fn test_config_no_auth() -> Config {
        Config {
            public_keys: HashMap::new(),
            subscriber_token: None,
            port: 8080,
            unsafe_no_auth: true,
        }
    }

    /// Creates a test configuration with authentication enabled.
    fn test_config_with_auth(public_key_base64: &str) -> Config {
        let mut public_keys = HashMap::new();
        public_keys.insert("test-source".to_string(), public_key_base64.to_string());

        Config {
            public_keys,
            subscriber_token: Some("test-token".to_string()),
            port: 8080,
            unsafe_no_auth: false,
        }
    }

    /// Creates a test signing key and returns (signing_key, public_key_base64).
    fn create_test_keypair() -> (SigningKey, String) {
        let mut seed_bytes = [0u8; 32];
        for (i, byte) in seed_bytes.iter_mut().enumerate() {
            *byte = (i as u8).wrapping_add(42);
        }
        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let public_key_base64 = BASE64_STANDARD.encode(public_key_bytes);
        (signing_key, public_key_base64)
    }

    /// Creates a test event.
    fn create_test_event() -> Event {
        Event {
            id: "evt_test12345678901234".to_string(),
            source: "test-source".to_string(),
            timestamp: Utc::now(),
            event_type: EventType::Session,
            payload: EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        }
    }

    // ========================================================================
    // Health endpoint tests
    // ========================================================================

    #[tokio::test]
    async fn health_returns_ok_status() {
        let state = AppState::new(test_config_no_auth());
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

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "ok");
        assert_eq!(health.connections, 0);
    }

    #[tokio::test]
    async fn health_reports_subscriber_count() {
        let state = AppState::new(test_config_no_auth());
        let _subscriber = state.broadcaster.subscribe();
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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.connections, 1);
    }

    // ========================================================================
    // POST /events tests (no auth)
    // ========================================================================

    #[tokio::test]
    async fn post_events_accepts_single_event() {
        let state = AppState::new(test_config_no_auth());
        let mut receiver = state.broadcaster.subscribe();
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // Verify event was broadcast
        let received = receiver.try_recv().unwrap();
        assert_eq!(received.id, event.id);
    }

    #[tokio::test]
    async fn post_events_accepts_batch_events() {
        let state = AppState::new(test_config_no_auth());
        let mut receiver = state.broadcaster.subscribe();
        let app = create_router(state);

        let events = vec![create_test_event(), create_test_event()];
        let body = serde_json::to_string(&events).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // Verify both events were broadcast
        assert!(receiver.try_recv().is_ok());
        assert!(receiver.try_recv().is_ok());
    }

    #[tokio::test]
    async fn post_events_rejects_missing_source_id() {
        let state = AppState::new(test_config_no_auth());
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    // Missing X-Source-ID header
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn post_events_rejects_invalid_json() {
        let state = AppState::new(test_config_no_auth());
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from("not valid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // ========================================================================
    // POST /events tests (with auth)
    // ========================================================================

    #[tokio::test]
    async fn post_events_with_auth_accepts_valid_signature() {
        let (signing_key, public_key_base64) = create_test_keypair();
        let state = AppState::new(test_config_with_auth(&public_key_base64));
        let mut receiver = state.broadcaster.subscribe();
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();
        let signature = signing_key.sign(body.as_bytes());
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .header(HEADER_SIGNATURE, signature_base64)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert!(receiver.try_recv().is_ok());
    }

    #[tokio::test]
    async fn post_events_with_auth_rejects_missing_signature() {
        let (_, public_key_base64) = create_test_keypair();
        let state = AppState::new(test_config_with_auth(&public_key_base64));
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    // Missing signature
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn post_events_with_auth_rejects_invalid_signature() {
        let (_, public_key_base64) = create_test_keypair();
        let state = AppState::new(test_config_with_auth(&public_key_base64));
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();

        // Create a different keypair for invalid signature
        let (wrong_key, _) = {
            let mut seed_bytes = [0u8; 32];
            for (i, byte) in seed_bytes.iter_mut().enumerate() {
                *byte = (i as u8).wrapping_add(100);
            }
            let signing_key = SigningKey::from_bytes(&seed_bytes);
            let public_key_bytes = signing_key.verifying_key().to_bytes();
            let public_key_base64 = BASE64_STANDARD.encode(public_key_bytes);
            (signing_key, public_key_base64)
        };

        let signature = wrong_key.sign(body.as_bytes());
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .header(HEADER_SIGNATURE, signature_base64)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn post_events_with_auth_rejects_unknown_source() {
        let (signing_key, public_key_base64) = create_test_keypair();
        let state = AppState::new(test_config_with_auth(&public_key_base64));
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();
        let signature = signing_key.sign(body.as_bytes());
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "unknown-source") // Not in config
                    .header(HEADER_SIGNATURE, signature_base64)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ========================================================================
    // Rate limiting tests
    // ========================================================================

    #[tokio::test]
    async fn post_events_rate_limits_when_exceeded() {
        let state = AppState::with_components(
            test_config_no_auth(),
            EventBroadcaster::new(),
            RateLimiter::new(1.0, 1), // Very low limit: 1 request allowed
        );
        let app = create_router(state);

        let event = create_test_event();
        let body = serde_json::to_string(&event).unwrap();

        // First request should succeed
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);

        // Second request should be rate limited
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key(HEADER_RETRY_AFTER));
    }

    // ========================================================================
    // Source validation tests
    // ========================================================================

    #[tokio::test]
    async fn post_events_rejects_source_mismatch() {
        let state = AppState::new(test_config_no_auth());
        let app = create_router(state);

        // Create event with source "wrong-source" but send with header "test-source"
        let mut event = create_test_event();
        event.source = "wrong-source".to_string();
        let body = serde_json::to_string(&event).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["code"], "source_mismatch");
    }

    #[tokio::test]
    async fn post_events_rejects_mixed_source_batch() {
        let state = AppState::new(test_config_no_auth());
        let app = create_router(state);

        // Create batch where first event matches but second doesn't
        let mut event1 = create_test_event();
        event1.source = "test-source".to_string();
        let mut event2 = create_test_event();
        event2.source = "other-source".to_string();

        let events = vec![event1, event2];
        let body = serde_json::to_string(&events).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(error["code"], "source_mismatch");
    }

    #[tokio::test]
    async fn post_events_accepts_matching_source() {
        let state = AppState::new(test_config_no_auth());
        let mut receiver = state.broadcaster.subscribe();
        let app = create_router(state);

        let mut event = create_test_event();
        event.source = "matching-source".to_string();
        let body = serde_json::to_string(&event).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "matching-source")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert!(receiver.try_recv().is_ok());
    }

    // ========================================================================
    // Body size limit tests
    // ========================================================================

    #[tokio::test]
    async fn post_events_rejects_oversized_request() {
        let state = AppState::new(test_config_no_auth());
        let app = create_router(state);

        // Create a body larger than MAX_BODY_SIZE (1 MB)
        let oversized_body = "x".repeat(1024 * 1024 + 1);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/events")
                    .header("Content-Type", "application/json")
                    .header(HEADER_SOURCE_ID, "test-source")
                    .body(Body::from(oversized_body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    // ========================================================================
    // WebSocket query params tests
    // ========================================================================

    #[test]
    fn ws_query_params_builds_empty_filter() {
        let params = WsQueryParams {
            token: Some("test".to_string()),
            source: None,
            event_type: None,
            project: None,
        };

        let filter = params.to_filter();
        assert!(filter.is_empty());
    }

    #[test]
    fn ws_query_params_builds_filter_with_source() {
        let params = WsQueryParams {
            token: None,
            source: Some("monitor-1".to_string()),
            event_type: None,
            project: None,
        };

        let filter = params.to_filter();
        assert_eq!(filter.source, Some("monitor-1".to_string()));
    }

    #[test]
    fn ws_query_params_builds_filter_with_all_fields() {
        let params = WsQueryParams {
            token: None,
            source: Some("monitor-1".to_string()),
            event_type: Some(EventType::Tool),
            project: Some("my-project".to_string()),
        };

        let filter = params.to_filter();
        assert_eq!(filter.source, Some("monitor-1".to_string()));
        assert_eq!(filter.event_type, Some(EventType::Tool));
        assert_eq!(filter.project, Some("my-project".to_string()));
    }

    // ========================================================================
    // AppState tests
    // ========================================================================

    #[test]
    fn app_state_new_creates_valid_state() {
        let state = AppState::new(test_config_no_auth());
        assert!(state.config.unsafe_no_auth);
        assert_eq!(state.broadcaster.subscriber_count(), 0);
    }

    #[test]
    fn app_state_debug_impl() {
        let state = AppState::new(test_config_no_auth());
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
    }

    // ========================================================================
    // Events payload tests
    // ========================================================================

    #[test]
    fn events_payload_single_into_events() {
        let event = create_test_event();
        let payload = EventsPayload::Single(event.clone());
        let events = payload.into_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[test]
    fn events_payload_batch_into_events() {
        let events = vec![create_test_event(), create_test_event()];
        let payload = EventsPayload::Batch(events.clone());
        let result = payload.into_events();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn events_payload_deserializes_single() {
        let event = create_test_event();
        let json = serde_json::to_string(&event).unwrap();
        let payload: EventsPayload = serde_json::from_str(&json).unwrap();
        assert!(matches!(payload, EventsPayload::Single(_)));
    }

    #[test]
    fn events_payload_deserializes_batch() {
        let events = vec![create_test_event(), create_test_event()];
        let json = serde_json::to_string(&events).unwrap();
        let payload: EventsPayload = serde_json::from_str(&json).unwrap();
        assert!(matches!(payload, EventsPayload::Batch(_)));
    }

    // ========================================================================
    // Error response tests
    // ========================================================================

    #[test]
    fn error_response_serializes_without_code() {
        let response = ErrorResponse::new("test error");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test error"));
        assert!(!json.contains("code"));
    }

    #[test]
    fn error_response_serializes_with_code() {
        let response = ErrorResponse::new("test error").with_code("test_code");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test error"));
        assert!(json.contains("test_code"));
    }
}
