//! Privacy tests for authentication data in logs.
//!
//! These tests verify compliance with Constitution I - Privacy by Design:
//! - No JWTs or session tokens should appear in log output, even at TRACE level.
//!
//! # Test Approach
//!
//! 1. Use a custom tracing subscriber Layer to capture all log messages
//! 2. Exercise session store and Supabase client code paths
//! 3. Verify that sensitive data (tokens, JWTs) does NOT appear in captured logs

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tracing::Subscriber;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use vibetea_server::session::{SessionStore, SessionStoreConfig};
use vibetea_server::supabase::SupabaseClient;

// ============================================================================
// Log Capture Infrastructure
// ============================================================================

/// A buffer for capturing log output during tests.
#[derive(Clone, Default)]
struct LogCapture {
    /// Captured log messages (field + message content).
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogCapture {
    /// Creates a new log capture buffer.
    fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a clone of all captured log messages joined into a single string.
    fn get_logs(&self) -> String {
        self.logs.lock().unwrap().join("\n")
    }

    /// Clears all captured logs.
    #[allow(dead_code)]
    fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }
}

/// A tracing Layer that captures log events for inspection.
struct CaptureLayer {
    capture: LogCapture,
}

impl CaptureLayer {
    fn new(capture: LogCapture) -> Self {
        Self { capture }
    }
}

impl<S> Layer<S> for CaptureLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = StringVisitor::new();
        event.record(&mut visitor);

        let message = format!(
            "[{}] {}: {}",
            event.metadata().level(),
            event.metadata().target(),
            visitor.into_string()
        );

        self.capture.logs.lock().unwrap().push(message);
    }
}

/// A visitor that collects all event fields into a string.
struct StringVisitor {
    parts: Vec<String>,
}

impl StringVisitor {
    fn new() -> Self {
        Self { parts: Vec::new() }
    }

    fn into_string(self) -> String {
        self.parts.join(" ")
    }
}

