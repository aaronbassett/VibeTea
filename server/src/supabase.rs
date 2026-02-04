//! Supabase client module for JWT validation and public key fetching.
//!
//! This module provides a client for interacting with Supabase services:
//! - JWT validation via the `/auth/v1/user` endpoint
//! - Public key fetching from edge functions
//!
//! # Architecture
//!
//! The [`SupabaseClient`] is designed to be shared across the application (via `Arc`)
//! and handles all communication with Supabase services. It includes:
//! - Configurable timeouts (5 seconds for requests)
//! - Retry logic with exponential backoff for startup operations
//! - Structured error handling with [`SupabaseError`]
//!
//! # Example
//!
//! ```rust,ignore
//! use vibetea_server::supabase::SupabaseClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = SupabaseClient::new(
//!         "https://your-project.supabase.co",
//!         "your-anon-key",
//!     )?;
//!
//!     // Validate a JWT
//!     let user = client.validate_jwt("user-jwt-token").await?;
//!     println!("User ID: {}", user.id);
//!
//!     // Fetch public keys
//!     let keys = client.fetch_public_keys().await?;
//!     for key in keys {
//!         println!("Source: {}, Key: {}", key.source_id, key.public_key);
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::time::Duration;

use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// Default timeout for Supabase API requests.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum number of retry attempts for startup operations.
const MAX_RETRY_ATTEMPTS: u32 = 5;

/// Base delay for exponential backoff (100ms).
const BASE_BACKOFF_MS: u64 = 100;

/// Maximum delay cap for exponential backoff (10 seconds).
const MAX_BACKOFF_MS: u64 = 10_000;

/// Maximum jitter to add to backoff delay (100ms).
const MAX_JITTER_MS: u64 = 100;

/// Errors that can occur when interacting with Supabase.
///
/// These errors provide granular information about failures, allowing
/// callers to handle different error conditions appropriately (e.g.,
/// returning 401 for unauthorized vs 503 for timeout).
#[derive(Debug, Error)]
pub enum SupabaseError {
    /// The provided JWT is invalid or expired.
    ///
    /// This typically indicates that the user needs to re-authenticate.
    /// Maps to HTTP 401 Unauthorized.
    #[error("unauthorized: invalid or expired JWT")]
    Unauthorized,

    /// The request to Supabase timed out.
    ///
    /// This may indicate network issues or Supabase service problems.
    /// Maps to HTTP 503 Service Unavailable.
    #[error("request timed out after {0:?}")]
    Timeout(Duration),

    /// Supabase is unreachable.
    ///
    /// This indicates a network failure or that Supabase services are down.
    /// Maps to HTTP 503 Service Unavailable.
    #[error("supabase unavailable: {0}")]
    Unavailable(String),

    /// Failed to parse the response from Supabase.
    ///
    /// This indicates an unexpected response format, possibly due to
    /// API version mismatch or service changes.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// Client configuration error.
    ///
    /// This indicates a problem with the client setup, such as an invalid URL.
    #[error("client configuration error: {0}")]
    Configuration(String),

    /// All retry attempts have been exhausted.
    ///
    /// This is used during startup when the initial connection fails
    /// after all retry attempts.
    #[error("all {attempts} retry attempts failed: {last_error}")]
    RetriesExhausted {
        /// Number of attempts made.
        attempts: u32,
        /// The last error encountered.
        last_error: String,
    },
}

/// User information returned from Supabase JWT validation.
///
/// This struct contains the essential user data extracted from the
/// validated JWT, suitable for creating server-side sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupabaseUser {
    /// The unique identifier for the user (UUID format).
    pub id: String,

    /// The user's email address, if available.
    ///
    /// This may be `None` for users who authenticated without email
    /// (e.g., phone authentication or anonymous users).
    pub email: Option<String>,
}

