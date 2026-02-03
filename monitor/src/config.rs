//! Configuration module for VibeTea Monitor.
//!
//! This module handles parsing configuration from environment variables.
//!
//! # Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|----------|---------|-------------|
//! | `VIBETEA_SERVER_URL` | Yes | - | Server URL (e.g., `https://vibetea.fly.dev`) |
//! | `VIBETEA_SOURCE_ID` | No | hostname | Monitor identifier (must match key registration) |
//! | `VIBETEA_KEY_PATH` | No | `~/.vibetea` | Directory containing `key.priv` and `key.pub` |
//! | `VIBETEA_CLAUDE_DIR` | No | `~/.claude` | Claude Code directory |
//! | `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |
//! | `VIBETEA_BASENAME_ALLOWLIST` | No | (all) | Comma-separated extensions to allow |
//! | `VIBETEA_SUPABASE_URL` | No | - | Supabase edge function URL (enables persistence) |
//! | `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS` | No | 60 | Seconds between batch submissions |
//! | `VIBETEA_SUPABASE_RETRY_LIMIT` | No | 3 | Max retry attempts (1-10) |
//!
//! # Example
//!
//! ```no_run
//! use vibetea_monitor::config::Config;
//!
//! let config = Config::from_env().expect("Failed to load configuration");
//! println!("Server URL: {}", config.server_url);
//! ```

use std::env;
use std::path::PathBuf;

use directories::BaseDirs;
use thiserror::Error;

/// Default event buffer capacity.
const DEFAULT_BUFFER_SIZE: usize = 1000;

/// Default key directory name relative to home.
const DEFAULT_KEY_DIR: &str = ".vibetea";

/// Default Claude Code directory name relative to home.
const DEFAULT_CLAUDE_DIR: &str = ".claude";

/// Default batch interval for persistence (in seconds).
const DEFAULT_BATCH_INTERVAL_SECS: u64 = 60;

/// Default retry limit for persistence batch submissions.
const DEFAULT_RETRY_LIMIT: u8 = 3;

/// Minimum allowed retry limit.
const MIN_RETRY_LIMIT: u8 = 1;

/// Maximum allowed retry limit.
const MAX_RETRY_LIMIT: u8 = 10;

/// Errors that can occur during configuration parsing.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Required environment variable is missing.
    #[error("missing required environment variable: {0}")]
    MissingEnvVar(String),

    /// Environment variable has an invalid value.
    #[error("invalid value for {key}: {message}")]
    InvalidValue { key: String, message: String },

    /// Failed to determine home directory.
    #[error("failed to determine home directory")]
    NoHomeDirectory,
}

/// Configuration for optional Supabase persistence.
///
/// When enabled, the monitor batches events and sends them to a Supabase edge function
/// for historic data storage and activity heatmap visualization.
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Supabase edge function base URL (e.g., `https://xyz.supabase.co/functions/v1`).
    pub supabase_url: String,

    /// Seconds between batch submissions. Events are also sent immediately when
    /// the batch reaches 1000 events.
    pub batch_interval_secs: u64,

    /// Maximum consecutive retry attempts before dropping a failed batch.
    /// Must be between 1 and 10 (inclusive).
    pub retry_limit: u8,
}

/// Configuration for the VibeTea Monitor.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server URL for the VibeTea server (e.g., `https://vibetea.fly.dev`).
    pub server_url: String,

    /// Monitor identifier, must match the key registration on the server.
    pub source_id: String,

    /// Path to the directory containing `key.priv` and `key.pub`.
    pub key_path: PathBuf,

    /// Path to the Claude Code directory to watch.
    pub claude_dir: PathBuf,

    /// Capacity of the event buffer.
    pub buffer_size: usize,

    /// Optional allowlist of file basename patterns to watch.
    /// If `None`, all files are watched.
    pub basename_allowlist: Option<Vec<String>>,

    /// Optional persistence configuration for Supabase.
    /// If `None`, persistence is disabled and events are only sent in real-time.
    pub persistence: Option<PersistenceConfig>,
}

