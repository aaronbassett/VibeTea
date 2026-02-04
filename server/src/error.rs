//! Error types for the VibeTea server.
//!
//! This module defines the error hierarchy used throughout the server,
//! providing type-safe error handling with meaningful error messages.
//!
//! # Error Types
//!
//! - [`ConfigError`] - Configuration-related errors (missing values, parse failures)
//! - [`ServerError`] - Top-level server errors encompassing all failure modes
//!
//! # HTTP Status Code Mapping
//!
//! `ServerError` implements `IntoResponse` for use with Axum, mapping each
//! variant to an appropriate HTTP status code:
//!
//! | Variant | HTTP Status |
//! |---------|-------------|
//! | `Config` | 500 Internal Server Error |
//! | `Auth` | 401 Unauthorized |
//! | `JwtInvalid` | 401 Unauthorized |
//! | `SessionInvalid` | 401 Unauthorized |
//! | `Validation` | 400 Bad Request |
//! | `RateLimit` | 429 Too Many Requests |
//! | `SessionCapacityExceeded` | 503 Service Unavailable |
//! | `SupabaseUnavailable` | 503 Service Unavailable |
//! | `WebSocket` | 500 Internal Server Error |
//! | `Internal` | 500 Internal Server Error |
//!
//! # Example
//!
//! ```rust,ignore
//! use vibetea_server::error::{ServerError, ConfigError};
//!
//! fn validate_event(event: &Event) -> Result<(), ServerError> {
//!     if event.payload.is_empty() {
//!         return Err(ServerError::Validation("event payload cannot be empty".into()));
//!     }
//!     Ok(())
//! }
//! ```

use std::error::Error;
use std::fmt;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use thiserror::Error as ThisError;

/// Errors that occur during configuration loading and validation.
///
/// These errors indicate problems with the server configuration,
/// such as missing required values or invalid formats.
#[derive(ThisError, Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A required configuration value is missing.
    #[error("missing required configuration: {0}")]
    Missing(String),

    /// A configuration value failed to parse or is invalid.
    #[error("invalid configuration value for '{key}': {reason}")]
    Invalid {
        /// The configuration key that has an invalid value.
        key: String,
        /// Description of why the value is invalid.
        reason: String,
    },

    /// Failed to read or parse the configuration file.
    #[error("failed to load configuration file: {0}")]
    FileError(String),
}

/// Top-level error type for the VibeTea server.
///
/// This enum encompasses all error types that can occur during server operation,
/// from configuration issues to runtime errors like authentication failures
/// and rate limiting.
///
/// # Error Categories
///
/// - **Configuration errors**: Problems loading or validating server config
/// - **Authentication errors**: Invalid credentials, expired tokens, etc.
/// - **JWT errors**: Invalid or expired JWT tokens (FR-028)
/// - **Session errors**: Invalid or expired session tokens (FR-029)
/// - **Validation errors**: Malformed events or invalid request data
/// - **Rate limiting**: Client exceeded allowed request rate
/// - **Service availability**: Supabase unavailable, session capacity exceeded
/// - **WebSocket errors**: Connection issues with WebSocket clients
/// - **Internal errors**: Unexpected failures that don't fit other categories
#[derive(Debug)]
pub enum ServerError {
    /// Configuration error during server initialization or runtime.
    Config(ConfigError),

    /// Authentication or authorization failure.
    ///
    /// This includes invalid API keys, expired tokens, and insufficient
    /// permissions for the requested operation.
    Auth(String),

    /// JWT validation failed (invalid or expired).
    ///
    /// Returned when a Supabase JWT token is invalid, expired, or cannot
    /// be validated. Maps to HTTP 401 Unauthorized (FR-028).
    JwtInvalid(String),

    /// Session token is invalid or expired.
    ///
    /// Returned when a session token provided by the client does not exist
    /// in the session store or has expired. Maps to HTTP 401 Unauthorized (FR-029).
    SessionInvalid(String),

    /// Event or request validation failure.
    ///
    /// Returned when an incoming event or API request fails validation,
    /// such as missing required fields or invalid field values.
    Validation(String),

    /// Rate limit exceeded.
    ///
    /// Returned when a client has exceeded their allowed request rate.
    /// The `retry_after` field indicates how many seconds the client
    /// should wait before retrying.
    RateLimit {
        /// Identifier for the rate-limited source (e.g., IP address, client ID).
        source: String,
        /// Number of seconds until the rate limit resets.
        retry_after: u64,
    },

    /// Session store is at capacity.
    ///
    /// Returned when the session store has reached its maximum capacity
    /// and cannot accept new sessions. Maps to HTTP 503 Service Unavailable (FR-022).
    SessionCapacityExceeded,

