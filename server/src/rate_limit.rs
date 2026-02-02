//! Per-source rate limiting using the token bucket algorithm.
//!
//! This module provides rate limiting functionality to protect the VibeTea server
//! from excessive requests. Each source (identified by the `X-Source-ID` header)
//! has its own token bucket that replenishes over time.
//!
//! # Algorithm
//!
//! The token bucket algorithm works as follows:
//! - Each source has a bucket that can hold up to `capacity` tokens
//! - Tokens are added at a rate of `rate` tokens per second
//! - Each request consumes one token
//! - If no tokens are available, the request is rejected with a `Retry-After` header
//!
//! # Example
//!
//! ```rust
//! use vibetea_server::rate_limit::{RateLimiter, RateLimitResult};
//!
//! #[tokio::main]
//! async fn main() {
//!     let limiter = RateLimiter::new(100.0, 100);
//!
//!     match limiter.check_rate_limit("source-123").await {
//!         RateLimitResult::Allowed => {
//!             // Process the request
//!         }
//!         RateLimitResult::Limited { retry_after_secs } => {
//!             // Return 429 Too Many Requests with Retry-After header
//!         }
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::Instant;

/// Default rate limit: 100 events per second.
pub const DEFAULT_RATE: f64 = 100.0;

/// Default bucket capacity: 100 tokens (allows bursts up to this size).
pub const DEFAULT_CAPACITY: u32 = 100;

/// Duration after which inactive entries are cleaned up (60 seconds).
pub const STALE_ENTRY_TIMEOUT: Duration = Duration::from_secs(60);

/// Result of a rate limit check.
///
/// This enum indicates whether a request should be allowed to proceed
/// or if the client has exceeded their rate limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitResult {
    /// The request is allowed to proceed.
    Allowed,

    /// The request is rate limited.
    ///
    /// The client should wait for the specified number of seconds
    /// before retrying. This value should be returned in the
    /// `Retry-After` HTTP header.
    Limited {
        /// Number of seconds until the client can retry.
        retry_after_secs: u64,
    },
}

impl RateLimitResult {
    /// Returns `true` if the request is allowed.
    #[inline]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Returns `true` if the request is rate limited.
    #[inline]
    pub fn is_limited(&self) -> bool {
        matches!(self, Self::Limited { .. })
    }

    /// Returns the retry-after duration if rate limited, or `None` if allowed.
    #[inline]
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::Allowed => None,
            Self::Limited { retry_after_secs } => Some(*retry_after_secs),
        }
    }
}

/// A token bucket for tracking rate limits for a single source.
///
/// The bucket refills at a constant rate and has a maximum capacity.
/// Each request consumes one token. When the bucket is empty,
/// requests are rejected until tokens are replenished.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Current number of tokens in the bucket.
    tokens: f64,

    /// Time of the last token refill.
    last_refill: Instant,

    /// Maximum number of tokens the bucket can hold.
    capacity: u32,

    /// Rate at which tokens are added (tokens per second).
    rate: f64,
}

impl TokenBucket {
    /// Creates a new token bucket with the specified rate and capacity.
    ///
    /// The bucket starts full (at capacity).
    ///
    /// # Arguments
    ///
    /// * `rate` - Number of tokens added per second
    /// * `capacity` - Maximum number of tokens the bucket can hold
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::rate_limit::TokenBucket;
    ///
    /// let bucket = TokenBucket::new(100.0, 100);
    /// ```
    pub fn new(rate: f64, capacity: u32) -> Self {
        Self {
            tokens: f64::from(capacity),
            last_refill: Instant::now(),
            capacity,
            rate,
        }
    }

