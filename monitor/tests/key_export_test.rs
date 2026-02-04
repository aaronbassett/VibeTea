//! Integration tests for the `export-key` subcommand.
//!
//! These tests verify the following requirements:
//!
//! - **FR-003**: Monitor MUST provide `export-key` subcommand to output ONLY the base64-encoded
//!   private key followed by a single newline (no additional text), enabling direct piping to
//!   clipboard or secret management tools.
//!
//! - **FR-023**: All diagnostic and error messages from `export-key` MUST go to stderr; only the
//!   key itself goes to stdout.
//!
//! - **FR-026**: Exit codes: 0 for success, 1 for configuration error (invalid env var, missing
//!   key), 2 for runtime error.
//!
//! - **FR-027**: Integration tests MUST verify that a key exported with `export-key` can be loaded
//!   via `VIBETEA_PRIVATE_KEY`.
//!
//! - **FR-028**: Integration tests MUST verify round-trip: generate key, export, load from env var,
//!   verify signing produces valid signatures.
//!
//! # Important Notes
//!
//! These tests modify environment variables and MUST be run with `--test-threads=1`
//! or use the `serial_test` crate to prevent interference between tests.
//!
//! # Test Status
//!
//! These tests are currently expected to FAIL because the `export-key` command has not been
//! implemented yet. The tests define the expected behavior based on the specification.

use base64::prelude::*;
use ed25519_dalek::Verifier;
use serial_test::serial;
use std::env;
use std::process::Command;
use tempfile::TempDir;
use vibetea_monitor::crypto::Crypto;

// =============================================================================
// Test Helpers
// =============================================================================

/// Environment variable name for the private key.
const ENV_VAR_NAME: &str = "VIBETEA_PRIVATE_KEY";

/// Exit code for successful execution.
const EXIT_SUCCESS: i32 = 0;

/// Exit code for configuration errors (invalid env var, missing key).
const EXIT_CONFIG_ERROR: i32 = 1;

/// Exit code for runtime errors.
#[allow(dead_code)]
const EXIT_RUNTIME_ERROR: i32 = 2;

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

/// Builds and returns the path to the vibetea-monitor binary.
///
/// This function assumes that `cargo build` has been run and the binary
/// is available in the target/debug directory.
fn get_monitor_binary_path() -> String {
    // Use cargo to find the binary path
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let target_dir = std::path::Path::new(manifest_dir)
        .parent()
        .expect("Should have parent directory")
        .join("target")
        .join("debug")
        .join("vibetea-monitor");
    target_dir.to_string_lossy().to_string()
}

/// Runs the vibetea-monitor export-key command with the given path.
///
/// Returns the command output (stdout, stderr, exit code).
fn run_export_key_command(key_path: &std::path::Path) -> std::process::Output {
    Command::new(get_monitor_binary_path())
        .arg("export-key")
        .arg("--path")
        .arg(key_path.to_string_lossy().as_ref())
        .output()
        .expect("Failed to execute vibetea-monitor binary")
}

/// Runs the vibetea-monitor export-key command without a path (uses default).
///
/// Returns the command output (stdout, stderr, exit code).
#[allow(dead_code)]
fn run_export_key_command_default() -> std::process::Output {
    Command::new(get_monitor_binary_path())
        .arg("export-key")
        .output()
        .expect("Failed to execute vibetea-monitor binary")
}

// =============================================================================
// FR-027/FR-028: Round-trip verification with export-key command
// =============================================================================