impl Config {
    /// Creates a new `Config` by parsing environment variables.
    ///
    /// # Errors
    ///
    /// Returns a `ConfigError` if:
    /// - `VIBETEA_SERVER_URL` is not set
    /// - `VIBETEA_BUFFER_SIZE` is set but cannot be parsed as a positive integer
    /// - The home directory cannot be determined (needed for default paths)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::config::Config;
    ///
    /// std::env::set_var("VIBETEA_SERVER_URL", "https://vibetea.fly.dev");
    /// let config = Config::from_env().unwrap();
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        let base_dirs = BaseDirs::new().ok_or(ConfigError::NoHomeDirectory)?;
        let home_dir = base_dirs.home_dir();

        // Required: VIBETEA_SERVER_URL
        let server_url = env::var("VIBETEA_SERVER_URL")
            .map_err(|_| ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".to_string()))?;

        // Optional: VIBETEA_SOURCE_ID (default: hostname)
        let source_id = env::var("VIBETEA_SOURCE_ID").unwrap_or_else(|_| get_hostname());

        // Optional: VIBETEA_KEY_PATH (default: ~/.vibetea)
        let key_path = env::var("VIBETEA_KEY_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(DEFAULT_KEY_DIR));

        // Optional: VIBETEA_CLAUDE_DIR (default: ~/.claude)
        let claude_dir = env::var("VIBETEA_CLAUDE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(DEFAULT_CLAUDE_DIR));

        // Optional: VIBETEA_BUFFER_SIZE (default: 1000, must be > 0)
        let buffer_size = match env::var("VIBETEA_BUFFER_SIZE") {
            Ok(val) => {
                let size = val
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidValue {
                        key: "VIBETEA_BUFFER_SIZE".to_string(),
                        message: format!("expected positive integer, got '{val}'"),
                    })?;
                if size == 0 {
                    return Err(ConfigError::InvalidValue {
                        key: "VIBETEA_BUFFER_SIZE".to_string(),
                        message: "buffer size must be greater than 0".to_string(),
                    });
                }
                size
            }
            Err(_) => DEFAULT_BUFFER_SIZE,
        };

        // Optional: VIBETEA_BASENAME_ALLOWLIST (default: None = all files)
        let basename_allowlist = env::var("VIBETEA_BASENAME_ALLOWLIST").ok().map(|val| {
            val.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });

        // Optional: Persistence configuration (enabled if VIBETEA_SUPABASE_URL is set)
        let persistence = match env::var("VIBETEA_SUPABASE_URL") {
            Ok(supabase_url) => {
                // Parse batch interval (default: 60, must be >= 1)
                let batch_interval_secs = match env::var("VIBETEA_SUPABASE_BATCH_INTERVAL_SECS") {
                    Ok(val) => {
                        let secs = val.parse::<u64>().map_err(|_| ConfigError::InvalidValue {
                            key: "VIBETEA_SUPABASE_BATCH_INTERVAL_SECS".to_string(),
                            message: format!("expected positive integer, got '{val}'"),
                        })?;
                        if secs == 0 {
                            return Err(ConfigError::InvalidValue {
                                key: "VIBETEA_SUPABASE_BATCH_INTERVAL_SECS".to_string(),
                                message: "batch interval must be at least 1 second".to_string(),
                            });
                        }
                        secs
                    }
                    Err(_) => DEFAULT_BATCH_INTERVAL_SECS,
                };

                // Parse retry limit (default: 3, must be 1-10)
                let retry_limit = match env::var("VIBETEA_SUPABASE_RETRY_LIMIT") {
                    Ok(val) => {
                        let limit = val.parse::<u8>().map_err(|_| ConfigError::InvalidValue {
                            key: "VIBETEA_SUPABASE_RETRY_LIMIT".to_string(),
                            message: format!("expected integer 1-10, got '{val}'"),
                        })?;
                        if !(MIN_RETRY_LIMIT..=MAX_RETRY_LIMIT).contains(&limit) {
                            return Err(ConfigError::InvalidValue {
                                key: "VIBETEA_SUPABASE_RETRY_LIMIT".to_string(),
                                message: format!(
                                    "retry limit must be between {MIN_RETRY_LIMIT} and {MAX_RETRY_LIMIT}, got {limit}"
                                ),
                            });
                        }
                        limit
                    }
                    Err(_) => DEFAULT_RETRY_LIMIT,
                };

                Some(PersistenceConfig {
                    supabase_url,
                    batch_interval_secs,
                    retry_limit,
                })
            }
            Err(_) => None,
        };

        Ok(Self {
            server_url,
            source_id,
            key_path,
            claude_dir,
            buffer_size,
            basename_allowlist,
            persistence,
        })
    }
}

/// Gets the system hostname, falling back to "unknown" if it cannot be determined.
fn get_hostname() -> String {
    gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    /// Helper to run tests with isolated environment variables.
    /// Clears all VIBETEA_* vars before the test and restores them after.
    fn with_clean_env<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // Save and remove existing VIBETEA_* vars
        let saved_vars: Vec<(String, String)> = env::vars()
            .filter(|(k, _)| k.starts_with("VIBETEA_"))
            .collect();

        for (key, _) in &saved_vars {
            env::remove_var(key);
        }

        let result = f();

        // Restore saved vars
        for (key, value) in saved_vars {
            env::set_var(key, value);
        }

        result
    }

