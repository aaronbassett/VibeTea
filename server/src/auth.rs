//! Authentication module for Ed25519 signature verification.
//!
//! This module provides cryptographic signature verification for Monitor
//! authentication. Monitors sign their event payloads with Ed25519 private keys,
//! and the server verifies these signatures using pre-registered public keys.
//!
//! # Overview
//!
//! The authentication flow works as follows:
//! 1. Each Monitor is assigned a unique `source_id` and generates an Ed25519 key pair
//! 2. The public key is registered with the server via `VIBETEA_PUBLIC_KEYS` config
//! 3. When submitting events, Monitors sign the message body and include:
//!    - `X-Source-ID` header: The monitor's unique identifier
//!    - `X-Signature` header: Base64-encoded Ed25519 signature
//! 4. The server verifies the signature against the registered public key
//!
//! # Example
//!
//! ```rust
//! use std::collections::HashMap;
//! use vibetea_server::auth::{verify_signature, AuthError};
//!
//! // Registered public keys from configuration
//! let mut public_keys = HashMap::new();
//! public_keys.insert(
//!     "monitor-1".to_string(),
//!     "MCowBQYDK2VwAyEAbase64encodedpublickey".to_string(), // example key
//! );
//!
//! // Verify a signature (would fail with this example data)
//! let result = verify_signature(
//!     "monitor-1",
//!     "base64signature",
//!     b"message to verify",
//!     &public_keys,
//! );
//! // Returns Ok(()) if valid, Err(AuthError) if invalid
//! ```

use std::collections::HashMap;

use base64::prelude::*;
use ed25519_dalek::{Signature, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use thiserror::Error;

/// Errors that can occur during signature verification.
///
/// These errors provide detailed information about why authentication failed,
/// allowing for appropriate HTTP status codes and error messages.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// The source_id is not registered in the server's public key configuration.
    ///
    /// This typically means the Monitor has not been provisioned or is using
    /// an incorrect source identifier.
    #[error("unknown source: {0}")]
    UnknownSource(String),

    /// The signature verification failed.
    ///
    /// The signature was properly formatted but did not match the message
    /// and public key combination. This could indicate:
    /// - The message was tampered with in transit
    /// - The wrong private key was used to sign
    /// - The public key on the server is outdated
    #[error("invalid signature")]
    InvalidSignature,

    /// Base64 decoding failed for the specified field.
    ///
    /// The signature or public key contains invalid base64 characters
    /// or has incorrect padding.
    #[error("invalid base64 encoding for {0}")]
    InvalidBase64(String),

    /// The public key bytes are malformed.
    ///
    /// The decoded public key is not a valid Ed25519 public key,
    /// typically because it has the wrong length or invalid point encoding.
    #[error("invalid public key format")]
    InvalidPublicKey,
}

impl AuthError {
    /// Creates an error for an unknown source identifier.
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::auth::AuthError;
    ///
    /// let err = AuthError::unknown_source("monitor-unknown");
    /// assert!(matches!(err, AuthError::UnknownSource(_)));
    /// ```
    pub fn unknown_source(source_id: impl Into<String>) -> Self {
        Self::UnknownSource(source_id.into())
    }

    /// Creates an error for invalid base64 encoding.
    ///
    /// # Arguments
    ///
    /// * `field` - The name of the field that failed to decode (e.g., "signature", "public_key")
    ///
    /// # Example
    ///
    /// ```rust
    /// use vibetea_server::auth::AuthError;
    ///
    /// let err = AuthError::invalid_base64("signature");
    /// assert!(matches!(err, AuthError::InvalidBase64(_)));
    /// ```
    pub fn invalid_base64(field: impl Into<String>) -> Self {
        Self::InvalidBase64(field.into())
    }

    /// Returns `true` if this error indicates the source is not registered.
    pub fn is_unknown_source(&self) -> bool {
        matches!(self, Self::UnknownSource(_))
    }

    /// Returns `true` if this error indicates a cryptographic verification failure.
    pub fn is_signature_error(&self) -> bool {
        matches!(self, Self::InvalidSignature)
    }

