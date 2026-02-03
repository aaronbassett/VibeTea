//! Integration tests for environment variable key loading.
//!
//! These tests verify FR-001 (load Ed25519 private key from `VIBETEA_PRIVATE_KEY` env var),
//! FR-002 (env var takes precedence over file), FR-004 (clear error messages),
//! FR-005 (whitespace trimming), FR-021 (standard Base64 RFC 4648),
//! FR-022 (validate 32-byte key length), and FR-027/FR-028 (round-trip verification).
//!
//! # Important Notes
//!
//! These tests modify environment variables and MUST be run with `--test-threads=1`
//! or use the `serial_test` crate to prevent interference between tests.

use base64::prelude::*;
use ed25519_dalek::Verifier;
use serial_test::serial;
use std::env;
use tempfile::TempDir;
use vibetea_monitor::crypto::{Crypto, KeySource};

// =============================================================================
// Test Helpers
// =============================================================================

/// Environment variable name for the private key.
const ENV_VAR_NAME: &str = "VIBETEA_PRIVATE_KEY";

/// RAII guard that saves and restores an environment variable.
///
/// When dropped, the guard restores the environment variable to its
/// original value (or removes it if it was not set).
struct EnvGuard {
    name: String,
    original: Option<String>,
}

impl EnvGuard {
    /// Creates a new guard that saves the current value of the env var.
    fn new(name: &str) -> Self {
        let original = env::var(name).ok();
        Self {
            name: name.to_string(),
            original,
        }
    }

    /// Sets the environment variable to a new value.
    fn set(&self, value: &str) {
        env::set_var(&self.name, value);
    }

    /// Removes the environment variable.
    fn remove(&self) {
        env::remove_var(&self.name);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(val) => env::set_var(&self.name, val),
            None => env::remove_var(&self.name),
        }
    }
}

/// Generates a valid 32-byte seed and returns it base64-encoded.
fn generate_valid_base64_seed() -> (String, [u8; 32]) {
    // Generate random bytes for the seed
    let mut seed = [0u8; 32];
    use rand::Rng;
    rand::rng().fill(&mut seed);
    let base64_seed = BASE64_STANDARD.encode(&seed);
    (base64_seed, seed)
}

// =============================================================================
// FR-001: Load Ed25519 private key seed from VIBETEA_PRIVATE_KEY env var
// =============================================================================

/// Verifies that a valid base64-encoded 32-byte seed can be loaded from
/// the `VIBETEA_PRIVATE_KEY` environment variable.
///
/// FR-001: Load Ed25519 private key seed from `VIBETEA_PRIVATE_KEY` env var
/// as base64-encoded string.
#[test]
#[serial]
fn load_valid_base64_key_from_env() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let (base64_seed, _seed) = generate_valid_base64_seed();

    guard.set(&base64_seed);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "Should load valid base64-encoded key: {:?}",
        result.err()
    );

    let (crypto, source) = result.unwrap();
    assert_eq!(
        source,
        KeySource::EnvironmentVariable,
        "Key source should indicate environment variable"
    );

    // Verify the crypto instance is functional
    let pubkey = crypto.public_key_base64();
    assert!(!pubkey.is_empty(), "Public key should not be empty");
}

/// Verifies that `load_from_env` returns an error when the environment
/// variable is not set.
#[test]
#[serial]
fn load_from_env_returns_error_when_not_set() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let result = Crypto::load_from_env();
    assert!(
        result.is_err(),
        "Should return error when env var is not set"
    );
}

// =============================================================================
// FR-005: Whitespace trimming
// =============================================================================

/// Verifies that leading and trailing whitespace is trimmed from the
/// environment variable value before base64 decoding.
///
/// FR-005: Trim whitespace from env var value before decoding.
#[test]
#[serial]
fn whitespace_is_trimmed_from_env_value() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let (base64_seed, seed) = generate_valid_base64_seed();

    // Test with leading/trailing spaces
    let padded_value = format!("   {}   ", base64_seed);
    guard.set(&padded_value);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "Should trim spaces: {:?}",
        result.err()
    );
}