    #[test]
    #[serial]
    fn test_missing_server_url() {
        with_clean_env(|| {
            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ConfigError::MissingEnvVar(ref s) if s == "VIBETEA_SERVER_URL"));
        });
    }

    #[test]
    #[serial]
    fn test_minimal_config() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");

            let config = Config::from_env().expect("should parse minimal config");

            assert_eq!(config.server_url, "https://test.example.com");
            assert_eq!(config.buffer_size, DEFAULT_BUFFER_SIZE);
            assert!(config.basename_allowlist.is_none());

            // source_id should be hostname
            assert!(!config.source_id.is_empty());

            // Paths should end with default directory names
            assert!(config.key_path.ends_with(DEFAULT_KEY_DIR));
            assert!(config.claude_dir.ends_with(DEFAULT_CLAUDE_DIR));
        });
    }

    #[test]
    #[serial]
    fn test_full_config() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://vibetea.fly.dev");
            env::set_var("VIBETEA_SOURCE_ID", "my-monitor");
            env::set_var("VIBETEA_KEY_PATH", "/custom/keys");
            env::set_var("VIBETEA_CLAUDE_DIR", "/custom/claude");
            env::set_var("VIBETEA_BUFFER_SIZE", "500");
            env::set_var("VIBETEA_BASENAME_ALLOWLIST", "jsonl,json,log");

            let config = Config::from_env().expect("should parse full config");

            assert_eq!(config.server_url, "https://vibetea.fly.dev");
            assert_eq!(config.source_id, "my-monitor");
            assert_eq!(config.key_path, PathBuf::from("/custom/keys"));
            assert_eq!(config.claude_dir, PathBuf::from("/custom/claude"));
            assert_eq!(config.buffer_size, 500);
            assert_eq!(
                config.basename_allowlist,
                Some(vec![
                    "jsonl".to_string(),
                    "json".to_string(),
                    "log".to_string()
                ])
            );
        });
    }

    #[test]
    #[serial]
    fn test_invalid_buffer_size() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_BUFFER_SIZE", "not-a-number");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, .. } if key == "VIBETEA_BUFFER_SIZE"
            ));
        });
    }

    #[test]
    #[serial]
    fn test_zero_buffer_size_rejected() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_BUFFER_SIZE", "0");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, ref message }
                    if key == "VIBETEA_BUFFER_SIZE" && message.contains("greater than 0")
            ));
        });
    }

    #[test]
    #[serial]
    fn test_allowlist_with_whitespace() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_BASENAME_ALLOWLIST", " jsonl , json , log ");

            let config = Config::from_env().expect("should parse config with whitespace");

            assert_eq!(
                config.basename_allowlist,
                Some(vec![
                    "jsonl".to_string(),
                    "json".to_string(),
                    "log".to_string()
                ])
            );
        });
    }

    #[test]
    #[serial]
    fn test_allowlist_filters_empty_entries() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_BASENAME_ALLOWLIST", "jsonl,,json,,,log");

            let config = Config::from_env().expect("should filter empty entries");

            assert_eq!(
                config.basename_allowlist,
                Some(vec![
                    "jsonl".to_string(),
                    "json".to_string(),
                    "log".to_string()
                ])
            );
        });
    }

    #[test]
    fn test_get_hostname() {
        let hostname = get_hostname();
        // Hostname should be non-empty (even if it's "unknown")
        assert!(!hostname.is_empty());
    }

    #[test]
    #[serial]
    fn test_persistence_disabled_when_url_not_set() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");

            let config = Config::from_env().expect("should parse config");

            assert!(config.persistence.is_none());
        });
    }

    #[test]
    #[serial]
    fn test_persistence_enabled_with_defaults() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var(
                "VIBETEA_SUPABASE_URL",
                "https://xyz.supabase.co/functions/v1",
            );

            let config = Config::from_env().expect("should parse config with persistence");

            let persistence = config.persistence.expect("persistence should be enabled");
            assert_eq!(
                persistence.supabase_url,
                "https://xyz.supabase.co/functions/v1"
            );
            assert_eq!(persistence.batch_interval_secs, DEFAULT_BATCH_INTERVAL_SECS);
            assert_eq!(persistence.retry_limit, DEFAULT_RETRY_LIMIT);
        });
    }

    #[test]
    #[serial]
    fn test_persistence_with_custom_values() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var(
                "VIBETEA_SUPABASE_URL",
                "https://xyz.supabase.co/functions/v1",
            );
            env::set_var("VIBETEA_SUPABASE_BATCH_INTERVAL_SECS", "120");
            env::set_var("VIBETEA_SUPABASE_RETRY_LIMIT", "5");

            let config = Config::from_env().expect("should parse config with custom persistence");

            let persistence = config.persistence.expect("persistence should be enabled");
            assert_eq!(
                persistence.supabase_url,
                "https://xyz.supabase.co/functions/v1"
            );
            assert_eq!(persistence.batch_interval_secs, 120);
            assert_eq!(persistence.retry_limit, 5);
        });
    }

    #[test]
    #[serial]
    fn test_persistence_retry_limit_zero_rejected() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var(
                "VIBETEA_SUPABASE_URL",
                "https://xyz.supabase.co/functions/v1",
            );
            env::set_var("VIBETEA_SUPABASE_RETRY_LIMIT", "0");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, ref message }
                    if key == "VIBETEA_SUPABASE_RETRY_LIMIT"
                    && message.contains("between 1 and 10")
            ));
        });
    }

    #[test]
    #[serial]
    fn test_persistence_retry_limit_eleven_rejected() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var(
                "VIBETEA_SUPABASE_URL",
                "https://xyz.supabase.co/functions/v1",
            );
            env::set_var("VIBETEA_SUPABASE_RETRY_LIMIT", "11");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, ref message }
                    if key == "VIBETEA_SUPABASE_RETRY_LIMIT"
                    && message.contains("between 1 and 10")
            ));
        });
    }

    #[test]
    #[serial]
    fn test_persistence_batch_interval_zero_rejected() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var(
                "VIBETEA_SUPABASE_URL",
                "https://xyz.supabase.co/functions/v1",
            );
            env::set_var("VIBETEA_SUPABASE_BATCH_INTERVAL_SECS", "0");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, ref message }
                    if key == "VIBETEA_SUPABASE_BATCH_INTERVAL_SECS"
                    && message.contains("at least 1 second")
            ));
        });
    }
}