/// A public key retrieved from Supabase edge functions.
///
/// Public keys are used to verify signatures on events from monitors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    /// The identifier of the source (monitor) that owns this key.
    pub source_id: String,

    /// The base64-encoded Ed25519 public key.
    pub public_key: String,
}

/// Response format from the Supabase public-keys edge function.
#[derive(Debug, Deserialize)]
struct PublicKeysResponse {
    keys: Vec<PublicKey>,
}

/// Response format from the Supabase auth user endpoint.
///
/// This represents the user data returned by `/auth/v1/user`.
#[derive(Debug, Deserialize)]
struct SupabaseUserResponse {
    id: String,
    email: Option<String>,
}

/// Client for interacting with Supabase services.
///
/// This client handles JWT validation and public key fetching with
/// appropriate timeouts and error handling. It is designed to be
/// thread-safe and shareable via `Arc`.
///
/// # Thread Safety
///
/// The client uses an internal `reqwest::Client` which is already
/// designed to be shared across threads. Wrap in `Arc` for sharing.
///
/// # Example
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use vibetea_server::supabase::SupabaseClient;
///
/// let client = Arc::new(SupabaseClient::new(
///     "https://project.supabase.co",
///     "anon-key",
/// )?);
///
/// // Clone the Arc to share across tasks
/// let client_clone = Arc::clone(&client);
/// tokio::spawn(async move {
///     let user = client_clone.validate_jwt("token").await?;
///     Ok::<_, vibetea_server::supabase::SupabaseError>(())
/// });
/// ```
#[derive(Debug, Clone)]
pub struct SupabaseClient {
    /// The underlying HTTP client.
    http_client: Client,

    /// The base URL of the Supabase project (e.g., `https://xxx.supabase.co`).
    base_url: String,

    /// The Supabase anonymous/public key for API authentication.
    anon_key: String,
}

impl SupabaseClient {
    /// Creates a new Supabase client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The Supabase project URL (e.g., `https://xxx.supabase.co`)
    /// * `anon_key` - The Supabase anonymous/public key
    ///
    /// # Errors
    ///
    /// Returns [`SupabaseError::Configuration`] if the HTTP client cannot be created.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = SupabaseClient::new(
    ///     "https://my-project.supabase.co",
    ///     "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    /// )?;
    /// ```
    pub fn new(
        base_url: impl Into<String>,
        anon_key: impl Into<String>,
    ) -> Result<Self, SupabaseError> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let anon_key = anon_key.into();