/// Verifies that newlines are trimmed from the environment variable value.
///
/// This handles the common case where the key is stored in a file and
/// read with `cat` or similar, which may include a trailing newline.
#[test]
#[serial]
fn newlines_are_trimmed_from_env_value() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let (base64_seed, _seed) = generate_valid_base64_seed();

    // Test with trailing newline (common when setting from file)
    let with_newline = format!("{}\n", base64_seed);
    guard.set(&with_newline);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "Should trim trailing newline: {:?}",
        result.err()
    );

    // Test with multiple newlines and mixed whitespace
    let with_mixed = format!("\n  {}\n\n  ", base64_seed);
    guard.set(&with_mixed);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "Should trim mixed whitespace and newlines: {:?}",
        result.err()
    );
}

/// Verifies that carriage return characters are also trimmed.
///
/// This handles cross-platform scenarios where files may have CRLF line endings.
#[test]
#[serial]
fn carriage_returns_are_trimmed() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let (base64_seed, _seed) = generate_valid_base64_seed();

    // Test with CRLF ending
    let with_crlf = format!("{}\r\n", base64_seed);
    guard.set(&with_crlf);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "Should trim CRLF: {:?}",
        result.err()
    );
}

// =============================================================================
// FR-004: Clear error messages for invalid keys
// =============================================================================

/// Verifies that an invalid base64 string produces a clear error message.
///
/// FR-004: Clear error messages for invalid keys.
/// FR-021: Standard Base64 (RFC 4648).
#[test]
#[serial]
fn invalid_base64_produces_clear_error() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // Invalid base64 characters
    guard.set("not!valid@base64#");

    let result = Crypto::load_from_env();
    assert!(result.is_err(), "Should reject invalid base64");

    let err = result.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("base64") || err_msg.contains("decode") || err_msg.contains("invalid"),
        "Error message should indicate base64 decoding failure: {err}"
    );
}

/// Verifies that valid base64 with wrong padding produces an error.
#[test]
#[serial]
fn invalid_base64_padding_produces_error() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // Valid characters but invalid padding
    guard.set("YWJjZGVm===");

    let result = Crypto::load_from_env();
    assert!(result.is_err(), "Should reject invalid base64 padding");
}

// =============================================================================
// FR-022: Validate decoded key is exactly 32 bytes
// =============================================================================

/// Verifies that a key shorter than 32 bytes produces a clear error.
///
/// FR-022: Validate decoded key is exactly 32 bytes.
#[test]
#[serial]
fn short_key_produces_clear_error() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // 16 bytes instead of 32 (valid base64, wrong length)
    let short_key = BASE64_STANDARD.encode(&[0u8; 16]);
    guard.set(&short_key);

    let result = Crypto::load_from_env();
    assert!(result.is_err(), "Should reject key shorter than 32 bytes");

    let err = result.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("32") || err_msg.contains("byte") || err_msg.contains("length"),
        "Error message should indicate wrong key length: {err}"
    );
}

/// Verifies that a key longer than 32 bytes produces a clear error.
///
/// FR-022: Validate decoded key is exactly 32 bytes.
#[test]
#[serial]
fn long_key_produces_clear_error() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // 64 bytes instead of 32 (valid base64, wrong length)
    let long_key = BASE64_STANDARD.encode(&[0u8; 64]);
    guard.set(&long_key);

    let result = Crypto::load_from_env();
    assert!(result.is_err(), "Should reject key longer than 32 bytes");

    let err = result.unwrap_err();
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("32") || err_msg.contains("byte") || err_msg.contains("length"),
        "Error message should indicate wrong key length: {err}"
    );
}

/// Verifies that an empty environment variable value produces an error.
#[test]
#[serial]
fn empty_env_value_produces_error() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.set("");

    let result = Crypto::load_from_env();
    assert!(
        result.is_err(),
        "Should reject empty environment variable value"
    );
}

/// Verifies that whitespace-only environment variable produces an error.
#[test]
#[serial]
fn whitespace_only_env_value_produces_error() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.set("   \n\t  ");

    let result = Crypto::load_from_env();
    assert!(
        result.is_err(),
        "Should reject whitespace-only environment variable value"
    );
}

// =============================================================================
// FR-002: Env var takes precedence over file-based key
// =============================================================================