/// Verifies the complete round-trip using the export-key command:
/// 1. Generate key with `Crypto::generate()`
/// 2. Save key to file
/// 3. Export via `export-key` command
/// 4. Load via `VIBETEA_PRIVATE_KEY` environment variable
/// 5. Sign and verify
///
/// **Covers:** FR-003, FR-027, FR-028
#[test]
#[serial]
fn roundtrip_generate_export_command_import_sign_verify() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove(); // Ensure env var is not set initially

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Step 1 & 2: Generate a new keypair and save it
    let original_crypto = Crypto::generate();
    original_crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");
    let original_pubkey = original_crypto.public_key_base64();

    // Step 3: Export via export-key command
    let output = run_export_key_command(temp_dir.path());

    // The command should succeed with exit code 0
    assert!(
        output.status.success(),
        "export-key should exit with code 0, got: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 4: Get the exported key from stdout and set as env var
    let exported_key =
        String::from_utf8(output.stdout.clone()).expect("stdout should be valid UTF-8");
    let exported_key_trimmed = exported_key.trim();

    // Verify the exported key matches the original seed
    let original_seed = original_crypto.seed_base64();
    assert_eq!(
        exported_key_trimmed, original_seed,
        "Exported key should match the original seed"
    );

    // Set the exported key as environment variable
    guard.set(exported_key_trimmed);

    // Load from environment variable
    let result = Crypto::load_from_env();
    assert!(
        result.is_ok(),
        "Should load exported key from env var: {:?}",
        result.err()
    );

    let (loaded_crypto, _source) = result.unwrap();

    // Step 5: Verify public keys match
    let loaded_pubkey = loaded_crypto.public_key_base64();
    assert_eq!(
        original_pubkey, loaded_pubkey,
        "Public keys should match after round-trip via export-key command"
    );

    // Sign a message with the loaded key
    let message = b"test message for export-key round-trip verification";
    let signature = loaded_crypto.sign(message);

    // Verify the signature using the original verifying key
    let signature_bytes = BASE64_STANDARD
        .decode(&signature)
        .expect("Failed to decode signature");
    let sig =
        ed25519_dalek::Signature::from_slice(&signature_bytes).expect("Failed to parse signature");

    let verification_result = original_crypto.verifying_key().verify(message, &sig);
    assert!(
        verification_result.is_ok(),
        "Signature verification should succeed after export-key round-trip: {:?}",
        verification_result.err()
    );
}

/// Verifies that signatures from the original key and the key loaded via
/// export-key are identical (Ed25519 is deterministic).
///
/// **Covers:** FR-027, FR-028
#[test]
#[serial]
fn roundtrip_export_command_signatures_are_identical() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save key
    let original_crypto = Crypto::generate();
    original_crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");

    // Export via command
    let output = run_export_key_command(temp_dir.path());
    assert!(
        output.status.success(),
        "export-key should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Load from env
    let exported_key = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    guard.set(exported_key.trim());
    let (loaded_crypto, _) = Crypto::load_from_env().expect("Should load key from env");

    // Sign same message with both
    let message = b"deterministic signature test via export-key";
    let original_sig = original_crypto.sign(message);
    let loaded_sig = loaded_crypto.sign(message);

    // Ed25519 is deterministic - signatures should be identical
    assert_eq!(
        original_sig, loaded_sig,
        "Signatures should be identical for same key and message after export-key round-trip"
    );
}

// =============================================================================
// FR-003: Output format verification
// =============================================================================

/// Verifies that the export-key command outputs ONLY the base64-encoded private key
/// followed by a single newline character to stdout.
///
/// **Covers:** FR-003
#[test]
#[serial]
fn export_key_output_format_base64_with_single_newline() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save key
    let crypto = Crypto::generate();
    crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");
    let expected_seed = crypto.seed_base64();

    // Export via command
    let output = run_export_key_command(temp_dir.path());
    assert!(
        output.status.success(),
        "export-key should succeed, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");

    // Verify exact format: base64 key followed by single newline
    let expected_output = format!("{}\n", expected_seed);
    assert_eq!(
        stdout,
        expected_output,
        "Output should be exactly base64 key + single newline.\n\
         Expected: {:?}\n\
         Got: {:?}",
        expected_output.as_bytes(),
        stdout.as_bytes()
    );

    // Verify it ends with exactly one newline
    assert!(stdout.ends_with('\n'), "Output should end with a newline");
    assert!(
        !stdout.ends_with("\n\n"),
        "Output should not end with multiple newlines"
    );

    // Verify no leading/trailing whitespace other than the final newline
    let trimmed = stdout.trim_end_matches('\n');
    assert!(
        !trimmed.starts_with(char::is_whitespace),
        "Output should not have leading whitespace"
    );
    assert!(
        !trimmed.ends_with(char::is_whitespace),
        "Output should not have trailing whitespace before newline"
    );
}