impl tracing::field::Visit for StringVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.parts.push(format!("{}={:?}", field.name(), value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.parts.push(format!("{}={}", field.name(), value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.parts.push(format!("{}={}", field.name(), value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.parts.push(format!("{}={}", field.name(), value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.parts.push(format!("{}={}", field.name(), value));
    }
}

/// Runs a test closure with log capture at TRACE level.
///
/// Returns the captured logs for assertion.
fn with_log_capture<F>(test_fn: F) -> String
where
    F: FnOnce(),
{
    let capture = LogCapture::new();
    let layer = CaptureLayer::new(capture.clone());

    // Build a subscriber that captures at TRACE level
    let subscriber = tracing_subscriber::registry()
        .with(layer.with_filter(tracing_subscriber::filter::LevelFilter::TRACE));

    // Run the test with our capturing subscriber
    tracing::subscriber::with_default(subscriber, test_fn);

    capture.get_logs()
}

/// Async version of with_log_capture for async tests.
async fn with_log_capture_async<F, Fut>(test_fn: F) -> String
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let capture = LogCapture::new();
    let layer = CaptureLayer::new(capture.clone());

    let subscriber = tracing_subscriber::registry()
        .with(layer.with_filter(tracing_subscriber::filter::LevelFilter::TRACE));

    // Run the async test with our capturing subscriber
    let _guard = tracing::subscriber::set_default(subscriber);
    test_fn().await;

    capture.get_logs()
}

// ============================================================================
// Privacy Assertion Helpers
// ============================================================================

/// Asserts that the given token does not appear in the logs.
fn assert_token_not_in_logs(logs: &str, token: &str, context: &str) {
    assert!(
        !logs.contains(token),
        "Session token found in logs during {context}!\nToken: {token}\nLogs:\n{logs}"
    );
}

/// Asserts that no JWT prefix (eyJ) appears in the logs.
///
/// JWTs are base64-encoded JSON that always starts with "eyJ" because
/// the header starts with `{"`.
fn assert_no_jwt_in_logs(logs: &str, context: &str) {
    // Check for the JWT header prefix
    assert!(
        !logs.contains("eyJ"),
        "JWT prefix 'eyJ' found in logs during {context}!\nLogs:\n{logs}"
    );
}

/// Asserts that the given sensitive value does not appear in logs.
fn assert_sensitive_not_in_logs(logs: &str, value: &str, value_name: &str, context: &str) {
    assert!(
        !logs.contains(value),
        "{value_name} found in logs during {context}!\nValue: {value}\nLogs:\n{logs}"
    );
}

// ============================================================================
// Test Cases
// ============================================================================

/// Test that session tokens are not logged when creating a session.
///
/// This verifies that `SessionStore::create_session` does not log the
/// generated token, even at TRACE level.
#[test]
fn session_token_not_logged_on_creation() {
    let logs = with_log_capture(|| {
        let store = SessionStore::new(SessionStoreConfig::default());

        // Create multiple sessions to exercise the code path thoroughly
        let token1 = store
            .create_session("user-123".to_string(), Some("user@example.com".to_string()))
            .expect("should create session");

        let token2 = store
            .create_session("user-456".to_string(), None)
            .expect("should create session");

        // Verify tokens were generated (sanity check)
        assert_eq!(token1.len(), 43, "Token should be 43 characters");
        assert_eq!(token2.len(), 43, "Token should be 43 characters");

        // Store tokens for assertion (we need them outside the closure)
        // We'll use thread-local storage to pass them out
        CAPTURED_TOKENS.with(|t| {
            t.borrow_mut().push(token1);
            t.borrow_mut().push(token2);
        });
    });

    // Retrieve captured tokens and verify they're not in logs
    CAPTURED_TOKENS.with(|t| {
        let tokens = t.borrow();
        for token in tokens.iter() {
            assert_token_not_in_logs(&logs, token, "session creation");
        }
    });

    // Clear for next test
    CAPTURED_TOKENS.with(|t| t.borrow_mut().clear());
}

// Thread-local storage for capturing tokens across closure boundary
std::thread_local! {
    static CAPTURED_TOKENS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Test that session tokens are not logged when validating a session.
///
/// This verifies that `SessionStore::validate_session` does not log the
/// token being validated, even at TRACE level.
#[test]
fn session_token_not_logged_on_validation() {
    let store = SessionStore::new(SessionStoreConfig::default());

    // Create session outside of log capture
    let token = store
        .create_session("user-789".to_string(), Some("test@example.com".to_string()))
        .expect("should create session");

    // Now capture logs during validation
    let logs = with_log_capture(|| {
        // Valid session lookup
        let result = store.validate_session(&token, false);
        assert!(result.is_some(), "Session should be valid");

        // Validate with grace period
        let result = store.validate_session(&token, true);
        assert!(
            result.is_some(),
            "Session should be valid with grace period"
        );

        // Invalid token lookup (should not log the attempted token)
        let invalid_token = "x".repeat(43);
        let result = store.validate_session(&invalid_token, false);
        assert!(result.is_none(), "Invalid session should not be found");
    });

    // Verify the actual token is not in logs
    assert_token_not_in_logs(&logs, &token, "session validation");

    // Also check that the invalid token attempt is not logged
    let invalid_token = "x".repeat(43);
    assert_token_not_in_logs(&logs, &invalid_token, "invalid session validation attempt");
}

/// Test that session tokens are not logged when a session expires.
///
/// This verifies that cleanup operations do not log expired tokens.
#[test]
fn session_token_not_logged_on_expiry() {
    // Create a store with very short TTL for testing
    let config = SessionStoreConfig::new(100, Duration::from_millis(5), Duration::from_millis(0));
    let store = SessionStore::new(config);

    // Create sessions outside of log capture
    let token1 = store
        .create_session("expiring-user-1".to_string(), None)
        .expect("should create session");
    let token2 = store
        .create_session("expiring-user-2".to_string(), None)
        .expect("should create session");

    // Wait for expiration
    std::thread::sleep(Duration::from_millis(20));

    // Capture logs during cleanup
    let logs = with_log_capture(|| {
        // Manual cleanup should not log tokens
        let removed = store.cleanup_expired();
        assert_eq!(removed, 2, "Both sessions should be cleaned up");

        // Validation of expired token should not log the token
        let result = store.validate_session(&token1, false);
        assert!(result.is_none(), "Expired session should not be valid");
    });

    // Verify tokens are not in cleanup logs
    assert_token_not_in_logs(&logs, &token1, "session expiry cleanup");
    assert_token_not_in_logs(&logs, &token2, "session expiry cleanup");
}

/// Test that session tokens are not logged during TTL extension.
///
/// This verifies that `SessionStore::extend_session_ttl` does not log tokens.
#[test]
fn session_token_not_logged_on_ttl_extension() {
    let store = SessionStore::new(SessionStoreConfig::default());

    // Create session outside of log capture
    let token = store
        .create_session("extend-user".to_string(), None)
        .expect("should create session");

    // Capture logs during TTL extension
    let logs = with_log_capture(|| {
        // Extend TTL
        let extended = store.extend_session_ttl(&token).expect("should extend");
        assert!(extended, "TTL should be extended on first call");

        // Try to extend again (should return false but not log token)
        let extended_again = store.extend_session_ttl(&token).expect("should not error");
        assert!(!extended_again, "TTL should not extend twice");
    });

    assert_token_not_in_logs(&logs, &token, "TTL extension");
}

/// Test that session tokens are not logged during session removal.
///
/// This verifies that `SessionStore::remove_session` does not log tokens.
#[test]
fn session_token_not_logged_on_removal() {
    let store = SessionStore::new(SessionStoreConfig::default());

    // Create session outside of log capture
    let token = store
        .create_session("remove-user".to_string(), None)
        .expect("should create session");

    // Capture logs during removal
    let logs = with_log_capture(|| {
        let removed = store.remove_session(&token);
        assert!(removed.is_some(), "Session should be removed");

        // Try to remove again (should return None but not log token)
        let removed_again = store.remove_session(&token);
        assert!(removed_again.is_none(), "Session should already be removed");
    });

    assert_token_not_in_logs(&logs, &token, "session removal");
}

/// Test that JWTs are not logged during validation attempts.
///
/// This verifies that `SupabaseClient::validate_jwt` does not log the JWT,
/// even when validation fails.
#[tokio::test]
async fn jwt_not_logged_on_validation_attempt() {
    // Create a mock server for Supabase
    let mock_server = MockServer::start().await;

    // A realistic-looking JWT (structure: header.payload.signature, all base64)
    let test_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.\
        SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

    // Mock successful validation
    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "user-abc123",
            "email": "jwt-user@example.com"
        })))
        .mount(&mock_server)
        .await;

    let client =
        SupabaseClient::new(mock_server.uri(), "test-anon-key").expect("should create client");

    // Capture logs during JWT validation
    let logs = with_log_capture_async(|| async {
        let result = client.validate_jwt(test_jwt).await;
        assert!(result.is_ok(), "JWT validation should succeed");
    })
    .await;

    // Verify JWT is not in logs
    assert_no_jwt_in_logs(&logs, "successful JWT validation");
    assert_sensitive_not_in_logs(&logs, test_jwt, "JWT", "successful JWT validation");
}

/// Test that JWTs are not logged when validation fails (401 Unauthorized).
#[tokio::test]
async fn jwt_not_logged_on_validation_failure() {
    let mock_server = MockServer::start().await;

    let invalid_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJleHAiOjAsInN1YiI6ImludmFsaWQifQ.\
        invalid_signature_here";

    // Mock 401 response
    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "invalid_token",
            "error_description": "Token is expired or invalid"
        })))
        .mount(&mock_server)
        .await;

    let client =
        SupabaseClient::new(mock_server.uri(), "test-anon-key").expect("should create client");

    let logs = with_log_capture_async(|| async {
        let result = client.validate_jwt(invalid_jwt).await;
        assert!(result.is_err(), "JWT validation should fail");
    })
    .await;

    // Verify invalid JWT is not logged
    assert_no_jwt_in_logs(&logs, "failed JWT validation");
    assert_sensitive_not_in_logs(&logs, invalid_jwt, "invalid JWT", "failed JWT validation");
}