    /// Attempts to consume a token from the bucket.
    ///
    /// This method first refills tokens based on elapsed time, then
    /// attempts to consume one token. If successful, returns `Allowed`.
    /// If no tokens are available, returns `Limited` with the time
    /// until the next token will be available.
    ///
    /// # Returns
    ///
    /// - `RateLimitResult::Allowed` if a token was consumed
    /// - `RateLimitResult::Limited { retry_after_secs }` if rate limited
    pub fn try_consume(&mut self) -> RateLimitResult {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            RateLimitResult::Allowed
        } else {
            // Calculate how long until we have at least one token
            let tokens_needed = 1.0 - self.tokens;
            let seconds_until_token = tokens_needed / self.rate;
            // Round up to ensure we have at least one token
            let retry_after_secs = seconds_until_token.ceil() as u64;
            // Ensure at least 1 second retry-after
            let retry_after_secs = retry_after_secs.max(1);

            RateLimitResult::Limited { retry_after_secs }
        }
    }

    /// Refills tokens based on elapsed time since last refill.
    ///
    /// Tokens are added at the configured rate, up to the bucket capacity.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let elapsed_secs = elapsed.as_secs_f64();

        // Calculate tokens to add based on elapsed time
        let tokens_to_add = elapsed_secs * self.rate;
        self.tokens = (self.tokens + tokens_to_add).min(f64::from(self.capacity));
        self.last_refill = now;
    }

    /// Returns the time since the last activity (refill) on this bucket.
    ///
    /// Used for cleaning up stale entries.
    pub fn time_since_last_activity(&self) -> Duration {
        self.last_refill.elapsed()
    }

    /// Returns the current number of tokens (for testing/debugging).
    #[cfg(test)]
    pub fn tokens(&self) -> f64 {
        self.tokens
    }
}

/// Thread-safe rate limiter with per-source tracking.
///
/// Each source (identified by a string ID, typically from the `X-Source-ID` header)
/// has its own [`TokenBucket`]. The rate limiter automatically cleans up
/// stale entries that have been inactive for longer than [`STALE_ENTRY_TIMEOUT`].
///
/// # Thread Safety
///
/// The `RateLimiter` uses a `RwLock` internally, making it safe to share
/// across multiple tokio tasks. The `Arc` wrapper allows cheap cloning
/// for use in axum handlers.
///
/// # Example
///
/// ```rust
/// use vibetea_server::rate_limit::RateLimiter;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let limiter = RateLimiter::new(100.0, 100);
///
///     // Clone for use in multiple handlers
///     let limiter_clone = limiter.clone();
///
///     // Check rate limit
///     let result = limiter.check_rate_limit("my-source").await;
///     assert!(result.is_allowed());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
}

#[derive(Debug)]
struct RateLimiterInner {
    /// Per-source token buckets.
    buckets: HashMap<String, TokenBucket>,

    /// Token replenishment rate (tokens per second).
    rate: f64,

    /// Maximum bucket capacity.
    capacity: u32,
}

