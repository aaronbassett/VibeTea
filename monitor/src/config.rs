//! Configuration module for VibeTea Monitor.
//!
//! This module handles parsing configuration from environment variables and
//! provides session tracking with LRU eviction.
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
//! | `VIBETEA_MAX_SESSIONS` | No | 1000 | Maximum tracked sessions (LRU eviction) |
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
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;
use directories::BaseDirs;
use lru::LruCache;
use thiserror::Error;
use tracing::warn;

/// Default event buffer capacity.
const DEFAULT_BUFFER_SIZE: usize = 1000;

/// Default key directory name relative to home.
const DEFAULT_KEY_DIR: &str = ".vibetea";

/// Default Claude Code directory name relative to home.
const DEFAULT_CLAUDE_DIR: &str = ".claude";

/// Default maximum number of tracked sessions.
/// When this limit is reached, the least recently used session is evicted.
pub const MAX_TRACKED_SESSIONS: usize = 1000;

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

    /// Maximum number of sessions to track simultaneously.
    /// When this limit is reached, the least recently used session is evicted.
    pub max_sessions: usize,
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

        // Optional: VIBETEA_MAX_SESSIONS (default: MAX_TRACKED_SESSIONS, must be > 0)
        let max_sessions = match env::var("VIBETEA_MAX_SESSIONS") {
            Ok(val) => {
                let size = val
                    .parse::<usize>()
                    .map_err(|_| ConfigError::InvalidValue {
                        key: "VIBETEA_MAX_SESSIONS".to_string(),
                        message: format!("expected positive integer, got '{val}'"),
                    })?;
                if size == 0 {
                    return Err(ConfigError::InvalidValue {
                        key: "VIBETEA_MAX_SESSIONS".to_string(),
                        message: "max sessions must be greater than 0".to_string(),
                    });
                }
                size
            }
            Err(_) => MAX_TRACKED_SESSIONS,
        };

        Ok(Self {
            server_url,
            source_id,
            key_path,
            claude_dir,
            buffer_size,
            basename_allowlist,
            max_sessions,
        })
    }
}

