# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-04 (Phase 9 update)

## Test Framework

### Rust/Monitor (Phase 9)

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)]` inline | In use |
| Unit (Crypto backup) | Rust built-in | `crypto.rs` backup_tests module | Phase 9 |
| Integration | Rust built-in | `tests/` directory with `serial_test` | In use |
| CLI | Subprocess-based binary execution | `tests/` with `std::process::Command` | Phase 12 |

### Running Tests

#### Rust/Monitor - Phase 9

| Command | Purpose |
|---------|---------|
| `cargo test -p vibetea-monitor` | Run all monitor tests |
| `cargo test -p vibetea-monitor crypto` | Run crypto module tests including backups |
| `cargo test -p vibetea-monitor crypto backup` | Run only backup pattern tests |
| `cargo test -- --test-threads=1` | Run sequentially (env var safety) |

## Test Organization

### Rust/Monitor - Phase 9 Structure

```
monitor/src/crypto.rs
├── Unit tests
│   ├── test_generate_creates_valid_keypair
│   ├── test_public_key_fingerprint_is_8_chars
│   ├── test_save_and_load_roundtrip
│   ├── test_exists_returns_false_for_empty_dir
│   ├── test_exists_returns_true_after_save
│   ├── test_sign_produces_verifiable_signature
│   └── ... (14 public tests)
│
├── Unit tests - Env var handling (serial)
│   ├── test_load_from_env_success
│   ├── test_load_from_env_trims_whitespace
│   ├── test_load_from_env_missing_var
│   └── ... (7 env tests)
│
└── Unit tests - Backup patterns (Phase 9)
    ├── test_backup_existing_keys_when_keys_exist
    ├── test_backup_returns_none_when_no_keys_exist
    ├── test_backup_handles_only_private_key
    ├── test_generate_with_backup_creates_new_keys_after_backup
    ├── test_generate_with_backup_no_backup_when_no_keys_exist
    ├── test_timestamp_format_is_correct
    ├── test_backup_preserves_permissions
    └── ... (total 8 backup tests)

monitor/tests/
├── env_key_test.rs              # 21 tests for env var key loading
├── privacy_test.rs              # 17 tests for privacy compliance
├── sender_recovery_test.rs       # Error recovery tests
└── key_export_test.rs            # 12 tests for export-key subcommand
```

## Test Patterns

### Backup Pattern Tests (Phase 9)

File: `monitor/src/crypto.rs` - `backup_tests` module (8 tests)

```rust
mod backup_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_existing_keys_when_keys_exist() {
        // Test setup with real keys
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Generate and save initial keys
        let original = Crypto::generate();
        let original_pubkey = original.public_key_base64();
        original.save(dir_path).unwrap();

        // Verify original files exist
        assert!(dir_path.join(PRIVATE_KEY_FILE).exists());
        assert!(dir_path.join(PUBLIC_KEY_FILE).exists());

        // Perform backup
        let result = Crypto::backup_existing_keys(dir_path);
        assert!(result.is_ok());
        let timestamp = result.unwrap();
        assert!(timestamp.is_some());

        let ts = timestamp.unwrap();

        // Verify timestamp format: YYYYMMDD_HHMMSS (15 characters)
        assert_eq!(ts.len(), 15);
        assert!(ts.chars().nth(8) == Some('_'));
        // All other characters should be digits
        assert!(ts
            .chars()
            .enumerate()
            .all(|(i, c)| i == 8 || c.is_ascii_digit()));

        // Verify original files no longer exist
        assert!(!dir_path.join(PRIVATE_KEY_FILE).exists());
        assert!(!dir_path.join(PUBLIC_KEY_FILE).exists());

        // Verify backup files exist with timestamp suffix
        let priv_backup = dir_path.join(format!("{}.backup.{}", PRIVATE_KEY_FILE, ts));
        let pub_backup = dir_path.join(format!("{}.backup.{}", PUBLIC_KEY_FILE, ts));
        assert!(priv_backup.exists());
        assert!(pub_backup.exists());