/// Verifies that when both environment variable and file-based key exist,
/// the environment variable takes precedence.
///
/// FR-002: Env var takes precedence over file-based key.
#[test]
#[serial]
fn env_var_takes_precedence_over_file() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save a key to file
    let file_crypto = Crypto::generate();
    file_crypto.save(temp_dir.path()).expect("Failed to save key to file");
    let file_pubkey = file_crypto.public_key_base64();

    // Generate a different key for the environment variable
    let (env_base64, _env_seed) = generate_valid_base64_seed();
    guard.set(&env_base64);

    // Load with both sources available - should use env var
    let result = Crypto::load_with_fallback(temp_dir.path());
    assert!(result.is_ok(), "Should load key: {:?}", result.err());

    let (loaded_crypto, source) = result.unwrap();

    // Verify env var was used (source should be EnvironmentVariable)
    assert_eq!(
        source,
        KeySource::EnvironmentVariable,
        "Should indicate environment variable as source"
    );

    // Verify the key is different from the file-based key
    let loaded_pubkey = loaded_crypto.public_key_base64();
    assert_ne!(
        loaded_pubkey, file_pubkey,
        "Should use env var key, not file key"
    );
}

/// Verifies that when only file-based key exists (no env var), the file is used.
#[test]
#[serial]
fn file_key_used_when_env_var_not_set() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove(); // Ensure env var is not set

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save a key to file
    let file_crypto = Crypto::generate();
    file_crypto.save(temp_dir.path()).expect("Failed to save key to file");
    let file_pubkey = file_crypto.public_key_base64();

    // Load with only file source available
    let result = Crypto::load_with_fallback(temp_dir.path());
    assert!(result.is_ok(), "Should load key from file: {:?}", result.err());

    let (loaded_crypto, source) = result.unwrap();

    // Verify file was used
    assert_eq!(
        source,
        KeySource::File(temp_dir.path().join("key.priv")),
        "Should indicate file as source"
    );

    // Verify the key matches the file-based key
    let loaded_pubkey = loaded_crypto.public_key_base64();
    assert_eq!(
        loaded_pubkey, file_pubkey,
        "Should use file key when env var not set"
    );
}

/// Verifies that when env var is set but invalid, and file exists,
/// loading fails (env var error takes precedence, not fallback to file).
#[test]
#[serial]
fn invalid_env_var_does_not_fallback_to_file() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save a valid key to file
    let file_crypto = Crypto::generate();
    file_crypto.save(temp_dir.path()).expect("Failed to save key to file");

    // Set invalid env var
    guard.set("invalid_base64!");

    // Load should fail with env var error, not fallback to file
    let result = Crypto::load_with_fallback(temp_dir.path());
    assert!(
        result.is_err(),
        "Should fail with env var error, not silently fallback to file"
    );
}

// =============================================================================
// FR-027/FR-028: Round-trip verification
// =============================================================================

/// Verifies the complete round-trip: generate key, get seed bytes,
/// base64 encode, set as env var, load from env, sign message, verify signature.
///
/// FR-027: Export private key seed as base64.
/// FR-028: Round-trip test (export -> env load -> sign -> verify).
#[test]
#[serial]
fn roundtrip_generate_export_import_sign_verify() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // Step 1: Generate a new keypair
    let original_crypto = Crypto::generate();
    let original_pubkey = original_crypto.public_key_base64();

    // Step 2: Export the seed as base64 (using the method that should exist)
    let seed_base64 = original_crypto.seed_base64();

    // Step 3: Set as environment variable
    guard.set(&seed_base64);

    // Step 4: Load from environment variable
    let result = Crypto::load_from_env();
    assert!(result.is_ok(), "Should load exported key: {:?}", result.err());

    let (loaded_crypto, source) = result.unwrap();
    assert_eq!(source, KeySource::EnvironmentVariable);

    // Step 5: Verify public keys match
    let loaded_pubkey = loaded_crypto.public_key_base64();
    assert_eq!(
        original_pubkey, loaded_pubkey,
        "Public keys should match after round-trip"
    );

    // Step 6: Sign a message with the loaded key
    let message = b"test message for round-trip verification";
    let signature = loaded_crypto.sign(message);

    // Step 7: Verify the signature using the original verifying key
    let signature_bytes = BASE64_STANDARD
        .decode(&signature)
        .expect("Failed to decode signature");
    let sig = ed25519_dalek::Signature::from_slice(&signature_bytes)
        .expect("Failed to parse signature");

    let verification_result = original_crypto.verifying_key().verify(message, &sig);
    assert!(
        verification_result.is_ok(),
        "Signature verification should succeed: {:?}",
        verification_result.err()
    );
}