/// Test that JWTs are not logged when Supabase returns an error.
#[tokio::test]
async fn jwt_not_logged_on_server_error() {
    let mock_server = MockServer::start().await;

    let test_jwt = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJpc3MiOiJodHRwczovL2V4YW1wbGUuY29tIiwic3ViIjoiMTIzIn0.\
        test_signature";

    // Mock 500 error
    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client =
        SupabaseClient::new(mock_server.uri(), "test-anon-key").expect("should create client");

    let logs = with_log_capture_async(|| async {
        let result = client.validate_jwt(test_jwt).await;
        assert!(
            result.is_err(),
            "JWT validation should fail on server error"
        );
    })
    .await;

    assert_no_jwt_in_logs(&logs, "server error during JWT validation");
    assert_sensitive_not_in_logs(&logs, test_jwt, "JWT", "server error during JWT validation");
}

/// Test that multiple sensitive values don't leak in a combined workflow.
///
/// This simulates a realistic authentication flow where both JWTs and
/// session tokens are involved.
#[tokio::test]
async fn combined_auth_flow_does_not_leak_secrets() {
    let mock_server = MockServer::start().await;

    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
        eyJzdWIiOiJ1c2VyLTk5OSIsImVtYWlsIjoiY29tYmluZWRAZXhhbXBsZS5jb20ifQ.\
        combined_test_signature";

    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "user-999",
            "email": "combined@example.com"
        })))
        .mount(&mock_server)
        .await;

    let client =
        SupabaseClient::new(mock_server.uri(), "test-anon-key").expect("should create client");
    let store = SessionStore::new(SessionStoreConfig::default());

    let mut captured_token = String::new();

    let logs = with_log_capture_async(|| async {
        // Step 1: Validate JWT (simulating token exchange)
        let user = client.validate_jwt(jwt).await.expect("should validate");

        // Step 2: Create session
        let token = store
            .create_session(user.id.clone(), user.email.clone())
            .expect("should create session");

        // Step 3: Validate session
        let session = store.validate_session(&token, false);
        assert!(session.is_some());

        // Step 4: Extend session TTL
        let _ = store.extend_session_ttl(&token);

        // Step 5: Validate again with grace period
        let _ = store.validate_session(&token, true);

        // Save token for assertion
        CAPTURED_TOKENS.with(|t| {
            t.borrow_mut().push(token);
        });
    })
    .await;

    // Get the captured token
    CAPTURED_TOKENS.with(|t| {
        if let Some(token) = t.borrow().first() {
            captured_token = token.clone();
        }
    });
    CAPTURED_TOKENS.with(|t| t.borrow_mut().clear());

    // Verify neither JWT nor session token appears in logs
    assert_no_jwt_in_logs(&logs, "combined auth flow");
    assert_sensitive_not_in_logs(&logs, jwt, "JWT", "combined auth flow");
    assert_token_not_in_logs(&logs, &captured_token, "combined auth flow");
}