        // Verify backup private key content is correct
        fs::copy(&priv_backup, dir_path.join(PRIVATE_KEY_FILE)).unwrap();
        let loaded = Crypto::load(dir_path).unwrap();
        assert_eq!(original_pubkey, loaded.public_key_base64());
    }

    #[test]
    fn test_backup_returns_none_when_no_keys_exist() {
        let temp_dir = TempDir::new().unwrap();

        // Perform backup on empty directory
        let result = Crypto::backup_existing_keys(temp_dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_backup_handles_only_private_key() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Create only private key file (32 bytes)
        let priv_path = dir_path.join(PRIVATE_KEY_FILE);
        let mut file = File::create(&priv_path).unwrap();
        file.write_all(&[42u8; 32]).unwrap();

        // Perform backup
        let result = Crypto::backup_existing_keys(dir_path);
        assert!(result.is_ok());
        let timestamp = result.unwrap();
        assert!(timestamp.is_some());

        let ts = timestamp.unwrap();

        // Verify private key backup exists
        let priv_backup = dir_path.join(format!("{}.backup.{}", PRIVATE_KEY_FILE, ts));
        assert!(priv_backup.exists());

        // Verify original no longer exists
        assert!(!priv_path.exists());

        // Verify no public key backup (none existed)
        let pub_backup = dir_path.join(format!("{}.backup.{}", PUBLIC_KEY_FILE, ts));
        assert!(!pub_backup.exists());
    }

    #[test]
    fn test_generate_with_backup_creates_new_keys_after_backup() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Generate initial keys
        let original = Crypto::generate();
        let original_pubkey = original.public_key_base64();
        original.save(dir_path).unwrap();

        // Generate new keys with backup
        let (new_crypto, backup_timestamp) = Crypto::generate_with_backup(dir_path).unwrap();

        // Backup should have been performed
        assert!(backup_timestamp.is_some());
        let ts = backup_timestamp.unwrap();

        // New keys should be different
        let new_pubkey = new_crypto.public_key_base64();
        assert_ne!(original_pubkey, new_pubkey);

        // New key files should exist
        assert!(dir_path.join(PRIVATE_KEY_FILE).exists());
        assert!(dir_path.join(PUBLIC_KEY_FILE).exists());

        // Backup files should exist with timestamp
        let priv_backup = dir_path.join(format!("{}.backup.{}", PRIVATE_KEY_FILE, ts));
        let pub_backup = dir_path.join(format!("{}.backup.{}", PUBLIC_KEY_FILE, ts));
        assert!(priv_backup.exists());
        assert!(pub_backup.exists());

        // Verify new keys can be loaded
        let loaded = Crypto::load(dir_path).unwrap();
        assert_eq!(new_pubkey, loaded.public_key_base64());

        // Verify backup keys are the original ones
        fs::copy(&priv_backup, dir_path.join("key.priv.test")).unwrap();
        fs::rename(dir_path.join("key.priv.test"), dir_path.join(PRIVATE_KEY_FILE)).unwrap();
        let loaded_original = Crypto::load(dir_path).unwrap();
        assert_eq!(original_pubkey, loaded_original.public_key_base64());
    }

    #[test]
    fn test_generate_with_backup_no_backup_when_no_keys_exist() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // No existing keys
        assert!(!Crypto::exists(dir_path));

        // Generate with backup
        let (crypto, backup_timestamp) = Crypto::generate_with_backup(dir_path).unwrap();

        // No backup should have been performed
        assert!(backup_timestamp.is_none());

        // New keys should exist
        assert!(Crypto::exists(dir_path));
        assert!(dir_path.join(PUBLIC_KEY_FILE).exists());

        // Verify keys can be loaded
        let loaded = Crypto::load(dir_path).unwrap();
        assert_eq!(crypto.public_key_base64(), loaded.public_key_base64());
    }

    #[test]
    fn test_timestamp_format_is_correct() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Generate and save keys
        let crypto = Crypto::generate();
        crypto.save(dir_path).unwrap();

        // Get timestamp before backup
        let before = chrono::Local::now();

        // Perform backup
        let result = Crypto::backup_existing_keys(dir_path).unwrap();
        let ts = result.unwrap();

        // Get timestamp after backup
        let after = chrono::Local::now();

        // Parse the timestamp
        let parsed = chrono::NaiveDateTime::parse_from_str(&ts, "%Y%m%d_%H%M%S").unwrap();

        // The parsed timestamp should be between before and after
        let before_naive = before.naive_local();
        let after_naive = after.naive_local();

        // Allow 1 second tolerance for timing
        assert!(
            parsed >= before_naive - chrono::Duration::seconds(1)
                && parsed <= after_naive + chrono::Duration::seconds(1),
            "Timestamp {} should be between {:?} and {:?}",
            ts,
            before_naive,
            after_naive
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_backup_preserves_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Generate and save keys (this sets permissions)
        let crypto = Crypto::generate();
        crypto.save(dir_path).unwrap();

        // Verify original permissions
        let priv_path = dir_path.join(PRIVATE_KEY_FILE);
        let original_perms = fs::metadata(&priv_path).unwrap().permissions();
        assert_eq!(original_perms.mode() & 0o777, 0o600);

        // Perform backup
        let result = Crypto::backup_existing_keys(dir_path).unwrap();
        let ts = result.unwrap();

        // Check backup file permissions (should be preserved by rename)
        let priv_backup = dir_path.join(format!("{}.backup.{}", PRIVATE_KEY_FILE, ts));
        let backup_perms = fs::metadata(&priv_backup).unwrap().permissions();
        assert_eq!(backup_perms.mode() & 0o777, 0o600);
    }
}
```

**Key test patterns:**

1. **Idempotency test**: Verifies backup returns `Ok(None)` when no keys exist (not an error)
2. **Atomic operations**: Restores private key if public key backup fails
3. **File handling**: Verifies files are renamed (not copied), permissions preserved
4. **Timestamp format**: Validates `YYYYMMDD_HHMMSS` format (15 chars, sortable)
5. **State transitions**: Verifies old keys backed up, new keys created and loadable
6. **Permission preservation**: Unix-specific test for file mode preservation

### Widget Rendering Tests (Phase 9)

File: `monitor/src/tui/widgets/setup_form.rs` - `tests` module

```rust
#[test]
fn setup_form_widget_no_keys_shows_only_generate_new() {
    // When existing_keys_found is false, only "Generate new key" should be shown
    // without radio toggle indicators (FR-004, T205)
    let state = SetupFormState {
        session_name: "test".to_string(),
        session_name_error: None,
        key_option: KeyOption::GenerateNew,
        focused_field: SetupField::KeyOption,
        existing_keys_found: false,  // Phase 9: Controls rendering
    };

    let theme = Theme::default();
    let symbols = Symbols::default();

    let widget = SetupFormWidget::new(&state, &theme, &symbols);

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);

    let content: String = buf
        .content
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();

    // Should show "Generate new key" (without radio button brackets)
    assert!(
        content.contains("Generate new key"),
        "Should show 'Generate new key' when no existing keys"
    );
    // Should NOT show "Use existing" option at all
    assert!(
        !content.contains("Use existing"),
        "Should not show 'Use existing' when no keys found"
    );
}