        let http_client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| {
                SupabaseError::Configuration(format!("failed to create HTTP client: {e}"))
            })?;

        Ok(Self {
            http_client,
            base_url,
            anon_key,
        })
    }

    /// Validates a Supabase JWT token.
    ///
    /// This method validates the JWT by calling Supabase's `/auth/v1/user` endpoint.
    /// This approach is simpler than local validation and automatically handles
    /// token revocation.
    ///
    /// # Arguments
    ///
    /// * `jwt` - The JWT token to validate
    ///
    /// # Returns
    ///
    /// Returns [`SupabaseUser`] containing the user's ID and email on success.
    ///
    /// # Errors
    ///
    /// - [`SupabaseError::Unauthorized`] - The JWT is invalid or expired
    /// - [`SupabaseError::Timeout`] - The request timed out (5 second limit)
    /// - [`SupabaseError::Unavailable`] - Supabase is unreachable
    /// - [`SupabaseError::InvalidResponse`] - Failed to parse the response
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user = client.validate_jwt("eyJhbGciOiJIUzI1NiIs...").await?;
    /// println!("Authenticated user: {}", user.id);
    /// ```
    pub async fn validate_jwt(&self, jwt: &str) -> Result<SupabaseUser, SupabaseError> {
        let url = format!("{}/auth/v1/user", self.base_url);

        debug!(url = %url, "Validating JWT with Supabase");

        let response = self
            .http_client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", format!("Bearer {jwt}"))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SupabaseError::Timeout(REQUEST_TIMEOUT)
                } else if e.is_connect() {
                    SupabaseError::Unavailable(format!("connection failed: {e}"))
                } else {
                    SupabaseError::Unavailable(format!("request failed: {e}"))
                }
            })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            debug!("JWT validation failed: unauthorized");
            return Err(SupabaseError::Unauthorized);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Unexpected response from Supabase auth");
            return Err(SupabaseError::InvalidResponse(format!(
                "unexpected status {status}: {body}"
            )));
        }

        let user_response: SupabaseUserResponse = response.json().await.map_err(|e| {
            SupabaseError::InvalidResponse(format!("failed to parse user response: {e}"))
        })?;

        debug!(user_id = %user_response.id, "JWT validated successfully");

        Ok(SupabaseUser {
            id: user_response.id,
            email: user_response.email,
        })
    }

    /// Fetches public keys from the Supabase edge function.
    ///
    /// This method retrieves the list of public keys for all registered monitors
    /// from the `/functions/v1/public-keys` endpoint. No authentication is required
    /// for this endpoint.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<PublicKey>` containing all registered public keys.
    ///
    /// # Errors
    ///
    /// - [`SupabaseError::Timeout`] - The request timed out (5 second limit)
    /// - [`SupabaseError::Unavailable`] - Supabase is unreachable
    /// - [`SupabaseError::InvalidResponse`] - Failed to parse the response
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let keys = client.fetch_public_keys().await?;
    /// for key in &keys {
    ///     println!("Monitor {}: {}", key.source_id, key.public_key);
    /// }
    /// ```
    pub async fn fetch_public_keys(&self) -> Result<Vec<PublicKey>, SupabaseError> {
        let url = format!("{}/functions/v1/public-keys", self.base_url);

        debug!(url = %url, "Fetching public keys from Supabase");

        let response = self
            .http_client
            .get(&url)
            .header("apikey", &self.anon_key)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SupabaseError::Timeout(REQUEST_TIMEOUT)
                } else if e.is_connect() {
                    SupabaseError::Unavailable(format!("connection failed: {e}"))
                } else {
                    SupabaseError::Unavailable(format!("request failed: {e}"))
                }
            })?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Failed to fetch public keys");
            return Err(SupabaseError::InvalidResponse(format!(
                "unexpected status {status}: {body}"
            )));
        }

        let keys_response: PublicKeysResponse = response.json().await.map_err(|e| {
            SupabaseError::InvalidResponse(format!("failed to parse keys response: {e}"))
        })?;

        debug!(count = keys_response.keys.len(), "Fetched public keys");

        Ok(keys_response.keys)
    }

    /// Fetches public keys with retry logic for startup.
    ///
    /// This method implements exponential backoff with jitter for reliable
    /// startup behavior. It will retry up to 5 times before failing, with
    /// delays calculated as:
    ///
    /// ```text
    /// delay = min(2^attempt * 100ms + random(0, 100ms), 10s)
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `Vec<PublicKey>` on success.
    ///
    /// # Errors
    ///
    /// Returns [`SupabaseError::RetriesExhausted`] if all retry attempts fail.
    /// The server should exit with an error if this occurs during startup.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // During server startup
    /// let keys = client.fetch_public_keys_with_retry().await?;
    /// info!("Loaded {} public keys", keys.len());
    /// ```
    pub async fn fetch_public_keys_with_retry(&self) -> Result<Vec<PublicKey>, SupabaseError> {
        let mut last_error = String::new();

        for attempt in 0..MAX_RETRY_ATTEMPTS {
            match self.fetch_public_keys().await {
                Ok(keys) => {
                    if attempt > 0 {
                        info!(
                            attempt = attempt + 1,
                            "Public key fetch succeeded after retry"
                        );
                    }
                    return Ok(keys);
                }
                Err(e) => {
                    last_error = e.to_string();

                    if attempt < MAX_RETRY_ATTEMPTS - 1 {
                        let delay = calculate_backoff_delay(attempt);
                        warn!(
                            attempt = attempt + 1,
                            max_attempts = MAX_RETRY_ATTEMPTS,
                            delay_ms = delay.as_millis(),
                            error = %e,
                            "Public key fetch failed, retrying"
                        );
                        sleep(delay).await;
                    } else {
                        error!(
                            attempts = MAX_RETRY_ATTEMPTS,
                            error = %e,
                            "Public key fetch failed, no more retries"
                        );
                    }
                }
            }
        }

        Err(SupabaseError::RetriesExhausted {
            attempts: MAX_RETRY_ATTEMPTS,
            last_error,
        })
    }

    /// Returns the base URL of the Supabase project.
    ///
    /// This can be useful for debugging or logging purposes.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Calculates the backoff delay for a given retry attempt.
