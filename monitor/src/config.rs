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

        // Optional: VIBETEA_BUFFER_SIZE (default: 1000)
        let buffer_size = match env::var("VIBETEA_BUFFER_SIZE") {
            Ok(val) => val.parse::<usize>().map_err(|_| ConfigError::InvalidValue {
                key: "VIBETEA_BUFFER_SIZE".to_string(),
                message: format!("expected positive integer, got '{val}'"),
            })?,
            Err(_) => DEFAULT_BUFFER_SIZE,
        };

        // Optional: VIBETEA_BASENAME_ALLOWLIST (default: None = all files)
        let basename_allowlist = env::var("VIBETEA_BASENAME_ALLOWLIST").ok().map(|val| {
            val.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });

        Ok(Self {
            server_url,
            source_id,
            key_path,
            claude_dir,
            buffer_size,
            basename_allowlist,
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
    fn test_missing_server_url() {
        with_clean_env(|| {
            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(err, ConfigError::MissingEnvVar(ref s) if s == "VIBETEA_SERVER_URL"));
        });
    }

    #[test]
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
}
