//! Session token store for managing authenticated user sessions.
//!
//! This module provides an in-memory session store with TTL management,
//! designed for the VibeTea authentication flow where Supabase JWTs are
//! exchanged for short-lived session tokens.
//!
//! # Architecture
//!
//! The session store maps opaque session tokens to user metadata (Supabase
//! user ID and email). Sessions have a default 5-minute TTL and can be
//! extended once for WebSocket connections.
//!
//! # Token Format
//!
//! Session tokens are 32 bytes of cryptographically secure random data,
//! base64-url encoded without padding, resulting in 43 character tokens.
//!
//! # Thread Safety
//!
//! The [`SessionStore`] uses interior mutability with [`RwLock`] for
//! thread-safe access across async tasks.
//!
//! # Example
//!
//! ```rust
//! use vibetea_server::session::{SessionStore, SessionStoreConfig};
//!
//! let store = SessionStore::new(SessionStoreConfig::default());
//!
//! // Create a session after validating a Supabase JWT
//! let token = store.create_session("user-123".to_string(), Some("user@example.com".to_string()))
//!     .expect("store has capacity");
//!
//! // Later, validate the session token
//! if let Some(session) = store.validate_session(&token, false) {
//!     println!("Valid session for user: {}", session.user_id);
//! }
//! ```

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use thiserror::Error;
use tracing::{debug, trace, warn};

/// Default session TTL (5 minutes per FR-008).
const DEFAULT_TTL_SECS: u64 = 300;

/// Grace period for WebSocket validation (30 seconds per FR-024).
const DEFAULT_GRACE_PERIOD_SECS: u64 = 30;

/// Maximum number of sessions (10,000 per FR-022).
const DEFAULT_MAX_CAPACITY: usize = 10_000;

/// Size of the random token in bytes (32 bytes per FR-021).
const TOKEN_BYTES: usize = 32;

/// Expected length of base64-url encoded token (43 characters).
const TOKEN_LENGTH: usize = 43;

/// Errors that can occur during session operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SessionError {
    /// The session store has reached maximum capacity.
    #[error("session store at maximum capacity ({max_capacity} sessions)")]
    AtCapacity {
        /// The maximum number of sessions allowed.
        max_capacity: usize,
    },

    /// The session token was not found or has expired.
    #[error("session not found or expired")]
    NotFound,

    /// The session token format is invalid.
    #[error("invalid session token format")]
    InvalidToken,
}

/// Configuration for the session store.
#[derive(Debug, Clone)]
pub struct SessionStoreConfig {
    /// Maximum number of concurrent sessions.
    pub max_capacity: usize,

    /// Default time-to-live for new sessions.
    pub default_ttl: Duration,

    /// Grace period added during WebSocket validation.
    pub grace_period: Duration,
}

impl Default for SessionStoreConfig {
    fn default() -> Self {
        Self {
            max_capacity: DEFAULT_MAX_CAPACITY,
            default_ttl: Duration::from_secs(DEFAULT_TTL_SECS),
            grace_period: Duration::from_secs(DEFAULT_GRACE_PERIOD_SECS),
        }
    }
}

impl SessionStoreConfig {
    /// Creates a new configuration with custom values.
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of concurrent sessions
    /// * `default_ttl` - Default time-to-live for new sessions
    /// * `grace_period` - Grace period for WebSocket validation
    pub fn new(max_capacity: usize, default_ttl: Duration, grace_period: Duration) -> Self {
        Self {
            max_capacity,
            default_ttl,
            grace_period,
        }
    }
}

/// A user session with associated metadata and expiration times.
#[derive(Debug, Clone)]
pub struct Session {
    /// The Supabase user ID.
    pub user_id: String,

    /// The user's email address (optional).
    pub email: Option<String>,

    /// When the session was created.
    pub created_at: Instant,

    /// When the session expires.
    pub expires_at: Instant,

    /// Whether the TTL has been extended for a WebSocket connection.
    ttl_extended: bool,
}