    /// Supabase service is unavailable.
    ///
    /// Returned when the Supabase API cannot be reached for JWT validation
    /// or other operations. Maps to HTTP 503 Service Unavailable (FR-030/FR-031).
    SupabaseUnavailable(String),

    /// WebSocket connection or protocol error.
    ///
    /// Covers failures in WebSocket handshakes, message framing,
    /// and connection management.
    WebSocket(String),

    /// Unexpected internal server error.
    ///
    /// Used for errors that don't fit into other categories, typically
    /// indicating bugs or unexpected system failures.
    Internal(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "configuration error: {err}"),
            Self::Auth(msg) => write!(f, "authentication failed: {msg}"),
            Self::JwtInvalid(msg) => write!(f, "JWT validation failed: {msg}"),
            Self::SessionInvalid(msg) => write!(f, "session invalid: {msg}"),
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
            Self::RateLimit {
                source,
                retry_after,
            } => {
                write!(
                    f,
                    "rate limit exceeded for {source}, retry after {retry_after} seconds"
                )
            }
            Self::SessionCapacityExceeded => write!(f, "session capacity exceeded"),
            Self::SupabaseUnavailable(msg) => write!(f, "Supabase unavailable: {msg}"),
            Self::WebSocket(msg) => write!(f, "websocket error: {msg}"),
            Self::Internal(msg) => write!(f, "internal server error: {msg}"),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Config(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ConfigError> for ServerError {
    fn from(err: ConfigError) -> Self {
        Self::Config(err)
    }
}

// ============================================================================
// JSON Error Response
// ============================================================================

/// JSON error response body for HTTP responses.
#[derive(Debug, Serialize)]
struct ErrorResponseBody {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
}

impl ErrorResponseBody {
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
// IntoResponse Implementation
// ============================================================================

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::Config(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_error",
                self.to_string(),
            ),
            Self::Auth(msg) => (StatusCode::UNAUTHORIZED, "auth_error", msg.clone()),
            Self::JwtInvalid(msg) => (StatusCode::UNAUTHORIZED, "jwt_invalid", msg.clone()),
            Self::SessionInvalid(msg) => (StatusCode::UNAUTHORIZED, "session_invalid", msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            Self::RateLimit { retry_after, .. } => {
                // For rate limiting, we return with Retry-After header
                let body = ErrorResponseBody::new("rate limit exceeded").with_code("rate_limited");
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("Retry-After", retry_after.to_string())],
                    Json(body),
                )
                    .into_response();
            }
            Self::SessionCapacityExceeded => (
                StatusCode::SERVICE_UNAVAILABLE,
                "session_capacity_exceeded",
                "session store is at capacity".to_string(),
            ),
            Self::SupabaseUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "supabase_unavailable",
                msg.clone(),
            ),
            Self::WebSocket(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "websocket_error",
                msg.clone(),
            ),
            Self::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                msg.clone(),
            ),
        };

        let body = ErrorResponseBody::new(message).with_code(code);
        (status, Json(body)).into_response()
    }
}

impl ServerError {
    /// Creates a new authentication error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::auth("invalid API key");
    /// assert!(matches!(err, ServerError::Auth(_)));
    /// ```
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Creates a new validation error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::validation("missing 'event_type' field");
    /// assert!(matches!(err, ServerError::Validation(_)));
    /// ```
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Creates a new rate limit error.
    ///
    /// # Arguments
    ///
    /// * `source` - Identifier for the rate-limited source
    /// * `retry_after` - Seconds until the client can retry
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::rate_limit("192.168.1.1", 60);
    /// assert!(matches!(err, ServerError::RateLimit { .. }));
    /// ```
    pub fn rate_limit(source: impl Into<String>, retry_after: u64) -> Self {
        Self::RateLimit {
            source: source.into(),
            retry_after,
        }
    }

    /// Creates a new WebSocket error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::websocket("connection reset by peer");
    /// assert!(matches!(err, ServerError::WebSocket(_)));
    /// ```
    pub fn websocket(message: impl Into<String>) -> Self {
        Self::WebSocket(message.into())
    }

    /// Creates a new internal error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::internal("database connection pool exhausted");
    /// assert!(matches!(err, ServerError::Internal(_)));
    /// ```
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Creates a new JWT validation error.
    ///
    /// Used when a Supabase JWT token is invalid or expired.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::jwt_invalid("token has expired");
    /// assert!(matches!(err, ServerError::JwtInvalid(_)));
    /// ```
    pub fn jwt_invalid(message: impl Into<String>) -> Self {
        Self::JwtInvalid(message.into())
    }

