//! Server configuration module.
//!
//! Parses configuration from environment variables for the VibeTea server.
//!
//! # Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|----------|---------|-------------|
//! | `VIBETEA_PUBLIC_KEYS` | Yes* | - | Format: `source1:pubkey1,source2:pubkey2` |
//! | `VIBETEA_SUBSCRIBER_TOKEN` | Yes* | - | Auth token for Clients |
//! | `VIBETEA_SUPABASE_URL` | Yes* | - | URL of the Supabase project |
//! | `VIBETEA_SUPABASE_ANON_KEY` | Yes* | - | Supabase anon/public key for API calls |
//! | `PORT` | No | 8080 | HTTP server port |
//! | `VIBETEA_UNSAFE_NO_AUTH` | No | false | Disable all authentication (dev only) |
//!
//! *Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

use std::collections::HashMap;
use std::env;

use thiserror::Error;
use tracing::warn;

/// Default HTTP server port.
const DEFAULT_PORT: u16 = 8080;

/// Errors that can occur when parsing configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Required environment variable is missing.
    #[error("missing required environment variable: {0}")]
    MissingEnvVar(String),

    /// Environment variable has invalid format.
    #[error("invalid format for {var}: {message}")]
    InvalidFormat { var: String, message: String },

    /// Port number is invalid.
    #[error("invalid port number: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),

    /// Configuration validation failed.
    #[error("configuration validation failed: {0}")]
    ValidationError(String),
}

/// Server configuration parsed from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Map of source_id to base64-encoded Ed25519 public key.
    pub public_keys: HashMap<String, String>,

    /// Authentication token for subscriber clients.
    pub subscriber_token: Option<String>,

    /// HTTP server port.
    pub port: u16,

    /// When true, disables all authentication (development only).
    pub unsafe_no_auth: bool,

    /// URL of the Supabase project (e.g., `https://xxx.supabase.co`).
    pub supabase_url: Option<String>,

    /// Supabase anon/public key for API calls.
    pub supabase_anon_key: Option<String>,
}

impl Config {
    /// Parse configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - Required environment variables are missing (when `VIBETEA_UNSAFE_NO_AUTH` is not true)
    /// - Environment variables have invalid format
    /// - Port number is not a valid u16
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_server::config::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load config");
    /// println!("Server will listen on port {}", config.port);
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        let unsafe_no_auth = parse_bool_env("VIBETEA_UNSAFE_NO_AUTH");
        let port = parse_port()?;
        let public_keys = parse_public_keys()?;
        let subscriber_token = env::var("VIBETEA_SUBSCRIBER_TOKEN").ok();
        let supabase_url = env::var("VIBETEA_SUPABASE_URL").ok();
        let supabase_anon_key = env::var("VIBETEA_SUPABASE_ANON_KEY").ok();

        let config = Self {
            public_keys,
            subscriber_token,
            port,
            unsafe_no_auth,
            supabase_url,
            supabase_anon_key,
        };

        config.validate()?;

        if config.unsafe_no_auth {
            warn!(
                "VIBETEA_UNSAFE_NO_AUTH is enabled - all authentication is disabled. \
                 Do not use in production!"
            );
        }

        Ok(config)
    }

    /// Validate the configuration.
    ///
    /// Ensures that either `unsafe_no_auth` is true, or all required authentication
    /// variables (`public_keys`, `subscriber_token`, `supabase_url`, and
    /// `supabase_anon_key`) are properly configured.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.unsafe_no_auth {
            return Ok(());
        }

        if self.public_keys.is_empty() {
            return Err(ConfigError::MissingEnvVar(
                "VIBETEA_PUBLIC_KEYS".to_string(),
            ));
        }

        if self.subscriber_token.is_none() {
            return Err(ConfigError::MissingEnvVar(
                "VIBETEA_SUBSCRIBER_TOKEN".to_string(),
            ));
        }

        if self.supabase_url.is_none() {
            return Err(ConfigError::MissingEnvVar(
                "VIBETEA_SUPABASE_URL".to_string(),
            ));
        }

        if self.supabase_anon_key.is_none() {
            return Err(ConfigError::MissingEnvVar(
                "VIBETEA_SUPABASE_ANON_KEY".to_string(),
            ));
        }

        Ok(())
    }
}

