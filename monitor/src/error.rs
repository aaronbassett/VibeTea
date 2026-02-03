//! Error types for the VibeTea Monitor.
//!
//! This module defines the error types used throughout the monitor crate,
//! providing structured error handling with clear, human-readable messages.

use thiserror::Error;

use crate::config::ConfigError;

/// Errors that can occur during monitor operations.
///
/// This is the primary error type for the monitor crate, encompassing all
/// possible failure modes.
///
/// # Examples
///
/// ```ignore
/// use vibetea_monitor::error::MonitorError;
///
/// fn load_config() -> Result<(), MonitorError> {
///     let contents = std::fs::read_to_string("config.json")?;
///     let config: Config = serde_json::from_str(&contents)?;
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum MonitorError {
    /// Configuration-related error.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// File system I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing or serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP request error.
    ///
    /// This variant wraps HTTP-related errors that occur during communication
    /// with the VibeTea server.
    #[error("HTTP error: {0}")]
    Http(String),

    /// Cryptographic operation error.
    ///
    /// This variant covers errors related to key loading, signature generation,
    /// and signature verification.
    #[error("cryptographic error: {0}")]
    Crypto(String),

    /// File watching error.
    ///
    /// This variant covers errors from the file system watcher used to monitor
    /// Claude Code session files.
    #[error("file watch error: {0}")]
    Watch(String),

    /// TUI-related error.
    #[error("TUI error: {0}")]
    Tui(#[from] TuiError),
}

/// Errors that can occur during TUI operation.
#[derive(Error, Debug)]
pub enum TuiError {
    /// Terminal initialization failed.
    #[error("failed to initialize terminal: {0}")]
    TerminalInit(#[source] std::io::Error),

    /// Terminal rendering failed.
    #[error("render error: {0}")]
    Render(#[source] std::io::Error),

    /// Event handling error.
    #[error("event error: {0}")]
    Event(String),

    /// Terminal size is below minimum (80x24).
    #[error("terminal too small: {width}x{height} (minimum 80x24)")]
    TerminalTooSmall { width: u16, height: u16 },

    /// Setup validation failed.
    #[error("setup error: {0}")]
    Setup(#[from] SetupError),
}

/// Errors that can occur during setup form validation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum SetupError {
    /// Session name is empty.
    #[error("session name cannot be empty")]
    EmptySessionName,

    /// Session name is too long (max 64 characters).
    #[error("session name too long ({length}/64 characters)")]
    SessionNameTooLong { length: usize },

    /// Session name contains invalid characters.
    #[error("session name can only contain letters, numbers, hyphens, and underscores")]
    InvalidSessionNameCharacters,

    /// Session name starts with invalid character.
    #[error("session name cannot start with hyphen or underscore")]
    InvalidSessionNameStart,

    /// Key loading failed.
    #[error("failed to load keys: {0}")]
    KeyLoadFailed(String),

    /// Key generation failed.
    #[error("failed to generate keys: {0}")]
    KeyGenerationFailed(String),
}

/// A specialized `Result` type for monitor operations.
pub type Result<T> = std::result::Result<T, MonitorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_missing_env_var_display() {
        let err = ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".to_string());
        assert_eq!(
            err.to_string(),
            "missing required environment variable: VIBETEA_SERVER_URL"
        );
    }

    #[test]
    fn config_error_invalid_value_display() {
        let err = ConfigError::InvalidValue {
            key: "VIBETEA_BUFFER_SIZE".to_string(),
            message: "expected positive integer".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid value for VIBETEA_BUFFER_SIZE: expected positive integer"
        );
    }

    #[test]
    fn config_error_no_home_directory_display() {
        let err = ConfigError::NoHomeDirectory;
        assert_eq!(err.to_string(), "failed to determine home directory");
    }

    #[test]
    fn monitor_error_config_display() {
        let config_err = ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".to_string());
        let err = MonitorError::Config(config_err);
        assert_eq!(
            err.to_string(),
            "configuration error: missing required environment variable: VIBETEA_SERVER_URL"
        );
    }

    #[test]
    fn monitor_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: MonitorError = io_err.into();
        assert!(matches!(err, MonitorError::Io(_)));
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn monitor_error_json_conversion() {
        let json_str = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let err: MonitorError = json_err.into();
        assert!(matches!(err, MonitorError::Json(_)));
        assert!(err.to_string().contains("JSON error"));
    }

    #[test]
    fn monitor_error_http_display() {
        let err = MonitorError::Http("connection refused".to_string());
        assert_eq!(err.to_string(), "HTTP error: connection refused");
    }

    #[test]
    fn monitor_error_crypto_display() {
        let err = MonitorError::Crypto("invalid key format".to_string());
        assert_eq!(err.to_string(), "cryptographic error: invalid key format");
    }

    #[test]
    fn monitor_error_watch_display() {
        let err = MonitorError::Watch("inotify limit reached".to_string());
        assert_eq!(err.to_string(), "file watch error: inotify limit reached");
    }

    #[test]
    fn config_error_to_monitor_error_conversion() {
        let config_err = ConfigError::MissingEnvVar("VIBETEA_API_KEY".to_string());
        let monitor_err: MonitorError = config_err.into();
        assert!(matches!(monitor_err, MonitorError::Config(_)));
    }

    #[test]
    fn result_type_alias_works() {
        fn example_function() -> Result<i32> {
            Ok(42)
        }

        fn example_error_function() -> Result<i32> {
            Err(MonitorError::Http("test error".to_string()))
        }

        assert!(example_function().is_ok());
        assert!(example_error_function().is_err());
    }

    #[test]
    fn error_source_chain() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let monitor_err: MonitorError = io_err.into();

        // Verify the error source chain is preserved
        let source = monitor_err.source();
        assert!(source.is_some());
    }

    // TuiError tests

    #[test]
    fn tui_error_terminal_init_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "raw mode failed");
        let err = TuiError::TerminalInit(io_err);
        assert_eq!(
            err.to_string(),
            "failed to initialize terminal: raw mode failed"
        );
    }

    #[test]
    fn tui_error_render_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "write failed");
        let err = TuiError::Render(io_err);
        assert_eq!(err.to_string(), "render error: write failed");
    }

    #[test]
    fn tui_error_event_display() {
        let err = TuiError::Event("poll timeout".to_string());
        assert_eq!(err.to_string(), "event error: poll timeout");
    }

    #[test]
    fn tui_error_terminal_too_small_display() {
        let err = TuiError::TerminalTooSmall {
            width: 60,
            height: 20,
        };
        assert_eq!(err.to_string(), "terminal too small: 60x20 (minimum 80x24)");
    }

    #[test]
    fn tui_error_setup_display() {
        let setup_err = SetupError::EmptySessionName;
        let err = TuiError::Setup(setup_err);
        assert_eq!(err.to_string(), "setup error: session name cannot be empty");
    }

    #[test]
    fn tui_error_source_chain() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "raw mode failed");
        let err = TuiError::TerminalInit(io_err);

        let source = err.source();
        assert!(source.is_some());
    }

    #[test]
    fn tui_error_to_monitor_error_conversion() {
        let tui_err = TuiError::Event("test".to_string());
        let monitor_err: MonitorError = tui_err.into();
        assert!(matches!(monitor_err, MonitorError::Tui(_)));
    }

    // SetupError tests

    #[test]
    fn setup_error_empty_session_name_display() {
        let err = SetupError::EmptySessionName;
        assert_eq!(err.to_string(), "session name cannot be empty");
    }

    #[test]
    fn setup_error_session_name_too_long_display() {
        let err = SetupError::SessionNameTooLong { length: 100 };
        assert_eq!(err.to_string(), "session name too long (100/64 characters)");
    }

    #[test]
    fn setup_error_invalid_session_name_characters_display() {
        let err = SetupError::InvalidSessionNameCharacters;
        assert_eq!(
            err.to_string(),
            "session name can only contain letters, numbers, hyphens, and underscores"
        );
    }

    #[test]
    fn setup_error_invalid_session_name_start_display() {
        let err = SetupError::InvalidSessionNameStart;
        assert_eq!(
            err.to_string(),
            "session name cannot start with hyphen or underscore"
        );
    }

    #[test]
    fn setup_error_key_load_failed_display() {
        let err = SetupError::KeyLoadFailed("file not found".to_string());
        assert_eq!(err.to_string(), "failed to load keys: file not found");
    }

    #[test]
    fn setup_error_key_generation_failed_display() {
        let err = SetupError::KeyGenerationFailed("entropy exhausted".to_string());
        assert_eq!(
            err.to_string(),
            "failed to generate keys: entropy exhausted"
        );
    }

    #[test]
    fn setup_error_clone() {
        let err = SetupError::SessionNameTooLong { length: 65 };
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn setup_error_equality() {
        let err1 = SetupError::EmptySessionName;
        let err2 = SetupError::EmptySessionName;
        let err3 = SetupError::InvalidSessionNameStart;

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn setup_error_to_tui_error_conversion() {
        let setup_err = SetupError::EmptySessionName;
        let tui_err: TuiError = setup_err.into();
        assert!(matches!(tui_err, TuiError::Setup(_)));
    }

    #[test]
    fn monitor_error_tui_display() {
        let tui_err = TuiError::Event("timeout".to_string());
        let err = MonitorError::Tui(tui_err);
        assert_eq!(err.to_string(), "TUI error: event error: timeout");
    }
}