/// Gets the system hostname, falling back to "unknown" if it cannot be determined.
fn get_hostname() -> String {
    gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Metrics for session tracking operations.
///
/// These metrics track the number of sessions added, accessed, and evicted
/// due to the session limit being reached.
#[derive(Debug, Default)]
pub struct SessionMetrics {
    /// Total number of sessions that have been tracked.
    sessions_added: AtomicU64,
    /// Total number of session accesses (get or touch operations).
    sessions_accessed: AtomicU64,
    /// Number of sessions evicted due to limit being reached.
    sessions_evicted: AtomicU64,
}

impl SessionMetrics {
    /// Creates a new `SessionMetrics` with all counters at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the total number of sessions that have been added.
    pub fn sessions_added(&self) -> u64 {
        self.sessions_added.load(Ordering::Relaxed)
    }

    /// Returns the total number of session accesses.
    pub fn sessions_accessed(&self) -> u64 {
        self.sessions_accessed.load(Ordering::Relaxed)
    }

    /// Returns the number of sessions evicted due to limit.
    pub fn sessions_evicted(&self) -> u64 {
        self.sessions_evicted.load(Ordering::Relaxed)
    }

    /// Increments the sessions added counter.
    fn record_add(&self) {
        self.sessions_added.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the sessions accessed counter.
    fn record_access(&self) {
        self.sessions_accessed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the sessions evicted counter.
    fn record_eviction(&self) {
        self.sessions_evicted.fetch_add(1, Ordering::Relaxed);
    }
}

/// Metadata associated with a tracked session.
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Timestamp when the session was first tracked (Unix timestamp in milliseconds).
    pub created_at: i64,
    /// Timestamp when the session was last accessed (Unix timestamp in milliseconds).
    pub last_accessed_at: i64,
}

impl SessionData {
    /// Creates new session data with the current timestamp.
    pub fn new() -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            created_at: now,
            last_accessed_at: now,
        }
    }

    /// Updates the last accessed timestamp to now.
    pub fn touch(&mut self) {
        self.last_accessed_at = Utc::now().timestamp_millis();
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks active sessions with LRU eviction when the limit is reached.
///
/// The `SessionTracker` maintains a bounded cache of session IDs. When the
/// maximum number of sessions is reached, the least recently used session
/// is evicted to make room for new sessions.
///
/// # Example
///
/// ```
/// use vibetea_monitor::config::SessionTracker;
///
/// let mut tracker = SessionTracker::new(3);
///
/// // Track some sessions
/// tracker.track("session-1");
/// tracker.track("session-2");
/// tracker.track("session-3");
///
/// assert!(tracker.contains("session-1"));
/// assert_eq!(tracker.len(), 3);
///
/// // Adding a fourth session evicts the least recently used
/// tracker.track("session-4");
/// assert!(!tracker.contains("session-1")); // Evicted
/// assert!(tracker.contains("session-4"));
/// ```
pub struct SessionTracker {
    /// LRU cache mapping session IDs to their metadata.
    cache: LruCache<String, SessionData>,
    /// Metrics for tracking session operations.
    metrics: SessionMetrics,
}

impl SessionTracker {
    /// Creates a new `SessionTracker` with the specified capacity.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).expect("SessionTracker capacity must be > 0");
        Self {
            cache: LruCache::new(cap),
            metrics: SessionMetrics::new(),
        }
    }

    /// Creates a new `SessionTracker` from the given configuration.
    pub fn from_config(config: &Config) -> Self {
        Self::new(config.max_sessions)
    }

    /// Tracks a session by its ID.
    ///
    /// If the session already exists, it is marked as recently used.
    /// If the session is new and the limit is reached, the least recently
    /// used session is evicted and a warning is logged.
    ///
    /// Returns `true` if this is a new session, `false` if it already existed.
    pub fn track(&mut self, session_id: &str) -> bool {
        // Check if session already exists
        if self.cache.contains(session_id) {
            // Update last accessed time and promote in LRU
            if let Some(data) = self.cache.get_mut(session_id) {
                data.touch();
            }
            self.metrics.record_access();
            return false;
        }

        // Check if we need to evict
        if self.cache.len() >= self.cache.cap().get() {
            // Pop the least recently used session
            if let Some((evicted_id, evicted_data)) = self.cache.pop_lru() {
                self.metrics.record_eviction();
                warn!(
                    session_id = %evicted_id,
                    created_at = evicted_data.created_at,
                    last_accessed_at = evicted_data.last_accessed_at,
                    current_count = self.cache.len(),
                    max_sessions = self.cache.cap().get(),
                    "Session evicted due to limit reached"
                );
            }
        }

        // Add the new session
        self.cache.put(session_id.to_string(), SessionData::new());
        self.metrics.record_add();
        true
    }

    /// Checks if a session is currently being tracked.
    ///
    /// This does NOT update the LRU order.
    pub fn contains(&self, session_id: &str) -> bool {
        self.cache.contains(session_id)
    }

    /// Gets the session data for a session ID, if it exists.
    ///
    /// This updates the LRU order, marking the session as recently used.
    pub fn get(&mut self, session_id: &str) -> Option<&SessionData> {
        self.metrics.record_access();
        self.cache.get(session_id)
    }

    /// Gets the session data for a session ID without updating LRU order.
    ///
    /// Use this when you need to inspect session data without affecting
    /// the eviction order.
    pub fn peek(&self, session_id: &str) -> Option<&SessionData> {
        self.cache.peek(session_id)
    }

    /// Removes a session from tracking.
    ///
    /// Returns the session data if the session was being tracked.
    pub fn remove(&mut self, session_id: &str) -> Option<SessionData> {
        self.cache.pop(session_id)
    }

    /// Returns the number of sessions currently being tracked.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if no sessions are being tracked.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Returns the maximum capacity of the tracker.
    pub fn capacity(&self) -> usize {
        self.cache.cap().get()
    }

    /// Returns a reference to the session metrics.
    pub fn metrics(&self) -> &SessionMetrics {
        &self.metrics
    }

    /// Clears all tracked sessions.
    ///
    /// This does not reset the metrics.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns an iterator over all tracked session IDs.
    ///
    /// Sessions are yielded in LRU order (least recently used first).
    pub fn session_ids(&self) -> impl Iterator<Item = &str> {
        self.cache.iter().map(|(id, _)| id.as_str())
    }
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

    // ==================== SessionTracker Tests ====================

    #[test]
    fn test_session_tracker_new() {
        let tracker = SessionTracker::new(100);
        assert_eq!(tracker.capacity(), 100);
        assert_eq!(tracker.len(), 0);
        assert!(tracker.is_empty());
    }

    #[test]
    #[should_panic(expected = "SessionTracker capacity must be > 0")]
    fn test_session_tracker_zero_capacity_panics() {
        let _ = SessionTracker::new(0);
    }

    #[test]
    fn test_session_tracker_track_new_session() {
        let mut tracker = SessionTracker::new(10);

        let is_new = tracker.track("session-1");
        assert!(is_new);
        assert_eq!(tracker.len(), 1);
        assert!(tracker.contains("session-1"));
        assert_eq!(tracker.metrics().sessions_added(), 1);
    }

    #[test]
    fn test_session_tracker_track_existing_session() {
        let mut tracker = SessionTracker::new(10);

        tracker.track("session-1");
        let is_new = tracker.track("session-1");

        assert!(!is_new); // Not a new session
        assert_eq!(tracker.len(), 1);
        assert_eq!(tracker.metrics().sessions_added(), 1);
        assert_eq!(tracker.metrics().sessions_accessed(), 1);
    }

    #[test]
    fn test_session_tracker_lru_eviction() {
        let mut tracker = SessionTracker::new(3);

        // Fill the tracker
        tracker.track("session-1");
        tracker.track("session-2");
        tracker.track("session-3");
        assert_eq!(tracker.len(), 3);

        // Adding a 4th session should evict the oldest (session-1)
        tracker.track("session-4");
        assert_eq!(tracker.len(), 3);
        assert!(!tracker.contains("session-1")); // Evicted
        assert!(tracker.contains("session-2"));
        assert!(tracker.contains("session-3"));
        assert!(tracker.contains("session-4"));
        assert_eq!(tracker.metrics().sessions_evicted(), 1);
    }

    #[test]
    fn test_session_tracker_lru_order_updated_on_access() {
        let mut tracker = SessionTracker::new(3);

        // Fill the tracker
        tracker.track("session-1");
        tracker.track("session-2");
        tracker.track("session-3");

        // Access session-1 to make it most recently used
        tracker.track("session-1");

        // Now add session-4, which should evict session-2 (now the oldest)
        tracker.track("session-4");
        assert!(tracker.contains("session-1")); // Still here, was accessed
        assert!(!tracker.contains("session-2")); // Evicted
        assert!(tracker.contains("session-3"));
        assert!(tracker.contains("session-4"));
    }

    #[test]
    fn test_session_tracker_get_updates_lru() {
        let mut tracker = SessionTracker::new(3);

        tracker.track("session-1");
        tracker.track("session-2");
        tracker.track("session-3");

        // Get session-1 to make it most recently used
        let data = tracker.get("session-1");
        assert!(data.is_some());

        // Add session-4, should evict session-2
        tracker.track("session-4");
        assert!(tracker.contains("session-1"));
        assert!(!tracker.contains("session-2"));
    }

    #[test]
    fn test_session_tracker_peek_does_not_update_lru() {
        let mut tracker = SessionTracker::new(3);

        tracker.track("session-1");
        tracker.track("session-2");
        tracker.track("session-3");

        // Peek at session-1 (should NOT update LRU order)
        let data = tracker.peek("session-1");
        assert!(data.is_some());

        // Add session-4, should evict session-1 (still oldest)
        tracker.track("session-4");
        assert!(!tracker.contains("session-1")); // Evicted because peek didn't update LRU
        assert!(tracker.contains("session-2"));
    }

    #[test]
    fn test_session_tracker_remove() {
        let mut tracker = SessionTracker::new(10);

        tracker.track("session-1");
        tracker.track("session-2");
        assert_eq!(tracker.len(), 2);

        let removed = tracker.remove("session-1");
        assert!(removed.is_some());
        assert_eq!(tracker.len(), 1);
        assert!(!tracker.contains("session-1"));
        assert!(tracker.contains("session-2"));

        // Removing non-existent session returns None
        let removed = tracker.remove("non-existent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_session_tracker_clear() {
        let mut tracker = SessionTracker::new(10);

        tracker.track("session-1");
        tracker.track("session-2");
        tracker.track("session-3");
        assert_eq!(tracker.len(), 3);

        tracker.clear();
        assert!(tracker.is_empty());
        assert_eq!(tracker.len(), 0);

        // Metrics should still be preserved
        assert_eq!(tracker.metrics().sessions_added(), 3);
    }

    #[test]
    fn test_session_tracker_session_ids() {
        let mut tracker = SessionTracker::new(10);

        tracker.track("session-1");
        tracker.track("session-2");
        tracker.track("session-3");

        let ids: Vec<&str> = tracker.session_ids().collect();
        assert_eq!(ids.len(), 3);
        // LRU order: least recently used first
        assert!(ids.contains(&"session-1"));
        assert!(ids.contains(&"session-2"));
        assert!(ids.contains(&"session-3"));
    }

    #[test]
    fn test_session_tracker_from_config() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_MAX_SESSIONS", "500");

            let config = Config::from_env().unwrap();
            let tracker = SessionTracker::from_config(&config);

            assert_eq!(tracker.capacity(), 500);
        });
    }

    #[test]
    fn test_session_data_timestamps() {
        let data = SessionData::new();
        assert!(data.created_at > 0);
        assert_eq!(data.created_at, data.last_accessed_at);

        // After a small delay, touch should update last_accessed_at
        std::thread::sleep(std::time::Duration::from_millis(5));
        let mut data = data;
        data.touch();
        assert!(data.last_accessed_at >= data.created_at);
    }

    #[test]
    fn test_session_metrics() {
        let metrics = SessionMetrics::new();
        assert_eq!(metrics.sessions_added(), 0);
        assert_eq!(metrics.sessions_accessed(), 0);
        assert_eq!(metrics.sessions_evicted(), 0);

        metrics.record_add();
        metrics.record_add();
        assert_eq!(metrics.sessions_added(), 2);

        metrics.record_access();
        assert_eq!(metrics.sessions_accessed(), 1);

        metrics.record_eviction();
        metrics.record_eviction();
        metrics.record_eviction();
        assert_eq!(metrics.sessions_evicted(), 3);
    }

    // ==================== Config Tests ====================

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
    fn test_max_sessions_default() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");

            let config = Config::from_env().expect("should parse config with default max_sessions");
            assert_eq!(config.max_sessions, MAX_TRACKED_SESSIONS);
        });
    }

    #[test]
    #[serial]
    fn test_max_sessions_custom() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_MAX_SESSIONS", "500");

            let config = Config::from_env().expect("should parse config with custom max_sessions");
            assert_eq!(config.max_sessions, 500);
        });
    }

    #[test]
    #[serial]
    fn test_max_sessions_invalid() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_MAX_SESSIONS", "not-a-number");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, .. } if key == "VIBETEA_MAX_SESSIONS"
            ));
        });
    }

    #[test]
    #[serial]
    fn test_max_sessions_zero_rejected() {
        with_clean_env(|| {
            env::set_var("VIBETEA_SERVER_URL", "https://test.example.com");
            env::set_var("VIBETEA_MAX_SESSIONS", "0");

            let result = Config::from_env();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(matches!(
                err,
                ConfigError::InvalidValue { ref key, ref message }
                    if key == "VIBETEA_MAX_SESSIONS" && message.contains("greater than 0")
            ));
        });
    }
}
