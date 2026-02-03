//! Cryptographic operations for VibeTea Monitor.
//!
//! This module handles Ed25519 keypair generation, storage, and event signing.
//! Keys are stored in the VibeTea directory (`~/.vibetea/` by default):
//!
//! - `key.priv`: Raw 32-byte Ed25519 seed (file mode 0600)
//! - `key.pub`: Base64-encoded public key (file mode 0644)
//!
//! # Example
//!
//! ```no_run
//! use vibetea_monitor::crypto::Crypto;
//! use std::path::Path;
//!
//! // Generate and save a new keypair
//! let crypto = Crypto::generate();
//! crypto.save(Path::new("/home/user/.vibetea")).unwrap();
//!
//! // Load an existing keypair
//! let crypto = Crypto::load(Path::new("/home/user/.vibetea")).unwrap();
//!
//! // Sign a message
//! let signature = crypto.sign(b"hello world");
//! println!("Signature (base64): {}", signature);
//! ```

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use base64::prelude::*;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::Rng;
use thiserror::Error;
use zeroize::Zeroize;

/// Indicates where the private key was loaded from.
///
/// Used for logging at startup (INFO level) to help users verify
/// which key source is active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    /// Key loaded from `VIBETEA_PRIVATE_KEY` environment variable.
    EnvironmentVariable,
    /// Key loaded from file at the given path.
    File(PathBuf),
}

/// Private key filename.
const PRIVATE_KEY_FILE: &str = "key.priv";

/// Public key filename.
const PUBLIC_KEY_FILE: &str = "key.pub";

/// Length of Ed25519 seed (private key material).
const SEED_LENGTH: usize = 32;

/// Environment variable name for the private key.
const ENV_PRIVATE_KEY: &str = "VIBETEA_PRIVATE_KEY";

/// Errors that can occur during cryptographic operations.
#[derive(Error, Debug)]
pub enum CryptoError {
    /// I/O error during key file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid key format or length.
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// Base64 decoding error.
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Key file already exists.
    #[error("key file already exists: {0}")]
    KeyExists(String),

    /// Environment variable not set or empty.
    #[error("environment variable not set: {0}")]
    EnvVar(String),
}

/// Handles Ed25519 cryptographic operations.
///
/// This struct manages an Ed25519 signing key and provides methods for
/// generating, loading, saving keys, and signing messages.
#[derive(Debug)]
pub struct Crypto {
    signing_key: SigningKey,
}

impl Crypto {
    /// Generates a new Ed25519 keypair using the operating system's
    /// cryptographically secure random number generator.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let crypto = Crypto::generate();
    /// let pubkey = crypto.public_key_base64();
    /// assert!(!pubkey.is_empty());
    /// ```
    #[must_use]
    pub fn generate() -> Self {
        // Generate 32 random bytes for the seed using the OS RNG
        let mut seed = [0u8; SEED_LENGTH];
        rand::rng().fill(&mut seed);
        let signing_key = SigningKey::from_bytes(&seed);
        // FR-020: Zero intermediate key material after signing key construction
        seed.zeroize();
        Self { signing_key }
    }