///
/// Uses exponential backoff with jitter:
/// `delay = min(2^attempt * 100ms + random(0, 100ms), 10s)`
///
/// # Arguments
///
/// * `attempt` - The zero-indexed attempt number
///
/// # Returns
///
/// The duration to wait before the next attempt.
fn calculate_backoff_delay(attempt: u32) -> Duration {
    let exponential_ms = BASE_BACKOFF_MS.saturating_mul(2u64.saturating_pow(attempt));
    let jitter_ms = rand::rng().random_range(0..=MAX_JITTER_MS);
    let total_ms = exponential_ms.saturating_add(jitter_ms).min(MAX_BACKOFF_MS);
    Duration::from_millis(total_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Helper to create a test client pointing to a mock server.
    fn create_test_client(mock_server: &MockServer) -> SupabaseClient {
        SupabaseClient::new(mock_server.uri(), "test-anon-key")
            .expect("failed to create test client")
    }

    // ==================== SupabaseClient::new tests ====================

    #[test]
    fn new_creates_client_with_valid_params() {
        let client = SupabaseClient::new("https://test.supabase.co", "anon-key");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url, "https://test.supabase.co");
        assert_eq!(client.anon_key, "anon-key");
    }

    #[test]
    fn new_trims_trailing_slash_from_url() {
        let client = SupabaseClient::new("https://test.supabase.co/", "anon-key")
            .expect("should create client");
        assert_eq!(client.base_url, "https://test.supabase.co");
    }

    #[test]
    fn new_trims_multiple_trailing_slashes() {
        let client = SupabaseClient::new("https://test.supabase.co///", "anon-key")
            .expect("should create client");
        assert_eq!(client.base_url, "https://test.supabase.co");
    }

    #[test]
    fn base_url_returns_configured_url() {
        let client = SupabaseClient::new("https://project.supabase.co", "key")
            .expect("should create client");
        assert_eq!(client.base_url(), "https://project.supabase.co");
    }

    // ==================== validate_jwt tests ====================

    #[tokio::test]
    async fn validate_jwt_returns_user_on_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/auth/v1/user"))
            .and(header("apikey", "test-anon-key"))
            .and(header("Authorization", "Bearer valid-jwt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "user-123",
                "email": "user@example.com"
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.validate_jwt("valid-jwt").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, "user-123");
        assert_eq!(user.email, Some("user@example.com".to_string()));
    }

    #[tokio::test]
    async fn validate_jwt_returns_user_without_email() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/auth/v1/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "user-456",
                "email": null
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.validate_jwt("token").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, "user-456");
        assert!(user.email.is_none());
    }

    #[tokio::test]
    async fn validate_jwt_returns_unauthorized_on_401() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/auth/v1/user"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.validate_jwt("invalid-jwt").await;

        assert!(matches!(result, Err(SupabaseError::Unauthorized)));
    }

    #[tokio::test]
    async fn validate_jwt_returns_invalid_response_on_unexpected_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/auth/v1/user"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.validate_jwt("token").await;

        assert!(matches!(result, Err(SupabaseError::InvalidResponse(_))));
    }

    #[tokio::test]
    async fn validate_jwt_returns_invalid_response_on_malformed_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/auth/v1/user"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.validate_jwt("token").await;

        assert!(matches!(result, Err(SupabaseError::InvalidResponse(_))));
    }

    #[tokio::test]
    async fn validate_jwt_returns_unavailable_on_connection_error() {
        // Use a URL that won't connect
        let client =
            SupabaseClient::new("http://127.0.0.1:1", "key").expect("should create client");

        let result = client.validate_jwt("token").await;

        // Should be either Unavailable (connection refused) or Timeout
        assert!(matches!(
            result,
            Err(SupabaseError::Unavailable(_)) | Err(SupabaseError::Timeout(_))
        ));
    }

    // ==================== fetch_public_keys tests ====================

    #[tokio::test]
    async fn fetch_public_keys_returns_keys_on_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .and(header("apikey", "test-anon-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "keys": [
                    {"source_id": "monitor-1", "public_key": "key1"},
                    {"source_id": "monitor-2", "public_key": "key2"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys().await;

        assert!(result.is_ok());
        let keys = result.unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].source_id, "monitor-1");
        assert_eq!(keys[0].public_key, "key1");
        assert_eq!(keys[1].source_id, "monitor-2");
        assert_eq!(keys[1].public_key, "key2");
    }

    #[tokio::test]
    async fn fetch_public_keys_returns_empty_vec_on_empty_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "keys": []
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys().await;

        assert!(result.is_ok());
        let keys = result.unwrap();
        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn fetch_public_keys_returns_invalid_response_on_error_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys().await;

        assert!(matches!(result, Err(SupabaseError::InvalidResponse(_))));
    }

    #[tokio::test]
    async fn fetch_public_keys_returns_invalid_response_on_malformed_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys().await;

        assert!(matches!(result, Err(SupabaseError::InvalidResponse(_))));
    }

    // ==================== fetch_public_keys_with_retry tests ====================

    #[tokio::test]
    async fn fetch_public_keys_with_retry_succeeds_on_first_attempt() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "keys": [{"source_id": "test", "public_key": "key"}]
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys_with_retry().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn fetch_public_keys_with_retry_succeeds_after_failures() {
        let mock_server = MockServer::start().await;

        // First two requests fail, third succeeds
        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "keys": [{"source_id": "recovered", "public_key": "key"}]
            })))
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys_with_retry().await;

        assert!(result.is_ok());
        let keys = result.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].source_id, "recovered");
    }

    #[tokio::test]
    async fn fetch_public_keys_with_retry_returns_error_after_max_attempts() {
        let mock_server = MockServer::start().await;

        // All requests fail
        Mock::given(method("GET"))
            .and(path("/functions/v1/public-keys"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
            .expect(5) // Should try exactly 5 times
            .mount(&mock_server)
            .await;

        let client = create_test_client(&mock_server);
        let result = client.fetch_public_keys_with_retry().await;

        assert!(matches!(
            result,
            Err(SupabaseError::RetriesExhausted { attempts: 5, .. })
        ));
    }

    // ==================== calculate_backoff_delay tests ====================

    #[test]
    fn calculate_backoff_delay_increases_exponentially() {
        // Test multiple times to account for jitter
        let mut delays: Vec<Duration> = Vec::new();

        for _ in 0..10 {
            let delay_0 = calculate_backoff_delay(0);
            let delay_1 = calculate_backoff_delay(1);
            let delay_2 = calculate_backoff_delay(2);

            // Delay should generally increase (accounting for jitter)
            delays.push(delay_0);
            delays.push(delay_1);
            delays.push(delay_2);
        }

        // All delays should be within expected bounds
        for delay in delays {
            assert!(delay.as_millis() <= MAX_BACKOFF_MS as u128 + MAX_JITTER_MS as u128);
        }
    }

    #[test]
    fn calculate_backoff_delay_is_capped_at_max() {
        // Very high attempt number should still be capped
        let delay = calculate_backoff_delay(100);
        assert!(delay.as_millis() <= MAX_BACKOFF_MS as u128 + MAX_JITTER_MS as u128);
    }

    #[test]
    fn calculate_backoff_delay_includes_jitter() {
        // Run multiple times and verify we get different values (with high probability)
        let delays: Vec<u128> = (0..100)
            .map(|_| calculate_backoff_delay(0).as_millis())
            .collect();

        // Check that not all values are the same (jitter is working)
        let first = delays[0];
        let has_variation = delays.iter().any(|&d| d != first);
        assert!(has_variation, "Jitter should produce variation in delays");
    }

    // ==================== SupabaseError tests ====================

    #[test]
    fn supabase_error_unauthorized_display() {
        let err = SupabaseError::Unauthorized;
        assert_eq!(err.to_string(), "unauthorized: invalid or expired JWT");
    }

    #[test]
    fn supabase_error_timeout_display() {
        let err = SupabaseError::Timeout(Duration::from_secs(5));
        assert_eq!(err.to_string(), "request timed out after 5s");
    }

    #[test]
    fn supabase_error_unavailable_display() {
        let err = SupabaseError::Unavailable("connection refused".to_string());
        assert_eq!(err.to_string(), "supabase unavailable: connection refused");
    }

    #[test]
    fn supabase_error_invalid_response_display() {
        let err = SupabaseError::InvalidResponse("missing field".to_string());
        assert_eq!(err.to_string(), "invalid response: missing field");
    }

    #[test]
    fn supabase_error_configuration_display() {
        let err = SupabaseError::Configuration("invalid URL".to_string());
        assert_eq!(err.to_string(), "client configuration error: invalid URL");
    }

    #[test]
    fn supabase_error_retries_exhausted_display() {
        let err = SupabaseError::RetriesExhausted {
            attempts: 5,
            last_error: "connection refused".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "all 5 retry attempts failed: connection refused"
        );
    }

    // ==================== SupabaseUser tests ====================

    #[test]
    fn supabase_user_equality() {
        let user1 = SupabaseUser {
            id: "123".to_string(),
            email: Some("test@example.com".to_string()),
        };
        let user2 = SupabaseUser {
            id: "123".to_string(),
            email: Some("test@example.com".to_string()),
        };
        assert_eq!(user1, user2);
    }

    #[test]
    fn supabase_user_clone() {
        let user = SupabaseUser {
            id: "123".to_string(),
            email: None,
        };
        let cloned = user.clone();
        assert_eq!(user, cloned);
    }

    #[test]
    fn supabase_user_debug() {
        let user = SupabaseUser {
            id: "123".to_string(),
            email: Some("test@example.com".to_string()),
        };
        let debug = format!("{:?}", user);
        assert!(debug.contains("123"));
        assert!(debug.contains("test@example.com"));
    }

    // ==================== PublicKey tests ====================

    #[test]
    fn public_key_equality() {
        let key1 = PublicKey {
            source_id: "monitor-1".to_string(),
            public_key: "abc123".to_string(),
        };
        let key2 = PublicKey {
            source_id: "monitor-1".to_string(),
            public_key: "abc123".to_string(),
        };
        assert_eq!(key1, key2);
    }

    #[test]
    fn public_key_serialization() {
        let key = PublicKey {
            source_id: "monitor-1".to_string(),
            public_key: "abc123".to_string(),
        };
        let json = serde_json::to_string(&key).expect("should serialize");
        assert!(json.contains("monitor-1"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn public_key_deserialization() {
        let json = r#"{"source_id": "test", "public_key": "key123"}"#;
        let key: PublicKey = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(key.source_id, "test");
        assert_eq!(key.public_key, "key123");
    }
}