/// Test that the store's debug representation doesn't leak session tokens.
#[test]
fn session_store_debug_does_not_leak_tokens() {
    let store = SessionStore::new(SessionStoreConfig::default());

    let token = store
        .create_session("debug-user".to_string(), None)
        .expect("should create session");

    // Get debug output
    let debug_output = format!("{:?}", store);

    // Token should not appear in debug output
    assert!(
        !debug_output.contains(&token),
        "Session token found in Debug output!\nToken: {token}\nDebug: {debug_output}"
    );

    // Should also not contain JWT-like patterns
    assert!(
        !debug_output.contains("eyJ"),
        "JWT-like pattern found in Debug output!\nDebug: {debug_output}"
    );
}

/// Test that even at TRACE level, no secrets are leaked during capacity warnings.
#[test]
fn capacity_warnings_do_not_leak_tokens() {
    // Create a store with very low capacity
    let config = SessionStoreConfig::new(2, Duration::from_secs(300), Duration::from_secs(30));
    let store = SessionStore::new(config);

    // Fill to capacity outside of log capture
    let token1 = store
        .create_session("cap-user-1".to_string(), None)
        .expect("should create");
    let token2 = store
        .create_session("cap-user-2".to_string(), None)
        .expect("should create");

    // Now capture logs when capacity is reached
    let logs = with_log_capture(|| {
        // This should fail with AtCapacity error
        let result = store.create_session("cap-user-3".to_string(), None);
        assert!(result.is_err(), "Should fail at capacity");
    });

    // Existing tokens should not be logged in capacity warning
    assert_token_not_in_logs(&logs, &token1, "capacity warning");
    assert_token_not_in_logs(&logs, &token2, "capacity warning");
}