impl Session {
    /// Creates a new session with the given user metadata.
    fn new(user_id: String, email: Option<String>, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            user_id,
            email,
            created_at: now,
            expires_at: now + ttl,
            ttl_extended: false,
        }
    }

    /// Returns true if the session has expired.
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Returns true if the session is valid with the given grace period.
    fn is_valid_with_grace(&self, grace_period: Duration) -> bool {
        Instant::now() < self.expires_at + grace_period
    }

    /// Extends the TTL by the given duration if not already extended.
    ///
    /// Returns true if the TTL was extended, false if already extended.
    fn extend_ttl(&mut self, extension: Duration) -> bool {
        if self.ttl_extended {
            return false;
        }
        self.expires_at += extension;
        self.ttl_extended = true;
        true
    }

    /// Returns the remaining time until expiration, or zero if expired.
    pub fn remaining_ttl(&self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }
}

/// Thread-safe in-memory session store.
///
/// The store maps session tokens to user sessions and enforces TTL
/// and capacity limits.
pub struct SessionStore {
    /// The session data, protected by a read-write lock.
    sessions: RwLock<HashMap<String, Session>>,

    /// Store configuration.
    config: SessionStoreConfig,
}

impl SessionStore {
    /// Creates a new session store with the given configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::session::{SessionStore, SessionStoreConfig};
    /// use std::time::Duration;
    ///
    /// let config = SessionStoreConfig::new(
    ///     1000,
    ///     Duration::from_secs(300),
    ///     Duration::from_secs(30),
    /// );
    /// let store = SessionStore::new(config);
    /// ```
    pub fn new(config: SessionStoreConfig) -> Self {
        debug!(
            max_capacity = config.max_capacity,
            ttl_secs = config.default_ttl.as_secs(),
            grace_period_secs = config.grace_period.as_secs(),
            "Creating new session store"
        );
        Self {
            sessions: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Creates a new session for the given user.
    ///
    /// Generates a cryptographically secure session token, stores the
    /// session, and returns the token.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The Supabase user ID
    /// * `email` - Optional email address
    ///
    /// # Returns
    ///
    /// The base64-url encoded session token (43 characters).
    ///
    /// # Errors
    ///
    /// Returns [`SessionError::AtCapacity`] if the store has reached
    /// its maximum capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::session::{SessionStore, SessionStoreConfig};
    ///
    /// let store = SessionStore::new(SessionStoreConfig::default());
    /// let token = store.create_session(
    ///     "user-123".to_string(),
    ///     Some("user@example.com".to_string()),
    /// ).expect("store has capacity");
    ///
    /// assert_eq!(token.len(), 43);
    /// ```
    pub fn create_session(
        &self,
        user_id: String,
        email: Option<String>,
    ) -> Result<String, SessionError> {
        // Generate token first (outside of lock)
        let token = generate_session_token();

        let mut sessions = self.sessions.write().unwrap();

        // Check capacity before insertion
        if sessions.len() >= self.config.max_capacity {
            warn!(
                capacity = sessions.len(),
                max_capacity = self.config.max_capacity,
                "Session store at capacity, rejecting new session"
            );
            return Err(SessionError::AtCapacity {
                max_capacity: self.config.max_capacity,
            });
        }

        let session = Session::new(user_id.clone(), email.clone(), self.config.default_ttl);

        trace!(
            user_id = %user_id,
            email = email.as_deref().unwrap_or("<none>"),
            ttl_secs = self.config.default_ttl.as_secs(),
            "Creating new session"
        );

        sessions.insert(token.clone(), session);

        Ok(token)
    }

    /// Validates a session token and returns the session data if valid.
    ///
    /// Performs lazy cleanup of the accessed session if expired.
    ///
    /// # Arguments
    ///
    /// * `token` - The session token to validate
    /// * `with_grace_period` - If true, allows sessions within the grace
    ///   period (for WebSocket validation per FR-024)
    ///
    /// # Returns
    ///
    /// The session data if valid, or `None` if invalid/expired.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::session::{SessionStore, SessionStoreConfig};
    ///
    /// let store = SessionStore::new(SessionStoreConfig::default());
    /// let token = store.create_session("user-123".to_string(), None)
    ///     .expect("store has capacity");
    ///
    /// // Standard validation (for token exchange)
    /// let session = store.validate_session(&token, false);
    /// assert!(session.is_some());
    ///
    /// // WebSocket validation (with grace period)
    /// let session = store.validate_session(&token, true);
    /// assert!(session.is_some());
    /// ```
    pub fn validate_session(&self, token: &str, with_grace_period: bool) -> Option<Session> {
        // Quick format check
        if token.len() != TOKEN_LENGTH {
            trace!(token_len = token.len(), "Invalid token length");
            return None;
        }

        // First try read-only access
        {
            let sessions = self.sessions.read().unwrap();
            if let Some(session) = sessions.get(token) {
                let is_valid = if with_grace_period {
                    session.is_valid_with_grace(self.config.grace_period)
                } else {
                    !session.is_expired()
                };

                if is_valid {
                    trace!(
                        user_id = %session.user_id,
                        remaining_secs = session.remaining_ttl().as_secs(),
                        "Session validated"
                    );
                    return Some(session.clone());
                }
            } else {
                trace!("Session token not found");
                return None;
            }
        }

        // Session exists but is expired - clean it up
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.remove(token);
            trace!("Removed expired session during validation");
        }

        None
    }

    /// Extends the TTL of a session for a WebSocket connection.
    ///
    /// Per the spec, each session can only be extended once when a
    /// WebSocket connection is established.
    ///
    /// # Arguments
    ///
    /// * `token` - The session token to extend
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the TTL was extended
    /// - `Ok(false)` if the TTL was already extended
    /// - `Err(SessionError::NotFound)` if the session doesn't exist or is expired
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::session::{SessionStore, SessionStoreConfig};
    ///
    /// let store = SessionStore::new(SessionStoreConfig::default());
    /// let token = store.create_session("user-123".to_string(), None)
    ///     .expect("store has capacity");
    ///
    /// // First extension succeeds
    /// assert!(store.extend_session_ttl(&token).unwrap());
    ///
    /// // Second extension returns false (already extended)
    /// assert!(!store.extend_session_ttl(&token).unwrap());
    /// ```
    pub fn extend_session_ttl(&self, token: &str) -> Result<bool, SessionError> {
        let mut sessions = self.sessions.write().unwrap();

        let session = sessions.get_mut(token).ok_or(SessionError::NotFound)?;

        if session.is_expired() {
            // Clean up expired session
            sessions.remove(token);
            return Err(SessionError::NotFound);
        }

        let extended = session.extend_ttl(self.config.default_ttl);

        if extended {
            trace!(
                user_id = %session.user_id,
                new_expires_in_secs = session.remaining_ttl().as_secs(),
                "Extended session TTL"
            );
        } else {
            trace!(
                user_id = %session.user_id,
                "Session TTL already extended"
            );
        }

        Ok(extended)
    }

    /// Removes a session from the store.
    ///
    /// # Arguments
    ///
    /// * `token` - The session token to remove
    ///
    /// # Returns
    ///
    /// The removed session if it existed, or `None`.
    pub fn remove_session(&self, token: &str) -> Option<Session> {
        let mut sessions = self.sessions.write().unwrap();
        let removed = sessions.remove(token);

        if let Some(ref session) = removed {
            trace!(user_id = %session.user_id, "Session removed");
        }

        removed
    }

    /// Returns the current number of sessions in the store.
    ///
    /// Note: This count may include expired sessions that haven't been
    /// cleaned up yet.
    pub fn len(&self) -> usize {
        self.sessions.read().unwrap().len()
    }

    /// Returns true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.sessions.read().unwrap().is_empty()
    }

