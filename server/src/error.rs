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
/// - **Validation errors**: Malformed events or invalid request data
/// - **Rate limiting**: Client exceeded allowed request rate
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
            Self::Validation(msg) => write!(f, "validation error: {msg}"),
            Self::RateLimit { source, retry_after } => {
                write!(
                    f,
                    "rate limit exceeded for {source}, retry after {retry_after} seconds"
                )
            }
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

    /// Returns `true` if this error indicates a client-side problem.
    ///
    /// Client errors are those where the client made an invalid request,
    /// as opposed to server-side failures.
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Auth(_) | Self::Validation(_) | Self::RateLimit { .. }
        )
    }

    /// Returns `true` if this error indicates a server-side problem.
    pub fn is_server_error(&self) -> bool {
        matches!(self, Self::Internal(_) | Self::Config(_))
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
        if let ServerError::RateLimit { source, retry_after } = err {
            assert_eq!(source, "192.168.1.1");
            assert_eq!(retry_after, 60);
        } else {
            panic!("Expected RateLimit variant");
        }
    }
}