    /// Returns `true` if this error indicates malformed input data.
    pub fn is_format_error(&self) -> bool {
        matches!(self, Self::InvalidBase64(_) | Self::InvalidPublicKey)
    }
}

/// Verifies an Ed25519 signature for a given message.
///
/// This function performs the complete signature verification process:
/// 1. Looks up the public key for the given source_id
/// 2. Decodes the base64-encoded public key and signature
/// 3. Verifies the signature against the message
///
/// # Arguments
///
/// * `source_id` - The unique identifier of the signing Monitor
/// * `signature_base64` - The base64-encoded Ed25519 signature
/// * `message` - The original message bytes that were signed
/// * `public_keys` - Map of source_id to base64-encoded public keys
///
/// # Returns
///
/// * `Ok(())` - The signature is valid
/// * `Err(AuthError)` - Verification failed (see [`AuthError`] variants)
///
/// # Example
///
/// ```rust,ignore
/// use std::collections::HashMap;
/// use vibetea_server::auth::verify_signature;
///
/// let public_keys: HashMap<String, String> = load_from_config();
/// let result = verify_signature(
///     "monitor-1",
///     "base64-signature-from-header",
///     request_body.as_bytes(),
///     &public_keys,
/// );
///
/// match result {
///     Ok(()) => println!("Signature verified!"),
///     Err(e) => eprintln!("Authentication failed: {}", e),
/// }
/// ```
///
/// # Security Considerations
///
/// - This function uses constant-time comparison for signature verification
/// - Public keys should be loaded from trusted configuration, not user input
/// - The message should be the exact bytes that were signed (typically the request body)
pub fn verify_signature(
    source_id: &str,
    signature_base64: &str,
    message: &[u8],
    public_keys: &HashMap<String, String>,
) -> Result<(), AuthError> {
    // Look up the public key for this source
    let public_key_base64 = public_keys
        .get(source_id)
        .ok_or_else(|| AuthError::unknown_source(source_id))?;

    // Decode the base64-encoded public key
    let public_key_bytes = BASE64_STANDARD
        .decode(public_key_base64)
        .map_err(|_| AuthError::invalid_base64("public_key"))?;

    // Verify the public key has the correct length
    let public_key_array: [u8; PUBLIC_KEY_LENGTH] = public_key_bytes
        .try_into()
        .map_err(|_| AuthError::InvalidPublicKey)?;

    // Parse the public key
    let verifying_key =
        VerifyingKey::from_bytes(&public_key_array).map_err(|_| AuthError::InvalidPublicKey)?;

    // Decode the base64-encoded signature
    let signature_bytes = BASE64_STANDARD
        .decode(signature_base64)
        .map_err(|_| AuthError::invalid_base64("signature"))?;

    // Verify the signature has the correct length
    let signature_array: [u8; SIGNATURE_LENGTH] = signature_bytes
        .try_into()
        .map_err(|_| AuthError::InvalidSignature)?;

    // Parse and verify the signature
    let signature = Signature::from_bytes(&signature_array);

    verifying_key
        .verify_strict(message, &signature)
        .map_err(|_| AuthError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey, SECRET_KEY_LENGTH};

    /// Creates a test key pair from a deterministic seed.
    ///
    /// Using deterministic seeds makes tests reproducible. The seed is expanded
    /// to fill the 32-byte private key requirement.
    fn create_test_keypair(seed: u8) -> (SigningKey, String) {
        let mut seed_bytes = [0u8; SECRET_KEY_LENGTH];
        for (i, byte) in seed_bytes.iter_mut().enumerate() {
            *byte = seed.wrapping_add(i as u8);
        }
        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let public_key_base64 = BASE64_STANDARD.encode(public_key_bytes);
        (signing_key, public_key_base64)
    }

    /// Helper to generate a test key pair (deterministic, seed 1).
    fn generate_test_keypair() -> (SigningKey, String) {
        create_test_keypair(1)
    }

    /// Helper to generate a second distinct test key pair (deterministic, seed 100).
    fn generate_test_keypair_alt() -> (SigningKey, String) {
        create_test_keypair(100)
    }

    /// Helper to create a public keys map with a single entry.
    fn create_keys_map(source_id: &str, public_key_base64: &str) -> HashMap<String, String> {
        let mut keys = HashMap::new();
        keys.insert(source_id.to_string(), public_key_base64.to_string());
        keys
    }

    #[test]
    fn verify_signature_succeeds_for_valid_signature() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let message = b"test message to sign";
        let signature = signing_key.sign(message);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature("monitor-1", &signature_base64, message, &public_keys);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_signature_succeeds_for_empty_message() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let message = b"";
        let signature = signing_key.sign(message);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature("monitor-1", &signature_base64, message, &public_keys);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_signature_succeeds_for_large_message() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        // Create a large message (1MB)
        let message: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
        let signature = signing_key.sign(&message);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature("monitor-1", &signature_base64, &message, &public_keys);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_signature_fails_for_unknown_source() {
        let (_, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let result = verify_signature(
            "unknown-monitor",
            "c29tZXNpZ25hdHVyZQ==",
            b"message",
            &public_keys,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::UnknownSource(ref s) if s == "unknown-monitor"));
        assert!(err.is_unknown_source());
        assert_eq!(err.to_string(), "unknown source: unknown-monitor");
    }

    #[test]
    fn verify_signature_fails_for_wrong_signature() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        // Sign a different message
        let signature = signing_key.sign(b"different message");
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature(
            "monitor-1",
            &signature_base64,
            b"actual message",
            &public_keys,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidSignature));
        assert!(err.is_signature_error());
        assert_eq!(err.to_string(), "invalid signature");
    }

    #[test]
    fn verify_signature_fails_for_wrong_public_key() {
        let (signing_key, _) = generate_test_keypair();
        let (_, wrong_public_key_base64) = generate_test_keypair_alt();
        let public_keys = create_keys_map("monitor-1", &wrong_public_key_base64);

        let message = b"test message";
        let signature = signing_key.sign(message);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature("monitor-1", &signature_base64, message, &public_keys);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidSignature));
    }

    #[test]
    fn verify_signature_fails_for_tampered_message() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let original_message = b"original message";
        let signature = signing_key.sign(original_message);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        // Try to verify with a tampered message
        let tampered_message = b"tampered message";
        let result = verify_signature(
            "monitor-1",
            &signature_base64,
            tampered_message,
            &public_keys,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidSignature));
    }

    #[test]
    fn verify_signature_fails_for_invalid_signature_base64() {
        let (_, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let result = verify_signature("monitor-1", "not-valid-base64!!!", b"message", &public_keys);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidBase64(ref s) if s == "signature"));
        assert!(err.is_format_error());
        assert_eq!(err.to_string(), "invalid base64 encoding for signature");
    }

    #[test]
    fn verify_signature_fails_for_invalid_public_key_base64() {
        let mut public_keys = HashMap::new();
        public_keys.insert("monitor-1".to_string(), "not-valid-base64!!!".to_string());

        let result = verify_signature(
            "monitor-1",
            "c29tZXNpZ25hdHVyZQ==",
            b"message",
            &public_keys,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidBase64(ref s) if s == "public_key"));
        assert_eq!(err.to_string(), "invalid base64 encoding for public_key");
    }

    #[test]
    fn verify_signature_fails_for_wrong_length_public_key() {
        // Base64 encoding of only 16 bytes (should be 32)
        let short_key = BASE64_STANDARD.encode([0u8; 16]);
        let public_keys = create_keys_map("monitor-1", &short_key);

        let result = verify_signature(
            "monitor-1",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            b"message",
            &public_keys,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::InvalidPublicKey));
        assert!(err.is_format_error());
        assert_eq!(err.to_string(), "invalid public key format");
    }

    #[test]
    fn verify_signature_fails_for_wrong_length_signature() {
        let (_, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        // Base64 encoding of only 32 bytes (should be 64)
        let short_signature = BASE64_STANDARD.encode([0u8; 32]);

        let result = verify_signature("monitor-1", &short_signature, b"message", &public_keys);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidSignature));
    }

    #[test]
    fn verify_signature_fails_for_identity_public_key() {
        // The identity point (all zeros) is technically a valid curve point
        // but any signature verification against it will fail.
        let identity_key = BASE64_STANDARD.encode([0u8; 32]);
        let public_keys = create_keys_map("monitor-1", &identity_key);

        let result = verify_signature(
            "monitor-1",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            b"message",
            &public_keys,
        );

        assert!(result.is_err());
        // Verification against the identity point fails with InvalidSignature
        assert!(matches!(result.unwrap_err(), AuthError::InvalidSignature));
    }

    #[test]
    fn verify_signature_works_with_multiple_sources() {
        let (signing_key_1, public_key_1) = generate_test_keypair();
        let (signing_key_2, public_key_2) = generate_test_keypair_alt();

        let mut public_keys = HashMap::new();
        public_keys.insert("monitor-1".to_string(), public_key_1);
        public_keys.insert("monitor-2".to_string(), public_key_2);

        let message = b"shared message";

        // Verify signature from monitor-1
        let sig_1 = signing_key_1.sign(message);
        let sig_1_base64 = BASE64_STANDARD.encode(sig_1.to_bytes());
        assert!(verify_signature("monitor-1", &sig_1_base64, message, &public_keys).is_ok());

        // Verify signature from monitor-2
        let sig_2 = signing_key_2.sign(message);
        let sig_2_base64 = BASE64_STANDARD.encode(sig_2.to_bytes());
        assert!(verify_signature("monitor-2", &sig_2_base64, message, &public_keys).is_ok());

        // Cross-verification should fail (monitor-1's signature with monitor-2's key)
        assert!(verify_signature("monitor-2", &sig_1_base64, message, &public_keys).is_err());
        assert!(verify_signature("monitor-1", &sig_2_base64, message, &public_keys).is_err());
    }

    #[test]
    fn verify_signature_with_json_payload() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        // Simulate a real JSON event payload
        let json_payload = r#"{"id":"evt-123","source":"monitor-1","type":"activity","payload":{"session_id":"abc"}}"#;
        let signature = signing_key.sign(json_payload.as_bytes());
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature(
            "monitor-1",
            &signature_base64,
            json_payload.as_bytes(),
            &public_keys,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn auth_error_unknown_source_helper() {
        let err = AuthError::unknown_source("test-source");
        assert!(matches!(err, AuthError::UnknownSource(ref s) if s == "test-source"));
    }

    #[test]
    fn auth_error_invalid_base64_helper() {
        let err = AuthError::invalid_base64("field_name");
        assert!(matches!(err, AuthError::InvalidBase64(ref s) if s == "field_name"));
    }

    #[test]
    fn auth_error_is_methods() {
        assert!(AuthError::UnknownSource("x".into()).is_unknown_source());
        assert!(!AuthError::InvalidSignature.is_unknown_source());

        assert!(AuthError::InvalidSignature.is_signature_error());
        assert!(!AuthError::UnknownSource("x".into()).is_signature_error());

        assert!(AuthError::InvalidBase64("x".into()).is_format_error());
        assert!(AuthError::InvalidPublicKey.is_format_error());
        assert!(!AuthError::InvalidSignature.is_format_error());
        assert!(!AuthError::UnknownSource("x".into()).is_format_error());
    }

    #[test]
    fn auth_error_is_clone_and_eq() {
        let err1 = AuthError::UnknownSource("test".into());
        let err2 = err1.clone();
        assert_eq!(err1, err2);

        let err3 = AuthError::InvalidSignature;
        let err4 = err3.clone();
        assert_eq!(err3, err4);
    }

    #[test]
    fn auth_error_is_debug() {
        let err = AuthError::InvalidSignature;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("InvalidSignature"));
    }

    #[test]
    fn auth_error_display_all_variants() {
        assert_eq!(
            AuthError::UnknownSource("src".into()).to_string(),
            "unknown source: src"
        );
        assert_eq!(AuthError::InvalidSignature.to_string(), "invalid signature");
        assert_eq!(
            AuthError::InvalidBase64("sig".into()).to_string(),
            "invalid base64 encoding for sig"
        );
        assert_eq!(
            AuthError::InvalidPublicKey.to_string(),
            "invalid public key format"
        );
    }
}