    /// Creates a new session invalid error.
    ///
    /// Used when a session token does not exist or has expired.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::session_invalid("session has expired");
    /// assert!(matches!(err, ServerError::SessionInvalid(_)));
    /// ```
    pub fn session_invalid(message: impl Into<String>) -> Self {
        Self::SessionInvalid(message.into())
    }

    /// Creates a new session capacity exceeded error.
    ///
    /// Used when the session store has reached its maximum capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::session_capacity_exceeded();
    /// assert!(matches!(err, ServerError::SessionCapacityExceeded));
    /// ```
    pub fn session_capacity_exceeded() -> Self {
        Self::SessionCapacityExceeded
    }

    /// Creates a new Supabase unavailable error.
    ///
    /// Used when the Supabase API cannot be reached.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ServerError;
    ///
    /// let err = ServerError::supabase_unavailable("connection timeout");
    /// assert!(matches!(err, ServerError::SupabaseUnavailable(_)));
    /// ```
    pub fn supabase_unavailable(message: impl Into<String>) -> Self {
        Self::SupabaseUnavailable(message.into())
    }

    /// Returns `true` if this error indicates a client-side problem.
    ///
    /// Client errors are those where the client made an invalid request,
    /// as opposed to server-side failures.
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Auth(_)
                | Self::JwtInvalid(_)
                | Self::SessionInvalid(_)
                | Self::Validation(_)
                | Self::RateLimit { .. }
        )
    }

    /// Returns `true` if this error indicates a server-side problem.
    ///
    /// This includes configuration errors, service unavailability, and
    /// unexpected internal failures.
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Internal(_)
                | Self::Config(_)
                | Self::SessionCapacityExceeded
                | Self::SupabaseUnavailable(_)
        )
    }

    /// Returns the appropriate HTTP status code for this error.
    ///
    /// This is useful when you need the status code without converting
    /// the entire error to a response.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::JwtInvalid(_) => StatusCode::UNAUTHORIZED,
            Self::SessionInvalid(_) => StatusCode::UNAUTHORIZED,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::RateLimit { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::SessionCapacityExceeded => StatusCode::SERVICE_UNAVAILABLE,
            Self::SupabaseUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::WebSocket(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ConfigError {
    /// Creates a new missing configuration error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ConfigError;
    ///
    /// let err = ConfigError::missing("VIBETEA_API_KEY");
    /// assert!(matches!(err, ConfigError::Missing(_)));
    /// ```
    pub fn missing(key: impl Into<String>) -> Self {
        Self::Missing(key.into())
    }

    /// Creates a new invalid configuration error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ConfigError;
    ///
    /// let err = ConfigError::invalid("port", "must be a number between 1 and 65535");
    /// assert!(matches!(err, ConfigError::Invalid { .. }));
    /// ```
    pub fn invalid(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Invalid {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Creates a new file error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::error::ConfigError;
    ///
    /// let err = ConfigError::file_error("config.toml not found");
    /// assert!(matches!(err, ConfigError::FileError(_)));
    /// ```
    pub fn file_error(message: impl Into<String>) -> Self {
        Self::FileError(message.into())
    }
}

/// A specialized Result type for server operations.
pub type Result<T> = std::result::Result<T, ServerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_missing_displays_correctly() {
        let err = ConfigError::missing("API_KEY");
        assert_eq!(err.to_string(), "missing required configuration: API_KEY");
    }

    #[test]
    fn config_error_invalid_displays_correctly() {
        let err = ConfigError::invalid("port", "must be a positive integer");
        assert_eq!(
            err.to_string(),
            "invalid configuration value for 'port': must be a positive integer"
        );
    }

    #[test]
    fn config_error_file_displays_correctly() {
        let err = ConfigError::file_error("permission denied");
        assert_eq!(
            err.to_string(),
            "failed to load configuration file: permission denied"
        );
    }

    #[test]
    fn server_error_config_displays_correctly() {
        let config_err = ConfigError::missing("SECRET_KEY");
        let err = ServerError::Config(config_err);
        assert_eq!(
            err.to_string(),
            "configuration error: missing required configuration: SECRET_KEY"
        );
    }

    #[test]
    fn server_error_auth_displays_correctly() {
        let err = ServerError::auth("invalid token");
        assert_eq!(err.to_string(), "authentication failed: invalid token");
    }

    #[test]
    fn server_error_validation_displays_correctly() {
        let err = ServerError::validation("missing required field 'event_type'");
        assert_eq!(
            err.to_string(),
            "validation error: missing required field 'event_type'"
        );
    }

    #[test]
    fn server_error_rate_limit_displays_correctly() {
        let err = ServerError::rate_limit("192.168.1.100", 30);
        assert_eq!(
            err.to_string(),
            "rate limit exceeded for 192.168.1.100, retry after 30 seconds"
        );
    }

    #[test]
    fn server_error_websocket_displays_correctly() {
        let err = ServerError::websocket("connection closed unexpectedly");
        assert_eq!(
            err.to_string(),
            "websocket error: connection closed unexpectedly"
        );
    }

    #[test]
    fn server_error_internal_displays_correctly() {
        let err = ServerError::internal("database connection failed");
        assert_eq!(
            err.to_string(),
            "internal server error: database connection failed"
        );
    }

    #[test]
    fn config_error_converts_to_server_error() {
        let config_err = ConfigError::missing("PORT");
        let server_err: ServerError = config_err.into();
        assert!(matches!(server_err, ServerError::Config(_)));
    }

    #[test]
    fn from_config_error_works_with_question_mark() {
        fn inner() -> std::result::Result<(), ServerError> {
            let _: () = Err(ConfigError::missing("KEY"))?;
            Ok(())
        }

        let result = inner();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServerError::Config(_)));
    }

    #[test]
    fn is_client_error_returns_true_for_client_errors() {
        assert!(ServerError::auth("bad token").is_client_error());
        assert!(ServerError::validation("bad input").is_client_error());
        assert!(ServerError::rate_limit("client", 60).is_client_error());
    }

    #[test]
    fn is_client_error_returns_false_for_server_errors() {
        assert!(!ServerError::internal("oops").is_client_error());
        assert!(!ServerError::Config(ConfigError::missing("X")).is_client_error());
        assert!(!ServerError::websocket("connection lost").is_client_error());
    }

    #[test]
    fn is_server_error_returns_true_for_server_errors() {
        assert!(ServerError::internal("oops").is_server_error());
        assert!(ServerError::Config(ConfigError::missing("X")).is_server_error());
    }

    #[test]
    fn is_server_error_returns_false_for_client_errors() {
        assert!(!ServerError::auth("bad token").is_server_error());
        assert!(!ServerError::validation("bad input").is_server_error());
        assert!(!ServerError::rate_limit("client", 60).is_server_error());
    }

    #[test]
    fn config_error_is_clone_and_eq() {
        let err1 = ConfigError::missing("KEY");
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn server_error_is_debug() {
        let err = ServerError::auth("test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Auth"));
    }

    #[test]
    fn server_error_source_returns_config_error() {
        let config_err = ConfigError::missing("KEY");
        let server_err = ServerError::Config(config_err.clone());

        let source = server_err.source();
        assert!(source.is_some());
        assert_eq!(source.unwrap().to_string(), config_err.to_string());
    }

    #[test]
    fn server_error_source_returns_none_for_other_variants() {
        assert!(ServerError::auth("test").source().is_none());
        assert!(ServerError::validation("test").source().is_none());
        assert!(ServerError::rate_limit("test", 60).source().is_none());
        assert!(ServerError::websocket("test").source().is_none());
        assert!(ServerError::internal("test").source().is_none());
    }

    #[test]
    fn rate_limit_has_source_field() {
        let err = ServerError::RateLimit {
            source: "192.168.1.1".to_string(),
            retry_after: 60,
        };
        if let ServerError::RateLimit {
            source,
            retry_after,
        } = err
        {
            assert_eq!(source, "192.168.1.1");
            assert_eq!(retry_after, 60);
        } else {
            panic!("Expected RateLimit variant");
        }
    }

    // ========================================================================
    // New authentication-related error tests (FR-028 to FR-031)
    // ========================================================================

    #[test]
    fn server_error_jwt_invalid_displays_correctly() {
        let err = ServerError::jwt_invalid("token has expired");
        assert_eq!(err.to_string(), "JWT validation failed: token has expired");
    }

    #[test]
    fn server_error_jwt_invalid_helper_works() {
        let err = ServerError::jwt_invalid("invalid signature");
        assert!(matches!(err, ServerError::JwtInvalid(_)));
        if let ServerError::JwtInvalid(msg) = err {
            assert_eq!(msg, "invalid signature");
        }
    }

    #[test]
    fn server_error_session_invalid_displays_correctly() {
        let err = ServerError::session_invalid("session not found");
        assert_eq!(err.to_string(), "session invalid: session not found");
    }

    #[test]
    fn server_error_session_invalid_helper_works() {
        let err = ServerError::session_invalid("session expired");
        assert!(matches!(err, ServerError::SessionInvalid(_)));
        if let ServerError::SessionInvalid(msg) = err {
            assert_eq!(msg, "session expired");
        }
    }

    #[test]
    fn server_error_session_capacity_exceeded_displays_correctly() {
        let err = ServerError::session_capacity_exceeded();
        assert_eq!(err.to_string(), "session capacity exceeded");
    }

    #[test]
    fn server_error_session_capacity_exceeded_helper_works() {
        let err = ServerError::session_capacity_exceeded();
        assert!(matches!(err, ServerError::SessionCapacityExceeded));
    }

    #[test]
    fn server_error_supabase_unavailable_displays_correctly() {
        let err = ServerError::supabase_unavailable("connection timeout");
        assert_eq!(err.to_string(), "Supabase unavailable: connection timeout");
    }

    #[test]
    fn server_error_supabase_unavailable_helper_works() {
        let err = ServerError::supabase_unavailable("network error");
        assert!(matches!(err, ServerError::SupabaseUnavailable(_)));
        if let ServerError::SupabaseUnavailable(msg) = err {
            assert_eq!(msg, "network error");
        }
    }

    // ========================================================================
    // Client/server error classification for new variants
    // ========================================================================

    #[test]
    fn jwt_invalid_is_client_error() {
        assert!(ServerError::jwt_invalid("expired").is_client_error());
        assert!(!ServerError::jwt_invalid("expired").is_server_error());
    }

    #[test]
    fn session_invalid_is_client_error() {
        assert!(ServerError::session_invalid("not found").is_client_error());
        assert!(!ServerError::session_invalid("not found").is_server_error());
    }

    #[test]
    fn session_capacity_exceeded_is_server_error() {
        assert!(ServerError::session_capacity_exceeded().is_server_error());
        assert!(!ServerError::session_capacity_exceeded().is_client_error());
    }

    #[test]
    fn supabase_unavailable_is_server_error() {
        assert!(ServerError::supabase_unavailable("timeout").is_server_error());
        assert!(!ServerError::supabase_unavailable("timeout").is_client_error());
    }

    // ========================================================================
    // HTTP status code tests
    // ========================================================================

    #[test]
    fn jwt_invalid_returns_401_unauthorized() {
        let err = ServerError::jwt_invalid("expired");
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn session_invalid_returns_401_unauthorized() {
        let err = ServerError::session_invalid("not found");
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn session_capacity_exceeded_returns_503_service_unavailable() {
        let err = ServerError::session_capacity_exceeded();
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn supabase_unavailable_returns_503_service_unavailable() {
        let err = ServerError::supabase_unavailable("timeout");
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn status_code_matches_for_all_existing_variants() {
        assert_eq!(
            ServerError::Config(ConfigError::missing("X")).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            ServerError::auth("test").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ServerError::validation("test").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ServerError::rate_limit("test", 60).status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            ServerError::websocket("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            ServerError::internal("test").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    // ========================================================================
    // IntoResponse tests
    // ========================================================================

    #[test]
    fn into_response_jwt_invalid_returns_401() {
        let err = ServerError::jwt_invalid("token expired");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn into_response_session_invalid_returns_401() {
        let err = ServerError::session_invalid("session not found");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn into_response_session_capacity_exceeded_returns_503() {
        let err = ServerError::session_capacity_exceeded();
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn into_response_supabase_unavailable_returns_503() {
        let err = ServerError::supabase_unavailable("connection failed");
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn into_response_rate_limit_includes_retry_after_header() {
        let err = ServerError::rate_limit("client", 120);
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            response.headers().get("Retry-After").unwrap().to_str().unwrap(),
            "120"
        );
    }

    #[test]
    fn server_error_source_returns_none_for_new_variants() {
        assert!(ServerError::jwt_invalid("test").source().is_none());
        assert!(ServerError::session_invalid("test").source().is_none());
        assert!(ServerError::session_capacity_exceeded().source().is_none());
        assert!(ServerError::supabase_unavailable("test").source().is_none());
    }

    #[test]
    fn new_variants_are_debug() {
        let jwt_err = ServerError::jwt_invalid("test");
        let session_err = ServerError::session_invalid("test");
        let capacity_err = ServerError::session_capacity_exceeded();
        let supabase_err = ServerError::supabase_unavailable("test");

        assert!(format!("{:?}", jwt_err).contains("JwtInvalid"));
        assert!(format!("{:?}", session_err).contains("SessionInvalid"));
        assert!(format!("{:?}", capacity_err).contains("SessionCapacityExceeded"));
        assert!(format!("{:?}", supabase_err).contains("SupabaseUnavailable"));
    }
}