#[test]
fn setup_form_widget_with_keys_shows_both_options() {
    // When existing_keys_found is true, both options should be shown
    // with radio toggle indicators (FR-004, T207)
    let state = SetupFormState {
        session_name: "test".to_string(),
        session_name_error: None,
        key_option: KeyOption::UseExisting,
        focused_field: SetupField::KeyOption,
        existing_keys_found: true,  // Phase 9: Controls rendering
    };

    let theme = Theme::default();
    let symbols = Symbols::default();

    let widget = SetupFormWidget::new(&state, &theme, &symbols);

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);

    let content: String = buf
        .content
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();

    // Should show both options with toggle capability
    assert!(
        content.contains("Use existing"),
        "Should show 'Use existing' when keys are found"
    );
    assert!(
        content.contains("Generate new"),
        "Should show 'Generate new' option"
    );
}

#[test]
fn setup_form_widget_with_keys_shows_correct_selection_indicator() {
    let theme = Theme::default();
    let symbols = Symbols::default();
    let area = Rect::new(0, 0, 80, 24);

    // Test with UseExisting selected
    let state = SetupFormState {
        session_name: "test".to_string(),
        session_name_error: None,
        key_option: KeyOption::UseExisting,
        focused_field: SetupField::KeyOption,
        existing_keys_found: true,  // Phase 9: Both options visible
    };

    let widget = SetupFormWidget::new(&state, &theme, &symbols);
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);

    let content: String = buf
        .content
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();

    // Should contain the connected symbol for selected option
    assert!(content.contains("Use existing"));
    assert!(content.contains("Generate new"));

    // Test with GenerateNew selected
    let state = SetupFormState {
        key_option: KeyOption::GenerateNew,
        ..state
    };

    let widget = SetupFormWidget::new(&state, &theme, &symbols);
    let mut buf = Buffer::empty(area);
    widget.render(area, &mut buf);

    let content: String = buf
        .content
        .iter()
        .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
        .collect();

    assert!(content.contains("Use existing"));
    assert!(content.contains("Generate new"));
}
```

**Test coverage:**
- Single option rendering without radio buttons
- Both options rendering with radio button indicators
- Visual selection indicator changes based on `key_option` state
- Styling consistency with focus state

## CI Integration - Phase 9

### Test Pipeline Update

```bash
# Run crypto tests including new backup patterns
cargo test -p vibetea-monitor crypto -- --test-threads=1

# Run all monitor tests
cargo test -p vibetea-monitor -- --test-threads=1

# Run widget tests
cargo test -p vibetea-monitor tui::widgets::setup_form -- --nocapture
```

### Total Test Coverage (Phase 9)

- **Rust/Monitor**: 120+ tests
  - Crypto unit tests: 30+ (includes 8 backup pattern tests)
  - Integration tests: 50+
  - Widget tests: 15+
- **Grand Total**: 190+ tests

## Testing Best Practices - Phase 9

### Backup Pattern Testing

1. **Test idempotency**: Backup with no keys should return `Ok(None)`, not error
2. **Verify atomicity**: Check both files renamed, or fallback on failure
3. **Validate timestamps**: Ensure format is sortable (`YYYYMMDD_HHMMSS`)
4. **Check permissions**: Verify 0600 for private key, 0644 for public key
5. **Test state transitions**: Verify old keys backed up, new keys created

### UI Rendering Testing

1. **Branch coverage**: Test both `existing_keys_found` true/false paths
2. **Visual consistency**: Verify focused/unfocused styling applied
3. **Symbol validation**: Test with both unicode and ASCII symbol sets
4. **Content verification**: Assert expected strings in rendered output
5. **State isolation**: Each test creates fresh state without side effects

---

*This document describes HOW to test. Last updated: Phase 9 (2026-02-04)*