/// Parse a boolean environment variable.
///
/// Returns `true` if the variable is set to "true" (case-insensitive),
/// `false` otherwise.
fn parse_bool_env(name: &str) -> bool {
    env::var(name)
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Parse the PORT environment variable.
///
/// Returns the default port if not set.
fn parse_port() -> Result<u16, ConfigError> {
    match env::var("PORT") {
        Ok(port_str) => Ok(port_str.parse()?),
        Err(env::VarError::NotPresent) => Ok(DEFAULT_PORT),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::InvalidFormat {
            var: "PORT".to_string(),
            message: "contains invalid unicode".to_string(),
        }),
    }
}

/// Parse the VIBETEA_PUBLIC_KEYS environment variable.
///
/// Expected format: `source1:pubkey1,source2:pubkey2`
/// where pubkey is a base64-encoded Ed25519 public key.
fn parse_public_keys() -> Result<HashMap<String, String>, ConfigError> {
    let keys_str = match env::var("VIBETEA_PUBLIC_KEYS") {
        Ok(s) if !s.is_empty() => s,
        _ => return Ok(HashMap::new()),
    };

    let mut keys = HashMap::new();

    for pair in keys_str.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(ConfigError::InvalidFormat {
                var: "VIBETEA_PUBLIC_KEYS".to_string(),
                message: format!("expected 'source:pubkey' format, got '{}'", pair),
            });
        }

        let source_id = parts[0].trim();
        let pubkey = parts[1].trim();

        if source_id.is_empty() {
            return Err(ConfigError::InvalidFormat {
                var: "VIBETEA_PUBLIC_KEYS".to_string(),
                message: "source_id cannot be empty".to_string(),
            });
        }

        if pubkey.is_empty() {
            return Err(ConfigError::InvalidFormat {
                var: "VIBETEA_PUBLIC_KEYS".to_string(),
                message: format!("pubkey for source '{}' cannot be empty", source_id),
            });
        }

        keys.insert(source_id.to_string(), pubkey.to_string());
    }

    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    /// Helper to temporarily set environment variables for testing.
    struct EnvGuard {
        vars: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self { vars: Vec::new() }
        }

        fn set(&mut self, key: &str, value: &str) {
            let old_value = env::var(key).ok();
            self.vars.push((key.to_string(), old_value));
            env::set_var(key, value);
        }

        fn remove(&mut self, key: &str) {
            let old_value = env::var(key).ok();
            self.vars.push((key.to_string(), old_value));
            env::remove_var(key);
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.vars {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }
        }
    }

    #[test]
    #[serial]
    fn test_config_with_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_UNSAFE_NO_AUTH", "true");
        guard.remove("VIBETEA_PUBLIC_KEYS");
        guard.remove("VIBETEA_SUBSCRIBER_TOKEN");
        guard.remove("VIBETEA_SUPABASE_URL");
        guard.remove("VIBETEA_SUPABASE_ANON_KEY");
        guard.remove("PORT");

        let config = Config::from_env().expect("should parse config");
        assert!(config.unsafe_no_auth);
        assert!(config.public_keys.is_empty());
        assert!(config.subscriber_token.is_none());
        assert!(config.supabase_url.is_none());
        assert!(config.supabase_anon_key.is_none());
        assert_eq!(config.port, DEFAULT_PORT);
    }

    #[test]
    #[serial]
    fn test_config_with_auth_enabled() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_UNSAFE_NO_AUTH", "false");
        guard.set(
            "VIBETEA_PUBLIC_KEYS",
            "source1:cHVia2V5MQ==,source2:cHVia2V5Mg==",
        );
        guard.set("VIBETEA_SUBSCRIBER_TOKEN", "secret-token");
        guard.set("VIBETEA_SUPABASE_URL", "https://test.supabase.co");
        guard.set("VIBETEA_SUPABASE_ANON_KEY", "test-anon-key");
        guard.set("PORT", "9090");

        let config = Config::from_env().expect("should parse config");
        assert!(!config.unsafe_no_auth);
        assert_eq!(config.public_keys.len(), 2);
        assert_eq!(
            config.public_keys.get("source1"),
            Some(&"cHVia2V5MQ==".to_string())
        );
        assert_eq!(
            config.public_keys.get("source2"),
            Some(&"cHVia2V5Mg==".to_string())
        );
        assert_eq!(config.subscriber_token, Some("secret-token".to_string()));
        assert_eq!(
            config.supabase_url,
            Some("https://test.supabase.co".to_string())
        );
        assert_eq!(config.supabase_anon_key, Some("test-anon-key".to_string()));
        assert_eq!(config.port, 9090);
    }

    #[test]
    #[serial]
    fn test_config_missing_public_keys_without_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.remove("VIBETEA_UNSAFE_NO_AUTH");
        guard.remove("VIBETEA_PUBLIC_KEYS");
        guard.set("VIBETEA_SUBSCRIBER_TOKEN", "secret-token");
        guard.set("VIBETEA_SUPABASE_URL", "https://test.supabase.co");
        guard.set("VIBETEA_SUPABASE_ANON_KEY", "test-anon-key");

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::MissingEnvVar(ref v) if v == "VIBETEA_PUBLIC_KEYS"));
    }

    #[test]
    #[serial]
    fn test_config_missing_subscriber_token_without_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.remove("VIBETEA_UNSAFE_NO_AUTH");
        guard.set("VIBETEA_PUBLIC_KEYS", "source1:pubkey1");
        guard.remove("VIBETEA_SUBSCRIBER_TOKEN");
        guard.set("VIBETEA_SUPABASE_URL", "https://test.supabase.co");
        guard.set("VIBETEA_SUPABASE_ANON_KEY", "test-anon-key");

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ConfigError::MissingEnvVar(ref v) if v == "VIBETEA_SUBSCRIBER_TOKEN")
        );
    }

    #[test]
    #[serial]
    fn test_parse_public_keys_valid() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_PUBLIC_KEYS", "source1:key1,source2:key2");

        let keys = parse_public_keys().expect("should parse keys");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys.get("source1"), Some(&"key1".to_string()));
        assert_eq!(keys.get("source2"), Some(&"key2".to_string()));
    }

    #[test]
    #[serial]
    fn test_parse_public_keys_with_whitespace() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_PUBLIC_KEYS", " source1 : key1 , source2 : key2 ");

        let keys = parse_public_keys().expect("should parse keys");
        assert_eq!(keys.len(), 2);
        assert_eq!(keys.get("source1"), Some(&"key1".to_string()));
        assert_eq!(keys.get("source2"), Some(&"key2".to_string()));
    }

    #[test]
    #[serial]
    fn test_parse_public_keys_invalid_format() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_PUBLIC_KEYS", "invalid-no-colon");

        let result = parse_public_keys();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ConfigError::InvalidFormat { var, .. } if var == "VIBETEA_PUBLIC_KEYS")
        );
    }

    #[test]
    #[serial]
    fn test_parse_public_keys_empty_source() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_PUBLIC_KEYS", ":key1");

        let result = parse_public_keys();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_parse_public_keys_empty_key() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_PUBLIC_KEYS", "source1:");

        let result = parse_public_keys();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_parse_bool_env_true() {
        let mut guard = EnvGuard::new();
        guard.set("TEST_BOOL", "true");
        assert!(parse_bool_env("TEST_BOOL"));

        guard.set("TEST_BOOL", "TRUE");
        assert!(parse_bool_env("TEST_BOOL"));

        guard.set("TEST_BOOL", "True");
        assert!(parse_bool_env("TEST_BOOL"));
    }

    #[test]
    #[serial]
    fn test_parse_bool_env_false() {
        let mut guard = EnvGuard::new();
        guard.set("TEST_BOOL", "false");
        assert!(!parse_bool_env("TEST_BOOL"));

        guard.set("TEST_BOOL", "anything-else");
        assert!(!parse_bool_env("TEST_BOOL"));

        guard.remove("TEST_BOOL");
        assert!(!parse_bool_env("TEST_BOOL"));
    }

    #[test]
    #[serial]
    fn test_parse_port_default() {
        let mut guard = EnvGuard::new();
        guard.remove("PORT");

        let port = parse_port().expect("should parse port");
        assert_eq!(port, DEFAULT_PORT);
    }

    #[test]
    #[serial]
    fn test_parse_port_custom() {
        let mut guard = EnvGuard::new();
        guard.set("PORT", "3000");

        let port = parse_port().expect("should parse port");
        assert_eq!(port, 3000);
    }

    #[test]
    #[serial]
    fn test_parse_port_invalid() {
        let mut guard = EnvGuard::new();
        guard.set("PORT", "not-a-number");

        let result = parse_port();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidPort(_)));
    }

    #[test]
    #[serial]
    fn test_parse_port_out_of_range() {
        let mut guard = EnvGuard::new();
        guard.set("PORT", "99999");

        let result = parse_port();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_config_missing_supabase_url_without_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.remove("VIBETEA_UNSAFE_NO_AUTH");
        guard.set("VIBETEA_PUBLIC_KEYS", "source1:pubkey1");
        guard.set("VIBETEA_SUBSCRIBER_TOKEN", "secret-token");
        guard.remove("VIBETEA_SUPABASE_URL");
        guard.set("VIBETEA_SUPABASE_ANON_KEY", "test-anon-key");

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::MissingEnvVar(ref v) if v == "VIBETEA_SUPABASE_URL"));
    }

    #[test]
    #[serial]
    fn test_config_missing_supabase_anon_key_without_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.remove("VIBETEA_UNSAFE_NO_AUTH");
        guard.set("VIBETEA_PUBLIC_KEYS", "source1:pubkey1");
        guard.set("VIBETEA_SUBSCRIBER_TOKEN", "secret-token");
        guard.set("VIBETEA_SUPABASE_URL", "https://test.supabase.co");
        guard.remove("VIBETEA_SUPABASE_ANON_KEY");

        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, ConfigError::MissingEnvVar(ref v) if v == "VIBETEA_SUPABASE_ANON_KEY")
        );
    }

    #[test]
    #[serial]
    fn test_config_supabase_vars_optional_with_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_UNSAFE_NO_AUTH", "true");
        guard.remove("VIBETEA_PUBLIC_KEYS");
        guard.remove("VIBETEA_SUBSCRIBER_TOKEN");
        guard.remove("VIBETEA_SUPABASE_URL");
        guard.remove("VIBETEA_SUPABASE_ANON_KEY");

        let config = Config::from_env().expect("should parse config");
        assert!(config.unsafe_no_auth);
        assert!(config.supabase_url.is_none());
        assert!(config.supabase_anon_key.is_none());
    }

    #[test]
    #[serial]
    fn test_config_supabase_vars_can_be_set_with_unsafe_no_auth() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_UNSAFE_NO_AUTH", "true");
        guard.remove("VIBETEA_PUBLIC_KEYS");
        guard.remove("VIBETEA_SUBSCRIBER_TOKEN");
        guard.set("VIBETEA_SUPABASE_URL", "https://test.supabase.co");
        guard.set("VIBETEA_SUPABASE_ANON_KEY", "test-anon-key");

        let config = Config::from_env().expect("should parse config");
        assert!(config.unsafe_no_auth);
        assert_eq!(
            config.supabase_url,
            Some("https://test.supabase.co".to_string())
        );
        assert_eq!(config.supabase_anon_key, Some("test-anon-key".to_string()));
    }
}