    /// Loads a keypair from the `VIBETEA_PRIVATE_KEY` environment variable.
    ///
    /// The environment variable should contain a base64-encoded 32-byte
    /// Ed25519 seed (RFC 4648 standard base64). Whitespace (including newlines)
    /// is trimmed before decoding.
    ///
    /// Returns a tuple of the `Crypto` instance and `KeySource::EnvironmentVariable`
    /// to indicate where the key was loaded from.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if:
    /// - The environment variable is not set or is empty
    /// - The value is not valid base64
    /// - The decoded key is not exactly 32 bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::crypto::{Crypto, KeySource};
    ///
    /// // Assuming VIBETEA_PRIVATE_KEY is set with a valid base64 seed
    /// let (crypto, source) = Crypto::load_from_env().unwrap();
    /// assert_eq!(source, KeySource::EnvironmentVariable);
    /// ```
    pub fn load_from_env() -> Result<(Self, KeySource), CryptoError> {
        let value = std::env::var(ENV_PRIVATE_KEY)
            .map_err(|_| CryptoError::EnvVar(ENV_PRIVATE_KEY.to_string()))?;

        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CryptoError::EnvVar(ENV_PRIVATE_KEY.to_string()));
        }

        let mut decoded = BASE64_STANDARD.decode(trimmed)?;
        let decoded_len = decoded.len();

        if decoded_len != SEED_LENGTH {
            // FR-020: Zero decoded buffer even on error path
            decoded.zeroize();
            return Err(CryptoError::InvalidKey(format!(
                "expected {} bytes, got {}",
                SEED_LENGTH, decoded_len
            )));
        }

        let mut seed: [u8; SEED_LENGTH] = decoded
            .try_into()
            .expect("length already validated");
        let signing_key = SigningKey::from_bytes(&seed);
        // FR-020: Zero intermediate key material after signing key construction
        seed.zeroize();

        Ok((Self { signing_key }, KeySource::EnvironmentVariable))
    }

    /// Loads a keypair, trying the environment variable first, then the file.
    ///
    /// This method implements the key precedence rule:
    /// 1. If `VIBETEA_PRIVATE_KEY` is set, it is used (env var takes precedence)
    /// 2. If the env var is set but invalid, an error is returned (no fallback)
    /// 3. If the env var is not set, the key is loaded from `{dir}/key.priv`
    ///
    /// Returns a tuple of the `Crypto` instance and `KeySource` indicating
    /// where the key was loaded from.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory containing the fallback key file
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if:
    /// - The env var is set but invalid (base64 or length error)
    /// - The env var is not set and the file doesn't exist or is invalid
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::crypto::{Crypto, KeySource};
    /// use std::path::Path;
    ///
    /// let (crypto, source) = Crypto::load_with_fallback(Path::new("/home/user/.vibetea")).unwrap();
    /// match source {
    ///     KeySource::EnvironmentVariable => println!("Loaded from env var"),
    ///     KeySource::File(path) => println!("Loaded from file: {:?}", path),
    /// }
    /// ```
    pub fn load_with_fallback(dir: &Path) -> Result<(Self, KeySource), CryptoError> {
        // Check if the environment variable is set
        match std::env::var(ENV_PRIVATE_KEY) {
            Ok(value) => {
                // Env var is set - try to load from it (no fallback on error)
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(CryptoError::EnvVar(ENV_PRIVATE_KEY.to_string()));
                }

                let mut decoded = BASE64_STANDARD.decode(trimmed)?;
                let decoded_len = decoded.len();

                if decoded_len != SEED_LENGTH {
                    // FR-020: Zero decoded buffer even on error path
                    decoded.zeroize();
                    return Err(CryptoError::InvalidKey(format!(
                        "expected {} bytes, got {}",
                        SEED_LENGTH, decoded_len
                    )));
                }

                let mut seed: [u8; SEED_LENGTH] = decoded
                    .try_into()
                    .expect("length already validated");
                let signing_key = SigningKey::from_bytes(&seed);
                // FR-020: Zero intermediate key material after signing key construction
                seed.zeroize();

                Ok((Self { signing_key }, KeySource::EnvironmentVariable))
            }
            Err(_) => {
                // Env var not set - load from file
                let priv_path = dir.join(PRIVATE_KEY_FILE);
                let crypto = Self::load(dir)?;
                Ok((crypto, KeySource::File(priv_path)))
            }
        }
    }

    /// Loads an existing keypair from a directory.
    ///
    /// Reads the private key from `{dir}/key.priv`. The file must contain
    /// exactly 32 bytes (the Ed25519 seed).
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory containing the key files
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if:
    /// - The key file doesn't exist or cannot be read
    /// - The key file doesn't contain exactly 32 bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::crypto::Crypto;
    /// use std::path::Path;
    ///
    /// let crypto = Crypto::load(Path::new("/home/user/.vibetea")).unwrap();
    /// ```
    pub fn load(dir: &Path) -> Result<Self, CryptoError> {
        let priv_path = dir.join(PRIVATE_KEY_FILE);

        let mut file = File::open(&priv_path)?;
        let mut seed = [0u8; SEED_LENGTH];
        let bytes_read = file.read(&mut seed)?;

        if bytes_read != SEED_LENGTH {
            // FR-020: Zero seed buffer even on error path
            seed.zeroize();
            return Err(CryptoError::InvalidKey(format!(
                "expected {} bytes, got {}",
                SEED_LENGTH, bytes_read
            )));
        }

        let signing_key = SigningKey::from_bytes(&seed);
        // FR-020: Zero intermediate key material after signing key construction
        seed.zeroize();
        Ok(Self { signing_key })
    }

    /// Saves the keypair to a directory.
    ///
    /// Creates two files:
    /// - `key.priv`: Raw 32-byte seed (mode 0600)
    /// - `key.pub`: Base64-encoded public key (mode 0644)
    ///
    /// The directory is created if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to save the key files
    ///
    /// # Errors
    ///
    /// Returns `CryptoError` if:
    /// - The directory cannot be created
    /// - The key files cannot be written
    /// - File permissions cannot be set (on Unix)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::crypto::Crypto;
    /// use std::path::Path;
    ///
    /// let crypto = Crypto::generate();
    /// crypto.save(Path::new("/home/user/.vibetea")).unwrap();
    /// ```
    pub fn save(&self, dir: &Path) -> Result<(), CryptoError> {
        // Create directory if it doesn't exist
        fs::create_dir_all(dir)?;

        // Save private key (raw bytes)
        let priv_path = dir.join(PRIVATE_KEY_FILE);
        let mut priv_file = File::create(&priv_path)?;
        priv_file.write_all(self.signing_key.to_bytes().as_slice())?;

        // Set private key permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&priv_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&priv_path, perms)?;
        }

        // Save public key (base64)
        let pub_path = dir.join(PUBLIC_KEY_FILE);
        let mut pub_file = File::create(&pub_path)?;
        pub_file.write_all(self.public_key_base64().as_bytes())?;
        pub_file.write_all(b"\n")?;

        // Set public key permissions to 0644 (owner read/write, others read)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&pub_path)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&pub_path, perms)?;
        }

        Ok(())
    }

    /// Checks if a keypair already exists in the given directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to check for key files
    ///
    /// # Returns
    ///
    /// `true` if the private key file exists, `false` otherwise.
    #[must_use]
    pub fn exists(dir: &Path) -> bool {
        dir.join(PRIVATE_KEY_FILE).exists()
    }

    /// Returns the public key as a base64-encoded string.
    ///
    /// This format is suitable for registration with the VibeTea server
    /// via the `VIBETEA_PUBLIC_KEYS` environment variable.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let crypto = Crypto::generate();
    /// let pubkey = crypto.public_key_base64();
    /// println!("Register this key: {}", pubkey);
    /// ```
    #[must_use]
    pub fn public_key_base64(&self) -> String {
        BASE64_STANDARD.encode(self.signing_key.verifying_key().as_bytes())
    }

    /// Returns the private key seed as a base64-encoded string.
    ///
    /// This is the inverse of `load_from_env()` - the returned string can be
    /// stored in the `VIBETEA_PRIVATE_KEY` environment variable.
    ///
    /// # Security
    ///
    /// The seed is sensitive key material. Handle the returned string with care
    /// and avoid logging it or storing it in insecure locations.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let crypto = Crypto::generate();
    /// let seed = crypto.seed_base64();
    /// // Can be used with: VIBETEA_PRIVATE_KEY=<seed>
    /// ```
    #[must_use]
    pub fn seed_base64(&self) -> String {
        BASE64_STANDARD.encode(self.signing_key.to_bytes())
    }

    /// Returns the first 8 characters of the base64-encoded public key.
    ///
    /// This fingerprint is used for key verification in logs without exposing
    /// the full key. Users can compare this with the server's registered key
    /// to verify they are using the correct keypair.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let crypto = Crypto::generate();
    /// let fingerprint = crypto.public_key_fingerprint();
    /// assert_eq!(fingerprint.len(), 8);
    /// println!("Key fingerprint: {}", fingerprint);
    /// ```
    #[must_use]
    pub fn public_key_fingerprint(&self) -> String {
        self.public_key_base64().chars().take(8).collect()
    }

    /// Returns the verifying (public) key.
    #[must_use]
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Signs a message and returns the signature as a base64-encoded string.
    ///
    /// The signature is created using the Ed25519 algorithm and can be
    /// verified by the server using the corresponding public key.
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes to sign
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::crypto::Crypto;
    ///
    /// let crypto = Crypto::generate();
    /// let signature = crypto.sign(b"event payload json");
    /// println!("X-Signature: {}", signature);
    /// ```
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> String {
        let signature: Signature = self.signing_key.sign(message);
        BASE64_STANDARD.encode(signature.to_bytes())
    }

    /// Signs a message and returns the raw signature bytes.
    ///
    /// Use this when you need the raw 64-byte signature instead of base64.
    ///
    /// # Arguments
    ///
    /// * `message` - The message bytes to sign
    #[must_use]
    pub fn sign_raw(&self, message: &[u8]) -> [u8; 64] {
        let signature: Signature = self.signing_key.sign(message);
        signature.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;
    use tempfile::TempDir;

    #[test]
    fn test_generate_creates_valid_keypair() {
        let crypto = Crypto::generate();
        let pubkey = crypto.public_key_base64();

        // Public key should be base64-encoded 32 bytes (44 chars with padding)
        assert!(!pubkey.is_empty());
        assert!(pubkey.len() >= 43); // Base64 of 32 bytes
    }

    #[test]
    fn test_public_key_fingerprint_is_8_chars() {
        let crypto = Crypto::generate();
        let fingerprint = crypto.public_key_fingerprint();

        // Fingerprint should be exactly 8 characters
        assert_eq!(fingerprint.len(), 8);

        // Should be prefix of full public key
        let pubkey = crypto.public_key_base64();
        assert!(pubkey.starts_with(&fingerprint));
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Generate and save
        let original = Crypto::generate();
        let original_pubkey = original.public_key_base64();
        original.save(dir_path).unwrap();

        // Load and verify
        let loaded = Crypto::load(dir_path).unwrap();
        let loaded_pubkey = loaded.public_key_base64();

        assert_eq!(original_pubkey, loaded_pubkey);
    }

    #[test]
    fn test_exists_returns_false_for_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!Crypto::exists(temp_dir.path()));
    }

    #[test]
    fn test_exists_returns_true_after_save() {
        let temp_dir = TempDir::new().unwrap();
        let crypto = Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        assert!(Crypto::exists(temp_dir.path()));
    }

    #[test]
    fn test_sign_produces_verifiable_signature() {
        let crypto = Crypto::generate();
        let message = b"test message for signing";

        let signature_b64 = crypto.sign(message);
        let signature_bytes = BASE64_STANDARD.decode(&signature_b64).unwrap();
        let signature = Signature::from_slice(&signature_bytes).unwrap();

        // Verify the signature using the public key
        let verifying_key = crypto.verifying_key();
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_sign_raw_produces_64_byte_signature() {
        let crypto = Crypto::generate();
        let message = b"test message";

        let signature = crypto.sign_raw(message);
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_different_messages_produce_different_signatures() {
        let crypto = Crypto::generate();
        let sig1 = crypto.sign(b"message one");
        let sig2 = crypto.sign(b"message two");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_same_message_produces_same_signature() {
        let crypto = Crypto::generate();
        let message = b"same message";

        // Note: Ed25519 is deterministic, so same message = same signature
        let sig1 = crypto.sign(message);
        let sig2 = crypto.sign(message);

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_load_from_nonexistent_dir_fails() {
        let result = Crypto::load(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_empty_file_fails() {
        let temp_dir = TempDir::new().unwrap();
        let priv_path = temp_dir.path().join(PRIVATE_KEY_FILE);

        // Create empty file
        File::create(&priv_path).unwrap();

        let result = Crypto::load(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidKey(_)));
    }

    #[test]
    fn test_load_from_short_file_fails() {
        let temp_dir = TempDir::new().unwrap();
        let priv_path = temp_dir.path().join(PRIVATE_KEY_FILE);

        // Create file with only 16 bytes (should be 32)
        let mut file = File::create(&priv_path).unwrap();
        file.write_all(&[0u8; 16]).unwrap();

        let result = Crypto::load(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidKey(_)));
    }

    #[cfg(unix)]
    #[test]
    fn test_save_sets_correct_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let crypto = Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // Check private key permissions (0600)
        let priv_path = temp_dir.path().join(PRIVATE_KEY_FILE);
        let priv_perms = fs::metadata(&priv_path).unwrap().permissions();
        assert_eq!(priv_perms.mode() & 0o777, 0o600);

        // Check public key permissions (0644)
        let pub_path = temp_dir.path().join(PUBLIC_KEY_FILE);
        let pub_perms = fs::metadata(&pub_path).unwrap().permissions();
        assert_eq!(pub_perms.mode() & 0o777, 0o644);
    }

    #[test]
    fn test_public_key_file_contains_base64() {
        let temp_dir = TempDir::new().unwrap();
        let crypto = Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // Read public key file
        let pub_path = temp_dir.path().join(PUBLIC_KEY_FILE);
        let contents = fs::read_to_string(pub_path).unwrap();
        let pubkey = contents.trim();

        // Should be valid base64 and decode to 32 bytes
        let decoded = BASE64_STANDARD.decode(pubkey).unwrap();
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_seed_base64_returns_valid_base64() {
        let crypto = Crypto::generate();
        let seed_b64 = crypto.seed_base64();

        // Should decode to exactly 32 bytes
        let decoded = BASE64_STANDARD.decode(&seed_b64).unwrap();
        assert_eq!(decoded.len(), SEED_LENGTH);
    }

    #[test]
    fn test_seed_base64_roundtrip() {
        // Generate a key and get its seed
        let original = Crypto::generate();
        let seed_b64 = original.seed_base64();
        let original_pubkey = original.public_key_base64();

        // Decode and recreate
        let decoded = BASE64_STANDARD.decode(&seed_b64).unwrap();
        let seed: [u8; SEED_LENGTH] = decoded.try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&seed);
        let recreated = Crypto { signing_key };

        // Should have the same public key
        assert_eq!(original_pubkey, recreated.public_key_base64());
    }

    mod env_tests {
        use super::*;
        use serial_test::serial;

        /// RAII guard for environment variable manipulation in tests.
        struct EnvGuard {
            key: String,
            original: Option<String>,
        }

        impl EnvGuard {
            fn new(key: &str) -> Self {
                let original = std::env::var(key).ok();
                Self {
                    key: key.to_string(),
                    original,
                }
            }

            fn set(&self, value: &str) {
                std::env::set_var(&self.key, value);
            }

            fn remove(&self) {
                std::env::remove_var(&self.key);
            }
        }

        impl Drop for EnvGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(val) => std::env::set_var(&self.key, val),
                    None => std::env::remove_var(&self.key),
                }
            }
        }

        #[test]
        #[serial]
        fn test_load_from_env_success() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);

            // Generate a key and export its seed
            let original = Crypto::generate();
            let seed_b64 = original.seed_base64();
            let original_pubkey = original.public_key_base64();

            // Set the env var and load
            guard.set(&seed_b64);
            let (loaded, source) = Crypto::load_from_env().unwrap();

            // Should have the same public key
            assert_eq!(original_pubkey, loaded.public_key_base64());
            assert_eq!(source, KeySource::EnvironmentVariable);
        }

        #[test]
        #[serial]
        fn test_load_from_env_trims_whitespace() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);

            let original = Crypto::generate();
            let seed_b64 = original.seed_base64();
            let original_pubkey = original.public_key_base64();

            // Add whitespace around the value
            let with_whitespace = format!("  \n{}\n  ", seed_b64);
            guard.set(&with_whitespace);

            let (loaded, _) = Crypto::load_from_env().unwrap();
            assert_eq!(original_pubkey, loaded.public_key_base64());
        }

        #[test]
        #[serial]
        fn test_load_from_env_missing_var() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);
            guard.remove();

            let result = Crypto::load_from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), CryptoError::EnvVar(_)));
        }

        #[test]
        #[serial]
        fn test_load_from_env_empty_var() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);
            guard.set("");

            let result = Crypto::load_from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), CryptoError::EnvVar(_)));
        }

        #[test]
        #[serial]
        fn test_load_from_env_whitespace_only() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);
            guard.set("   \n\t  ");

            let result = Crypto::load_from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), CryptoError::EnvVar(_)));
        }

        #[test]
        #[serial]
        fn test_load_from_env_invalid_base64() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);
            guard.set("not-valid-base64!!!");

            let result = Crypto::load_from_env();
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), CryptoError::Base64(_)));
        }

        #[test]
        #[serial]
        fn test_load_from_env_wrong_length() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);
            // 16 bytes instead of 32
            let short_seed = BASE64_STANDARD.encode([0u8; 16]);
            guard.set(&short_seed);

            let result = Crypto::load_from_env();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, CryptoError::InvalidKey(_)));
            if let CryptoError::InvalidKey(msg) = err {
                assert!(msg.contains("expected 32 bytes"));
                assert!(msg.contains("got 16"));
            }
        }

        #[test]
        #[serial]
        fn test_load_from_env_too_long() {
            let guard = EnvGuard::new(ENV_PRIVATE_KEY);
            // 64 bytes instead of 32
            let long_seed = BASE64_STANDARD.encode([0u8; 64]);
            guard.set(&long_seed);

            let result = Crypto::load_from_env();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, CryptoError::InvalidKey(_)));
            if let CryptoError::InvalidKey(msg) = err {
                assert!(msg.contains("expected 32 bytes"));
                assert!(msg.contains("got 64"));
            }
        }
    }
}