    /// Returns the number of expired sessions in the store.
    ///
    /// This is useful for monitoring and determining when to trigger
    /// a cleanup sweep.
    pub fn count_expired(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.values().filter(|s| s.is_expired()).count()
    }

    /// Returns the current capacity utilization as a percentage.
    pub fn capacity_percentage(&self) -> f64 {
        let len = self.sessions.read().unwrap().len();
        (len as f64 / self.config.max_capacity as f64) * 100.0
    }

    /// Returns the number of available slots.
    pub fn available_capacity(&self) -> usize {
        let len = self.sessions.read().unwrap().len();
        self.config.max_capacity.saturating_sub(len)
    }

    /// Returns the maximum capacity of the store.
    pub fn max_capacity(&self) -> usize {
        self.config.max_capacity
    }

    /// Removes all expired sessions from the store.
    ///
    /// This method can be called from a background task for periodic
    /// cleanup, complementing the lazy cleanup on access.
    ///
    /// # Returns
    ///
    /// The number of sessions that were removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::session::{SessionStore, SessionStoreConfig};
    /// use std::time::Duration;
    ///
    /// // Short TTL for demonstration
    /// let config = SessionStoreConfig::new(
    ///     1000,
    ///     Duration::from_millis(1),
    ///     Duration::from_millis(0),
    /// );
    /// let store = SessionStore::new(config);
    ///
    /// store.create_session("user-1".to_string(), None).unwrap();
    ///
    /// // Wait for expiration
    /// std::thread::sleep(Duration::from_millis(10));
    ///
    /// let removed = store.cleanup_expired();
    /// assert_eq!(removed, 1);
    /// ```
    pub fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write().unwrap();
        let initial_len = sessions.len();