/// Verifies that the output is valid base64 and decodes to exactly 32 bytes.
///
/// **Covers:** FR-003
#[test]
#[serial]
fn export_key_output_is_valid_base64_32_bytes() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save key
    let crypto = Crypto::generate();
    crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");

    // Export via command
    let output = run_export_key_command(temp_dir.path());
    assert!(output.status.success(), "export-key should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let key_base64 = stdout.trim();

    // Verify it's valid base64 that decodes to 32 bytes
    let decoded = BASE64_STANDARD
        .decode(key_base64)
        .expect("Exported key should be valid base64");

    assert_eq!(
        decoded.len(),
        32,
        "Decoded key should be exactly 32 bytes (Ed25519 seed)"
    );
}

// =============================================================================
// FR-023: Stderr for diagnostics, stdout for key only
// =============================================================================

/// Verifies that all diagnostic and informational messages go to stderr,
/// not stdout. Only the key itself should go to stdout.
///
/// **Covers:** FR-023
#[test]
#[serial]
fn export_key_diagnostics_go_to_stderr() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save key
    let crypto = Crypto::generate();
    crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");

    // Export via command
    let output = run_export_key_command(temp_dir.path());
    assert!(output.status.success(), "export-key should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    // stdout should contain ONLY the base64 key + newline
    // No prose, no labels, no prefixes
    let stdout_trimmed = stdout.trim();

    // Check that stdout doesn't contain any common diagnostic patterns
    let diagnostic_patterns = [
        "loading",
        "loaded",
        "key:",
        "path:",
        "exporting",
        "exported",
        "private",
        "success",
        "error",
        "warning",
        "info",
        "debug",
    ];

    for pattern in diagnostic_patterns {
        assert!(
            !stdout_trimmed.to_lowercase().contains(pattern),
            "stdout should not contain diagnostic text like '{}'. \
             Found stdout: {:?}",
            pattern,
            stdout_trimmed
        );
    }

    // Stdout should be purely base64 characters + possible padding
    assert!(
        stdout_trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='),
        "stdout should contain only base64 characters. Found: {:?}",
        stdout_trimmed
    );

    // Note: stderr may or may not have content - if it does, that's fine
    // We just verify stdout is clean
    drop(stderr); // Silence unused warning
}

/// Verifies that error messages are written to stderr (not stdout) when
/// the export-key command fails.
///
/// **Covers:** FR-023, FR-026
#[test]
#[serial]
fn export_key_error_messages_go_to_stderr() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    // Do NOT create a key file - this should cause an error

    // Export via command (should fail - no key exists)
    let output = run_export_key_command(temp_dir.path());

    // Should fail with exit code 1 (configuration error)
    assert!(
        !output.status.success(),
        "export-key should fail when no key exists"
    );
    assert_eq!(
        output.status.code(),
        Some(EXIT_CONFIG_ERROR),
        "Exit code should be {} for missing key",
        EXIT_CONFIG_ERROR
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be valid UTF-8");

    // stdout should be empty when there's an error
    assert!(
        stdout.is_empty(),
        "stdout should be empty on error. Found: {:?}",
        stdout
    );

    // stderr should contain the error message
    assert!(
        !stderr.is_empty(),
        "stderr should contain error message when key is missing"
    );
}

// =============================================================================
// FR-026: Exit code verification
// =============================================================================

/// Verifies that export-key returns exit code 0 on success.
///
/// **Covers:** FR-026
#[test]
#[serial]
fn export_key_exit_code_success() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save key
    let crypto = Crypto::generate();
    crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");

    // Export via command
    let output = run_export_key_command(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(EXIT_SUCCESS),
        "export-key should return exit code {} on success",
        EXIT_SUCCESS
    );
}