/// Verifies that signatures from the original and loaded keys are identical
/// (Ed25519 is deterministic).
#[test]
#[serial]
fn roundtrip_signatures_are_identical() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // Generate and export
    let original_crypto = Crypto::generate();
    let seed_base64 = original_crypto.seed_base64();

    // Load from env
    guard.set(&seed_base64);
    let (loaded_crypto, _) = Crypto::load_from_env().expect("Should load key");

    // Sign same message with both
    let message = b"deterministic signature test";
    let original_sig = original_crypto.sign(message);
    let loaded_sig = loaded_crypto.sign(message);

    // Ed25519 is deterministic - signatures should be identical
    assert_eq!(
        original_sig, loaded_sig,
        "Signatures should be identical for same key and message"
    );
}

// =============================================================================
// Edge Cases and Error Handling
// =============================================================================

/// Verifies that standard base64 (RFC 4648) is required, not URL-safe base64.
///
/// FR-021: Standard Base64 (RFC 4648).
#[test]
#[serial]
fn rejects_url_safe_base64() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    // Create a seed that produces different encodings between standard and URL-safe
    // Use bytes that contain values that encode to '+' and '/' in standard base64
    let mut seed = [0u8; 32];
    // 0xfb encodes with '+' in standard base64, 0xff encodes with '/'
    for i in 0..16 {
        seed[i * 2] = 0xfb;
        seed[i * 2 + 1] = 0xff;
    }

    // URL-safe base64 uses '-' and '_' instead of '+' and '/'
    let url_safe_encoded = base64::prelude::BASE64_URL_SAFE.encode(&seed);

    // Only set if URL-safe encoding differs from standard (it should for this seed)
    let standard_encoded = BASE64_STANDARD.encode(&seed);
    if url_safe_encoded != standard_encoded {
        guard.set(&url_safe_encoded);

        // Should fail because we expect standard base64
        let result = Crypto::load_from_env();
        // Note: This may actually succeed if the URL-safe characters aren't present
        // The test documents the expected behavior
        if url_safe_encoded.contains('-') || url_safe_encoded.contains('_') {
            assert!(
                result.is_err(),
                "Should reject URL-safe base64 encoding: {:?}",
                result
            );
        }
    }
}

/// Verifies that a key with correct length but all zeros works.
/// (All-zero seed is technically valid for Ed25519)
#[test]
#[serial]
fn all_zero_seed_is_valid() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    let zero_seed = [0u8; 32];
    let base64_seed = BASE64_STANDARD.encode(&zero_seed);
    guard.set(&base64_seed);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "All-zero seed should be valid (though not recommended): {:?}",
        result.err()
    );
}

/// Verifies that a key with all 0xFF bytes works.
#[test]
#[serial]
fn all_ff_seed_is_valid() {
    let guard = EnvGuard::new(ENV_VAR_NAME);

    let ff_seed = [0xffu8; 32];
    let base64_seed = BASE64_STANDARD.encode(&ff_seed);
    guard.set(&base64_seed);

    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "All-0xFF seed should be valid: {:?}",
        result.err()
    );
}

// =============================================================================
// KeySource Verification
// =============================================================================

/// Verifies that KeySource::EnvironmentVariable is returned when loading from env.
#[test]
#[serial]
fn key_source_is_environment_variable_when_loaded_from_env() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let (base64_seed, _) = generate_valid_base64_seed();
    guard.set(&base64_seed);

    let (_, source) = Crypto::load_from_env().expect("Should load key");

    assert!(
        matches!(source, KeySource::EnvironmentVariable),
        "Source should be EnvironmentVariable, got: {:?}",
        source
    );
}

/// Verifies that KeySource::File contains the correct path when loading from file.
#[test]
#[serial]
fn key_source_is_file_with_correct_path_when_loaded_from_file() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove(); // Ensure env var is not set

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let expected_path = temp_dir.path().join("key.priv");

    let crypto = Crypto::generate();
    crypto.save(temp_dir.path()).expect("Failed to save key");

    let (_, source) = Crypto::load_with_fallback(temp_dir.path()).expect("Should load key");

    assert_eq!(
        source,
        KeySource::File(expected_path),
        "Source should be File with correct path"
    );
}