        sessions.retain(|_, session| !session.is_expired());

        let removed = initial_len - sessions.len();

        if removed > 0 {
            debug!(
                removed_count = removed,
                remaining_count = sessions.len(),
                "Cleaned up expired sessions"
            );
        }

        removed
    }

    /// Clears all sessions from the store.
    ///
    /// This is primarily useful for testing.
    pub fn clear(&self) {
        let mut sessions = self.sessions.write().unwrap();
        let count = sessions.len();
        sessions.clear();
        debug!(cleared_count = count, "Cleared all sessions");
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(SessionStoreConfig::default())
    }
}

impl std::fmt::Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.sessions.read().map(|s| s.len()).unwrap_or(0);
        f.debug_struct("SessionStore")
            .field("session_count", &len)
            .field("config", &self.config)
            .finish()
    }
}

/// Generates a cryptographically secure session token.
///
/// The token is 32 bytes of random data, base64-url encoded without
/// padding, resulting in a 43-character string.
fn generate_session_token() -> String {
    let mut bytes = [0u8; TOKEN_BYTES];
    rand::rng().fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_generate_session_token_length() {
        let token = generate_session_token();
        assert_eq!(token.len(), TOKEN_LENGTH);
    }

    #[test]
    fn test_generate_session_token_uniqueness() {
        let tokens: Vec<String> = (0..1000).map(|_| generate_session_token()).collect();
        let unique: std::collections::HashSet<_> = tokens.iter().collect();
        assert_eq!(tokens.len(), unique.len(), "All tokens should be unique");
    }

    #[test]
    fn test_generate_session_token_is_valid_base64_url() {
        let token = generate_session_token();
        // Verify it can be decoded
        let decoded = URL_SAFE_NO_PAD.decode(&token);
        assert!(decoded.is_ok());
        assert_eq!(decoded.unwrap().len(), TOKEN_BYTES);
    }

    #[test]
    fn test_session_store_create_and_validate() {
        let store = SessionStore::new(SessionStoreConfig::default());

        let token = store
            .create_session("user-123".to_string(), Some("test@example.com".to_string()))
            .expect("should create session");

        assert_eq!(token.len(), TOKEN_LENGTH);

        let session = store.validate_session(&token, false);
        assert!(session.is_some());

        let session = session.unwrap();
        assert_eq!(session.user_id, "user-123");
        assert_eq!(session.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_session_store_validate_invalid_token() {
        let store = SessionStore::new(SessionStoreConfig::default());

        // Wrong length
        assert!(store.validate_session("short", false).is_none());
        assert!(store.validate_session("x".repeat(100).as_str(), false).is_none());

        // Correct length but doesn't exist
        let fake_token = "a".repeat(TOKEN_LENGTH);
        assert!(store.validate_session(&fake_token, false).is_none());
    }

    #[test]
    fn test_session_expiration() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_millis(10),
            Duration::from_millis(0),
        );
        let store = SessionStore::new(config);

        let token = store
            .create_session("user-123".to_string(), None)
            .expect("should create session");

        // Should be valid immediately
        assert!(store.validate_session(&token, false).is_some());

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Should be expired now
        assert!(store.validate_session(&token, false).is_none());
    }

    #[test]
    fn test_session_grace_period() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_millis(10),
            Duration::from_millis(50),
        );
        let store = SessionStore::new(config);

        let token = store
            .create_session("user-123".to_string(), None)
            .expect("should create session");

        // Wait for TTL to expire but within grace period
        thread::sleep(Duration::from_millis(20));

        // Without grace period, should be invalid
        assert!(store.validate_session(&token, false).is_none());

        // Create new session for grace period test
        let token2 = store
            .create_session("user-456".to_string(), None)
            .expect("should create session");

        thread::sleep(Duration::from_millis(20));

        // With grace period, should still be valid
        assert!(store.validate_session(&token2, true).is_some());
    }

    #[test]
    fn test_session_ttl_extension() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_millis(100),
            Duration::from_millis(0),
        );
        let store = SessionStore::new(config);

        let token = store
            .create_session("user-123".to_string(), None)
            .expect("should create session");

        // First extension should succeed
        assert!(store.extend_session_ttl(&token).unwrap());

        // Second extension should return false
        assert!(!store.extend_session_ttl(&token).unwrap());

        // Session should still be valid
        assert!(store.validate_session(&token, false).is_some());
    }

    #[test]
    fn test_session_ttl_extension_not_found() {
        let store = SessionStore::new(SessionStoreConfig::default());

        let result = store.extend_session_ttl("nonexistent-token-that-is-43-chars-long!!");
        assert!(matches!(result, Err(SessionError::NotFound)));
    }

    #[test]
    fn test_session_removal() {
        let store = SessionStore::new(SessionStoreConfig::default());

        let token = store
            .create_session("user-123".to_string(), None)
            .expect("should create session");

        assert!(store.validate_session(&token, false).is_some());

        let removed = store.remove_session(&token);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().user_id, "user-123");

        assert!(store.validate_session(&token, false).is_none());
    }

    #[test]
    fn test_session_store_capacity_limit() {
        let config = SessionStoreConfig::new(
            3,
            Duration::from_secs(300),
            Duration::from_secs(30),
        );
        let store = SessionStore::new(config);

        // Fill to capacity
        store.create_session("user-1".to_string(), None).unwrap();
        store.create_session("user-2".to_string(), None).unwrap();
        store.create_session("user-3".to_string(), None).unwrap();

        assert_eq!(store.len(), 3);

        // Fourth session should fail
        let result = store.create_session("user-4".to_string(), None);
        assert!(matches!(result, Err(SessionError::AtCapacity { max_capacity: 3 })));
    }

    #[test]
    fn test_session_store_cleanup_expired() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_millis(5),
            Duration::from_millis(0),
        );
        let store = SessionStore::new(config);

        // Create some sessions
        store.create_session("user-1".to_string(), None).unwrap();
        store.create_session("user-2".to_string(), None).unwrap();
        store.create_session("user-3".to_string(), None).unwrap();

        assert_eq!(store.len(), 3);

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // All should be expired
        assert_eq!(store.count_expired(), 3);

        // Cleanup
        let removed = store.cleanup_expired();
        assert_eq!(removed, 3);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_session_store_capacity_metrics() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_secs(300),
            Duration::from_secs(30),
        );
        let store = SessionStore::new(config);

        assert_eq!(store.max_capacity(), 100);
        assert_eq!(store.available_capacity(), 100);
        assert_eq!(store.capacity_percentage(), 0.0);

        store.create_session("user-1".to_string(), None).unwrap();
        store.create_session("user-2".to_string(), None).unwrap();

        assert_eq!(store.len(), 2);
        assert_eq!(store.available_capacity(), 98);
        assert_eq!(store.capacity_percentage(), 2.0);
    }

    #[test]
    fn test_session_store_clear() {
        let store = SessionStore::new(SessionStoreConfig::default());

        store.create_session("user-1".to_string(), None).unwrap();
        store.create_session("user-2".to_string(), None).unwrap();

        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());

        store.clear();

        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_session_remaining_ttl() {
        let session = Session::new(
            "user-123".to_string(),
            None,
            Duration::from_secs(300),
        );

        let remaining = session.remaining_ttl();
        // Should be close to 300 seconds (allow some margin for test execution)
        assert!(remaining.as_secs() >= 299);
        assert!(remaining.as_secs() <= 300);
    }

    #[test]
    fn test_session_store_debug_impl() {
        let store = SessionStore::new(SessionStoreConfig::default());
        store.create_session("user-1".to_string(), None).unwrap();

        let debug_str = format!("{:?}", store);
        assert!(debug_str.contains("SessionStore"));
        assert!(debug_str.contains("session_count"));
    }

    #[test]
    fn test_session_error_display() {
        let err = SessionError::AtCapacity { max_capacity: 100 };
        assert_eq!(
            err.to_string(),
            "session store at maximum capacity (100 sessions)"
        );

        let err = SessionError::NotFound;
        assert_eq!(err.to_string(), "session not found or expired");

        let err = SessionError::InvalidToken;
        assert_eq!(err.to_string(), "invalid session token format");
    }

    #[test]
    fn test_session_store_config_default() {
        let config = SessionStoreConfig::default();
        assert_eq!(config.max_capacity, DEFAULT_MAX_CAPACITY);
        assert_eq!(config.default_ttl, Duration::from_secs(DEFAULT_TTL_SECS));
        assert_eq!(config.grace_period, Duration::from_secs(DEFAULT_GRACE_PERIOD_SECS));
    }

    #[test]
    fn test_session_store_default_impl() {
        let store = SessionStore::default();
        assert_eq!(store.max_capacity(), DEFAULT_MAX_CAPACITY);
    }

    #[test]
    fn test_lazy_cleanup_on_validation() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_millis(5),
            Duration::from_millis(0),
        );
        let store = SessionStore::new(config);

        let token = store.create_session("user-1".to_string(), None).unwrap();

        assert_eq!(store.len(), 1);

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Validation should trigger lazy cleanup
        assert!(store.validate_session(&token, false).is_none());

        // Session should be removed
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_extend_expired_session() {
        let config = SessionStoreConfig::new(
            100,
            Duration::from_millis(5),
            Duration::from_millis(0),
        );
        let store = SessionStore::new(config);

        let token = store.create_session("user-1".to_string(), None).unwrap();

        // Wait for expiration
        thread::sleep(Duration::from_millis(20));

        // Should fail since session is expired
        let result = store.extend_session_ttl(&token);
        assert!(matches!(result, Err(SessionError::NotFound)));

        // Session should be cleaned up
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_session_with_no_email() {
        let store = SessionStore::new(SessionStoreConfig::default());

        let token = store
            .create_session("user-123".to_string(), None)
            .expect("should create session");

        let session = store.validate_session(&token, false).unwrap();
        assert_eq!(session.user_id, "user-123");
        assert!(session.email.is_none());
    }

    #[test]
    fn test_concurrent_access() {
        use std::sync::Arc;

        let store = Arc::new(SessionStore::new(SessionStoreConfig::new(
            1000,
            Duration::from_secs(300),
            Duration::from_secs(30),
        )));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    for j in 0..100 {
                        let user_id = format!("user-{}-{}", i, j);
                        if let Ok(token) = store.create_session(user_id, None) {
                            let _ = store.validate_session(&token, false);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All sessions should be created (10 threads * 100 sessions = 1000)
        assert_eq!(store.len(), 1000);
    }
}