/// Verifies that export-key returns exit code 1 when the key file doesn't exist.
///
/// **Covers:** FR-026
#[test]
#[serial]
fn export_key_exit_code_missing_key_file() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    // Do NOT create a key file

    // Export via command (should fail)
    let output = run_export_key_command(temp_dir.path());

    assert_eq!(
        output.status.code(),
        Some(EXIT_CONFIG_ERROR),
        "export-key should return exit code {} for missing key file.\nstderr: {}",
        EXIT_CONFIG_ERROR,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Verifies that export-key returns exit code 1 when pointing to a non-existent directory.
///
/// **Covers:** FR-026
#[test]
#[serial]
fn export_key_exit_code_nonexistent_path() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    // Use a path that doesn't exist
    let nonexistent_path = std::path::Path::new("/nonexistent/path/to/keys");

    // Export via command (should fail)
    let output = Command::new(get_monitor_binary_path())
        .arg("export-key")
        .arg("--path")
        .arg(nonexistent_path.to_string_lossy().as_ref())
        .output()
        .expect("Failed to execute vibetea-monitor binary");

    assert_eq!(
        output.status.code(),
        Some(EXIT_CONFIG_ERROR),
        "export-key should return exit code {} for non-existent path.\nstderr: {}",
        EXIT_CONFIG_ERROR,
        String::from_utf8_lossy(&output.stderr)
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

/// Verifies that export-key works correctly with paths containing spaces.
///
/// **Covers:** FR-003
#[test]
#[serial]
fn export_key_handles_path_with_spaces() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let path_with_spaces = temp_dir.path().join("path with spaces");
    std::fs::create_dir_all(&path_with_spaces).expect("Failed to create directory with spaces");

    // Generate and save key
    let crypto = Crypto::generate();
    crypto
        .save(&path_with_spaces)
        .expect("Failed to save keypair");
    let expected_seed = crypto.seed_base64();

    // Export via command
    let output = Command::new(get_monitor_binary_path())
        .arg("export-key")
        .arg("--path")
        .arg(path_with_spaces.to_string_lossy().as_ref())
        .output()
        .expect("Failed to execute vibetea-monitor binary");

    assert!(
        output.status.success(),
        "export-key should handle paths with spaces.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    assert_eq!(
        stdout.trim(),
        expected_seed,
        "Exported key should match for path with spaces"
    );
}

/// Verifies that the exported key can be directly piped to another command.
/// This test simulates the use case of piping to clipboard tools.
///
/// **Covers:** FR-003
#[test]
#[serial]
fn export_key_suitable_for_piping() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Generate and save key
    let crypto = Crypto::generate();
    crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");

    // Export via command
    let output = run_export_key_command(temp_dir.path());
    assert!(output.status.success(), "export-key should succeed");

    // The output should be suitable for direct use without processing
    // This means: no ANSI escape codes, no prompts, just clean data
    let stdout = output.stdout;

    // Check for ANSI escape codes
    assert!(
        !stdout.windows(2).any(|w| w == [0x1b, b'[']),
        "Output should not contain ANSI escape codes"
    );

    // Check for carriage returns (would mess up piping on Unix)
    #[cfg(unix)]
    assert!(
        !stdout.contains(&b'\r'),
        "Output should not contain carriage returns on Unix"
    );
}

/// Verifies that export-key reads from the correct file (key.priv).
///
/// **Covers:** FR-003
#[test]
#[serial]
fn export_key_reads_from_key_priv_file() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Manually create key.priv with known content
    let mut known_seed = [0u8; 32];
    for (i, byte) in known_seed.iter_mut().enumerate() {
        *byte = i as u8;
    }

    let key_priv_path = temp_dir.path().join("key.priv");
    std::fs::write(&key_priv_path, known_seed).expect("Failed to write key.priv");

    // Also create key.pub (required for consistency)
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&known_seed);
    let pubkey_base64 = BASE64_STANDARD.encode(signing_key.verifying_key().as_bytes());
    let key_pub_path = temp_dir.path().join("key.pub");
    std::fs::write(&key_pub_path, format!("{}\n", pubkey_base64)).expect("Failed to write key.pub");

    // Export via command
    let output = run_export_key_command(temp_dir.path());
    assert!(
        output.status.success(),
        "export-key should succeed.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid UTF-8");
    let exported_key = stdout.trim();

    // Verify the exported key matches what we wrote
    let expected_base64 = BASE64_STANDARD.encode(known_seed);
    assert_eq!(
        exported_key, expected_base64,
        "Exported key should match the content of key.priv"
    );
}