impl RateLimiter {
    /// Creates a new rate limiter with the specified rate and capacity.
    ///
    /// # Arguments
    ///
    /// * `rate` - Number of tokens added per second per source
    /// * `capacity` - Maximum tokens per source (burst capacity)
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::rate_limit::RateLimiter;
    ///
    /// // 100 requests/second, burst of 100
    /// let limiter = RateLimiter::new(100.0, 100);
    /// ```
    pub fn new(rate: f64, capacity: u32) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RateLimiterInner {
                buckets: HashMap::new(),
                rate,
                capacity,
            })),
        }
    }

    /// Creates a new rate limiter with default settings.
    ///
    /// Uses [`DEFAULT_RATE`] (100 tokens/sec) and [`DEFAULT_CAPACITY`] (100 tokens).
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::rate_limit::RateLimiter;
    ///
    /// let limiter = RateLimiter::default();
    /// ```
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_RATE, DEFAULT_CAPACITY)
    }

    /// Checks if a request from the given source should be rate limited.
    ///
    /// This method:
    /// 1. Creates a new token bucket for the source if one doesn't exist
    /// 2. Refills tokens based on elapsed time
    /// 3. Attempts to consume one token
    /// 4. Returns whether the request is allowed or rate limited
    ///
    /// # Arguments
    ///
    /// * `source_id` - Unique identifier for the request source (e.g., from X-Source-ID header)
    ///
    /// # Returns
    ///
    /// - `RateLimitResult::Allowed` if the request can proceed
    /// - `RateLimitResult::Limited { retry_after_secs }` if rate limited
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::rate_limit::{RateLimiter, RateLimitResult};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let limiter = RateLimiter::new(100.0, 100);
    ///
    ///     match limiter.check_rate_limit("source-123").await {
    ///         RateLimitResult::Allowed => {
    ///             println!("Request allowed");
    ///         }
    ///         RateLimitResult::Limited { retry_after_secs } => {
    ///             println!("Rate limited, retry after {} seconds", retry_after_secs);
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn check_rate_limit(&self, source_id: &str) -> RateLimitResult {
        let mut inner = self.inner.write().await;

        // Extract rate and capacity before borrowing buckets mutably
        let rate = inner.rate;
        let capacity = inner.capacity;

        let bucket = inner
            .buckets
            .entry(source_id.to_string())
            .or_insert_with(|| TokenBucket::new(rate, capacity));

        bucket.try_consume()
    }

    /// Removes stale entries that have been inactive for longer than the timeout.
    ///
    /// This method should be called periodically (e.g., every 30 seconds) to
    /// prevent memory growth from abandoned source IDs.
    ///
    /// # Returns
    ///
    /// The number of entries that were removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::rate_limit::RateLimiter;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let limiter = RateLimiter::new(100.0, 100);
    ///
    ///     // Periodically clean up
    ///     let removed = limiter.cleanup_stale_entries().await;
    ///     println!("Removed {} stale entries", removed);
    /// }
    /// ```
    pub async fn cleanup_stale_entries(&self) -> usize {
        self.cleanup_stale_entries_with_timeout(STALE_ENTRY_TIMEOUT)
            .await
    }

    /// Removes stale entries with a custom timeout duration.
    ///
    /// This is useful for testing or when a different timeout is desired.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Duration after which an inactive entry is considered stale
    ///
    /// # Returns
    ///
    /// The number of entries that were removed.
    pub async fn cleanup_stale_entries_with_timeout(&self, timeout: Duration) -> usize {
        let mut inner = self.inner.write().await;
        let initial_count = inner.buckets.len();

        inner
            .buckets
            .retain(|_, bucket| bucket.time_since_last_activity() < timeout);

        initial_count - inner.buckets.len()
    }

    /// Returns the current number of tracked sources.
    ///
    /// Useful for monitoring and debugging.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::rate_limit::RateLimiter;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let limiter = RateLimiter::new(100.0, 100);
    ///     limiter.check_rate_limit("source-1").await;
    ///     limiter.check_rate_limit("source-2").await;
    ///
    ///     assert_eq!(limiter.source_count().await, 2);
    /// }
    /// ```
    pub async fn source_count(&self) -> usize {
        self.inner.read().await.buckets.len()
    }

    /// Spawns a background task that periodically cleans up stale entries.
    ///
    /// The task runs every `cleanup_interval` and removes entries that have
    /// been inactive for longer than [`STALE_ENTRY_TIMEOUT`].
    ///
    /// # Arguments
    ///
    /// * `cleanup_interval` - How often to run the cleanup task
    ///
    /// # Returns
    ///
    /// A `JoinHandle` for the spawned task. The task runs until dropped.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use vibetea_server::rate_limit::RateLimiter;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let limiter = RateLimiter::new(100.0, 100);
    ///
    ///     // Clean up every 30 seconds
    ///     let _cleanup_handle = limiter.spawn_cleanup_task(Duration::from_secs(30));
    ///
    ///     // Server runs...
    /// }
    /// ```
    pub fn spawn_cleanup_task(&self, cleanup_interval: Duration) -> tokio::task::JoinHandle<()> {
        let limiter = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;
                let removed = limiter.cleanup_stale_entries().await;
                if removed > 0 {
                    tracing::debug!(
                        removed_count = removed,
                        "Cleaned up stale rate limit entries"
                    );
                }
            }
        })
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that requests under the limit are allowed.
    #[tokio::test]
    async fn allows_requests_under_limit() {
        let limiter = RateLimiter::new(10.0, 10);

        // Should allow up to capacity (10) requests
        for i in 0..10 {
            let result = limiter.check_rate_limit("test-source").await;
            assert!(result.is_allowed(), "Request {} should be allowed", i + 1);
        }
    }

    /// Test that requests over the limit are blocked.
    #[tokio::test]
    async fn blocks_requests_over_limit() {
        let limiter = RateLimiter::new(10.0, 5);

        // Exhaust the bucket (capacity is 5)
        for _ in 0..5 {
            let result = limiter.check_rate_limit("test-source").await;
            assert!(result.is_allowed());
        }

        // Next request should be blocked
        let result = limiter.check_rate_limit("test-source").await;
        assert!(result.is_limited());
    }

    /// Test that retry_after is calculated correctly.
    #[tokio::test]
    async fn retry_after_calculation() {
        let limiter = RateLimiter::new(1.0, 1); // 1 token per second

        // Consume the single token
        let result = limiter.check_rate_limit("test-source").await;
        assert!(result.is_allowed());

        // Next request should be blocked with ~1 second retry-after
        let result = limiter.check_rate_limit("test-source").await;
        assert!(result.is_limited());

        if let RateLimitResult::Limited { retry_after_secs } = result {
            // Should be at least 1 second
            assert!(
                retry_after_secs >= 1,
                "retry_after should be at least 1 second, got {}",
                retry_after_secs
            );
        }
    }

    /// Test that tokens refill over time.
    #[tokio::test]
    async fn tokens_refill_over_time() {
        let limiter = RateLimiter::new(10.0, 2); // 10 tokens/sec, capacity 2

        // Exhaust the bucket
        for _ in 0..2 {
            let result = limiter.check_rate_limit("test-source").await;
            assert!(result.is_allowed());
        }

        // Should be blocked now
        let result = limiter.check_rate_limit("test-source").await;
        assert!(result.is_limited());

        // Wait for tokens to refill (100ms = 1 token at 10 tokens/sec)
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should have at least one token now
        let result = limiter.check_rate_limit("test-source").await;
        assert!(
            result.is_allowed(),
            "Should have refilled at least one token"
        );
    }

    /// Test that different sources have independent buckets.
    #[tokio::test]
    async fn independent_source_tracking() {
        let limiter = RateLimiter::new(10.0, 2);

        // Exhaust source-1's bucket
        for _ in 0..2 {
            limiter.check_rate_limit("source-1").await;
        }
        assert!(limiter.check_rate_limit("source-1").await.is_limited());

        // source-2 should still be allowed
        let result = limiter.check_rate_limit("source-2").await;
        assert!(result.is_allowed(), "source-2 should have its own bucket");
    }

    /// Test stale entry cleanup.
    #[tokio::test]
    async fn cleanup_stale_entries() {
        let limiter = RateLimiter::new(10.0, 10);

        // Create some entries
        limiter.check_rate_limit("source-1").await;
        limiter.check_rate_limit("source-2").await;
        limiter.check_rate_limit("source-3").await;

        assert_eq!(limiter.source_count().await, 3);

        // Clean up with a very short timeout (all entries should be removed)
        // We need to wait a tiny bit so entries become "stale"
        tokio::time::sleep(Duration::from_millis(10)).await;
        let removed = limiter
            .cleanup_stale_entries_with_timeout(Duration::from_millis(5))
            .await;

        assert_eq!(removed, 3);
        assert_eq!(limiter.source_count().await, 0);
    }

    /// Test that recent entries are not cleaned up.
    #[tokio::test]
    async fn keeps_recent_entries() {
        let limiter = RateLimiter::new(10.0, 10);

        // Create an entry
        limiter.check_rate_limit("recent-source").await;

        // Clean up with a long timeout (entry should be kept)
        let removed = limiter
            .cleanup_stale_entries_with_timeout(Duration::from_secs(3600))
            .await;

        assert_eq!(removed, 0);
        assert_eq!(limiter.source_count().await, 1);
    }

    /// Test RateLimitResult helper methods.
    #[test]
    fn rate_limit_result_helpers() {
        let allowed = RateLimitResult::Allowed;
        assert!(allowed.is_allowed());
        assert!(!allowed.is_limited());
        assert_eq!(allowed.retry_after(), None);

        let limited = RateLimitResult::Limited {
            retry_after_secs: 5,
        };
        assert!(!limited.is_allowed());
        assert!(limited.is_limited());
        assert_eq!(limited.retry_after(), Some(5));
    }

    /// Test TokenBucket directly.
    #[test]
    fn token_bucket_starts_full() {
        let bucket = TokenBucket::new(100.0, 50);
        assert_eq!(bucket.tokens(), 50.0);
    }

    /// Test that consuming tokens reduces the count.
    #[test]
    fn token_bucket_consumes_tokens() {
        let mut bucket = TokenBucket::new(100.0, 10);

        let result = bucket.try_consume();
        assert!(result.is_allowed());
        assert!((bucket.tokens() - 9.0).abs() < 0.1); // Allow for small refill
    }

    /// Test default rate limiter configuration.
    #[tokio::test]
    async fn default_configuration() {
        let limiter = RateLimiter::default();

        // Should allow DEFAULT_CAPACITY (100) requests
        for _ in 0..100 {
            let result = limiter.check_rate_limit("test").await;
            assert!(result.is_allowed());
        }

        // Next request should be limited
        let result = limiter.check_rate_limit("test").await;
        assert!(result.is_limited());
    }

    /// Test that the rate limiter can be cloned and shared.
    #[tokio::test]
    async fn cloneable_and_shareable() {
        let limiter = RateLimiter::new(10.0, 5);
        let limiter_clone = limiter.clone();

        // Exhaust through original
        for _ in 0..5 {
            limiter.check_rate_limit("shared-source").await;
        }

        // Clone should see the same state
        let result = limiter_clone.check_rate_limit("shared-source").await;
        assert!(result.is_limited());
    }

    /// Test high-throughput scenario.
    #[tokio::test]
    async fn high_throughput_scenario() {
        let limiter = RateLimiter::new(1000.0, 100);

        // Rapid-fire 100 requests
        for _ in 0..100 {
            let result = limiter.check_rate_limit("burst-source").await;
            assert!(result.is_allowed());
        }

        // 101st should be limited
        let result = limiter.check_rate_limit("burst-source").await;
        assert!(result.is_limited());
    }

    /// Test retry-after minimum is 1 second.
    #[tokio::test]
    async fn retry_after_minimum_one_second() {
        let limiter = RateLimiter::new(1000.0, 1); // Very fast refill rate

        // Exhaust the bucket
        limiter.check_rate_limit("fast-source").await;

        // Even with fast refill, retry-after should be at least 1 second
        let result = limiter.check_rate_limit("fast-source").await;
        if let RateLimitResult::Limited { retry_after_secs } = result {
            assert!(
                retry_after_secs >= 1,
                "retry_after should be at least 1 second"
            );
        }
    }

    /// Test bucket capacity is respected (no overflow).
    #[tokio::test]
    async fn bucket_capacity_respected() {
        let limiter = RateLimiter::new(1000.0, 5); // High rate, low capacity

        // Wait to let it "overfill" (but it shouldn't)
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should only allow 5 requests (capacity), not more
        for i in 0..5 {
            let result = limiter.check_rate_limit("capped-source").await;
            assert!(result.is_allowed(), "Request {} should be allowed", i + 1);
        }

        // 6th should be limited
        let result = limiter.check_rate_limit("capped-source").await;
        assert!(result.is_limited());
    }
}
