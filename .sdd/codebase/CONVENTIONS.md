# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-04 (Phase 9 update)

## Code Style

### Formatting Tools

| Tool | Configuration | Command |
|------|---------------|---------|
| Prettier (TypeScript/Client) | `.prettierrc` | `npm run format` |
| ESLint (TypeScript/Client) | `eslint.config.js` | `npm run lint` |
| rustfmt (Rust/Server/Monitor) | Default settings | `cargo fmt` |
| clippy (Rust/Server/Monitor) | Default lints | `cargo clippy` |

### Style Rules

#### TypeScript/Client

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Enforced by Prettier |
| Quotes | Single quotes | `'string'` |
| Semicolons | Required | `const x = 1;` |
| Line length | No specific limit | Prettier handles wrapping |
| Trailing commas | ES5 style | Arrays/objects only (not function args) |
| JSX curly braces | Single-line only | `<Component />` or multi-line |

#### Rust/Server/Monitor

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 4 spaces | rustfmt default |
| Strings | Double quotes | `"string"` |
| Line length | 100 chars (soft) | rustfmt respects natural breaks |
| Comments | `//` for lines, `///` for docs | Doc comments on public items |
| Naming | snake_case for functions, PascalCase for types | `fn get_config()`, `struct Config` |

#### YAML (GitHub Actions)

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Enforced by GitHub Actions spec |
| Quotes | Single quotes for strings | `'value'` |
| Comments | `#` for explanations | Describe complex logic |
| Line length | 100 chars (soft) | Wrap long values with folded scalars |

## Naming Conventions

### Rust/Server/Monitor (Phase 9 Focus)

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `config.rs`, `error.rs`, `crypto.rs`, `setup_form.rs` |
| Test files | `*_test.rs` in `tests/` directory | `env_key_test.rs`, `privacy_test.rs` |
| Backup files | Original name + `.backup.` + timestamp | `key.priv.backup.20260204_143022` |
| Widget modules | snake_case | `setup_form.rs`, `event_stream.rs` |

#### Code Elements (Phase 9 Additions)

| Type | Convention | Example |
|------|------------|---------|
| Timestamp constants | SCREAMING_SNAKE_CASE | `BACKUP_TIMESTAMP_FORMAT = "%Y%m%d_%H%M%S"` |
| Backup methods | `backup_*` prefix | `backup_existing_keys()` |
| Conditional logic variables | `*_found` / `*_available` | `existing_keys_found` |
| UI state variants | PascalCase enum variants | `KeyOption::UseExisting`, `KeyOption::GenerateNew` |

## Common Patterns

### Key Backup Pattern (Phase 9)

The `Crypto` module implements atomic key backup with timestamp suffixes (FR-015):

```rust
// Backs up existing keys with timestamp suffix
pub fn backup_existing_keys(dir: &Path) -> Result<Option<String>, CryptoError> {
    let priv_path = dir.join(PRIVATE_KEY_FILE);
    let pub_path = dir.join(PUBLIC_KEY_FILE);

    // Idempotent: returns Ok(None) if no keys exist
    if !priv_path.exists() {
        return Ok(None);
    }

    // Generate timestamp: YYYYMMDD_HHMMSS (15 chars)
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

    // Backup private key: key.priv -> key.priv.backup.20260204_143022
    let priv_backup = dir.join(format!("{}.backup.{}", PRIVATE_KEY_FILE, timestamp));
    fs::rename(&priv_path, &priv_backup).map_err(|e| {
        CryptoError::BackupFailed(format!(
            "failed to backup private key to {:?}: {}",
            priv_backup, e
        ))
    })?;

    // Backup public key if it exists
    if pub_path.exists() {
        let pub_backup = dir.join(format!("{}.backup.{}", PUBLIC_KEY_FILE, timestamp));
        if let Err(e) = fs::rename(&pub_path, &pub_backup) {
            // Try to restore private key backup on public key backup failure
            let _ = fs::rename(&priv_backup, &priv_path);
            return Err(CryptoError::BackupFailed(format!(
                "failed to backup public key to {:?}: {}",
                pub_backup, e
            )));
        }
    }

    Ok(Some(timestamp))
}

// Generate keys with automatic backup
pub fn generate_with_backup(dir: &Path) -> Result<(Self, Option<String>), CryptoError> {
    let backup_timestamp = Self::backup_existing_keys(dir)?;
    let crypto = Self::generate();
    crypto.save(dir)?;
    Ok((crypto, backup_timestamp))
}
```

**Key conventions:**
- Timestamp format: `YYYYMMDD_HHMMSS` (15 characters, sortable lexicographically)
- Backup filename: `{original_name}.backup.{timestamp}`
- Idempotent: Returns `Ok(None)` if no keys to backup (not an error)
- Atomic restore on failure: Attempts to restore private key if public key backup fails
- Clear error messages: Includes paths in error context for debugging

**Tests verify:**
- Backup only occurs when keys exist (`test_backup_returns_none_when_no_keys_exist`)
- Timestamp format is correct (`test_timestamp_format_is_correct`)
- Files are renamed (not copied), preserving permissions
- Backup operation is idempotent

### Conditional UI Rendering Pattern (Phase 9)

The `SetupFormWidget` conditionally renders UI based on application state (FR-004):

```rust
// In monitor/src/tui/widgets/setup_form.rs

/// Renders the key option selector.
///
/// The display behavior depends on whether existing keys were found (FR-004):
/// - When `existing_keys_found` is `false`: Only shows "Generate new key" as the
///   sole option, since "Use existing" is not available.
/// - When `existing_keys_found` is `true`: Shows both options with radio button
///   indicators, allowing the user to toggle between them.
fn render_key_option_field(&self, buf: &mut Buffer, area: Rect) {
    let is_focused = self.state.focused_field == SetupField::KeyOption;

    // Label styling
    let label_style = if is_focused {
        self.theme.label.add_modifier(Modifier::BOLD)
    } else {
        self.theme.label
    };
    let label = Paragraph::new("Key Option:").style(label_style);

    // Style for the options
    let option_style = if is_focused {
        self.theme.input_focused
    } else {
        self.theme.input_unfocused
    };

    // Conditional rendering based on state
    let options_line = if self.state.existing_keys_found {
        // Both options available - show toggle with radio indicators
        let (existing_indicator, generate_indicator) = match self.state.key_option {
            KeyOption::UseExisting => (self.symbols.connected, self.symbols.disconnected),
            KeyOption::GenerateNew => (self.symbols.disconnected, self.symbols.connected),
        };

        let existing_text = format!("[{}] Use existing", existing_indicator);
        let generate_text = format!("[{}] Generate new", generate_indicator);

        // Highlight the selected option
        let (existing_style, generate_style) = match self.state.key_option {
            KeyOption::UseExisting => {
                let selected = if is_focused {
                    option_style.add_modifier(Modifier::BOLD)
                } else {
                    option_style
                };
                (selected, self.theme.text_muted)
            }
            KeyOption::GenerateNew => {
                let selected = if is_focused {
                    option_style.add_modifier(Modifier::BOLD)
                } else {
                    option_style
                };
                (self.theme.text_muted, selected)
            }
        };

        Line::from(vec![
            Span::styled(existing_text, existing_style),
            Span::raw("  "),
            Span::styled(generate_text, generate_style),
        ])
    } else {
        // Only "Generate new" available - show as the sole, pre-selected option
        let generate_style = if is_focused {
            option_style.add_modifier(Modifier::BOLD)
        } else {
            option_style
        };

        Line::from(Span::styled("Generate new key", generate_style))
    };

    let options = Paragraph::new(options_line);

    // Render with layout constraints
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);
    label.render(chunks[0], buf);
    options.render(chunks[1], buf);
}
```

**Key conventions:**
- **State-driven rendering**: All UI branches depend on `self.state.existing_keys_found`
- **Simplified UX when no option**: When only one option is available, don't show radio buttons - just show the option
- **Radio button indicators**: Use `symbols.connected` (●) for selected, `symbols.disconnected` (○) for unselected
- **Visual feedback**: Selected option is styled with focus color and bold when field is focused
- **Graceful degradation**: Works with both unicode and ASCII symbol sets

**Test coverage:**
- `setup_form_widget_no_keys_shows_only_generate_new` - Verifies single option display
- `setup_form_widget_with_keys_shows_both_options` - Verifies toggle display with both options
- `setup_form_widget_with_keys_shows_correct_selection_indicator` - Verifies radio button state

### Session Name Validation Pattern (FR-026)

```rust
/// Maximum allowed length for session names per FR-026.
const MAX_SESSION_NAME_LENGTH: usize = 64;

/// Validates a session name according to FR-026 requirements.
pub fn validate_session_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    // Check for empty name
    if trimmed.is_empty() {
        return Err("Session name cannot be empty".to_string());
    }

    // Check length constraint
    if trimmed.len() > MAX_SESSION_NAME_LENGTH {
        return Err("Session name must be 64 characters or less".to_string());
    }

    // Check character validity: only alphanumeric, hyphens, and underscores
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "Session name can only contain letters, numbers, hyphens, and underscores".to_string(),
        );
    }

    Ok(())
}
```

## Error Handling

### Error Patterns

The codebase uses the `thiserror` crate with phase-specific error types.

#### Phase 9 - Backup Errors

```rust
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("backup failed: {0}")]
    BackupFailed(String),

    // ... other variants
}
```

Error messages are descriptive and context-rich:
- Include file paths that failed
- Explain what operation was attempted
- Provide guidance on recovery (e.g., check permissions)

### Logging Conventions

The `tracing` crate is used for structured logging:

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Unrecoverable failures | `error!("Backup failed: {e}")` |
| warn | Recoverable issues | `warn!("Retry attempted")` |
| info | Important events | `info!("Keys backed up: {timestamp}")` |
| debug | Development details | `debug!("Timestamp format: {timestamp}")` |

## Module Organization

### Monitor Crate - Phase 9 Structure

```
monitor/src/
├── crypto.rs                    # Ed25519 operations (1179 lines, Phase 9)
│   ├── Crypto struct            # Main cryptographic operations
│   ├── CryptoError enum         # Error types including BackupFailed
│   ├── KeySource enum           # Tracks key origin (env var or file)
│   ├── backup_existing_keys()   # Phase 9: Timestamp-based backup
│   ├── generate_with_backup()   # Phase 9: Generate with auto-backup
│   └── tests module             # 30+ unit tests + backup tests
│
├── tui/
│   ├── app.rs                   # Application state
│   │   ├── SetupFormState       # Form state tracking
│   │   ├── KeyOption enum       # Phase 9: UseExisting | GenerateNew
│   │   ├── SetupField enum      # Focus tracking
│   │   └── existing_keys_found: bool  # Phase 9: UI state control
│   │
│   └── widgets/
│       └── setup_form.rs        # Phase 9: Conditional rendering
│           ├── SetupFormWidget  # Stateless widget
│           ├── validate_session_name()  # FR-026 validation
│           └── render_key_option_field()  # Phase 9: Conditional logic
```

## Common Patterns

### RAII Pattern for Test Cleanup (Phase 11+)

The `EnvGuard` pattern saves and restores environment variables:

```rust
struct EnvGuard {
    name: String,
    original: Option<String>,
}

impl EnvGuard {
    fn new(name: &str) -> Self {
        let original = env::var(name).ok();
        Self {
            name: name.to_string(),
            original,
        }
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
```

### Test Parallelism with serial_test (Phase 11+)

Tests modifying environment variables use `#[serial]`:

```rust
#[test]
#[serial]  // Prevents concurrent test execution
fn backup_test_with_env_var() {
    let guard = EnvGuard::new("VIBETEA_PRIVATE_KEY");
    // ... test code
}
```

## Git Conventions

### Commit Messages

Format: `type(scope): description`

| Type | Usage | Example |
|------|-------|---------|
| feat | New feature | `feat(crypto): add key backup functionality with timestamp suffix` |
| fix | Bug fix | `fix(setup): correct UI rendering when no existing keys found` |
| docs | Documentation updates | `docs(codebase): document Phase 9 patterns` |
| refactor | Code restructuring | `refactor(widgets): simplify conditional rendering logic` |
| test | Adding/updating tests | `test(crypto): add 10 backup pattern tests` |
| chore | Maintenance | `chore: update dependencies` |

## Code Review Guidelines

### Phase 9 Specific

When reviewing Phase 9 changes:

1. **Backup Pattern**:
   - Verify timestamp format is `YYYYMMDD_HHMMSS` (15 chars)
   - Confirm idempotency: no error if no keys to backup
   - Check error handling: restore private key if public backup fails
   - Validate permissions are preserved during rename

2. **Conditional Rendering**:
   - Verify single option doesn't show radio buttons
   - Confirm both options show when `existing_keys_found` is true
   - Check selected option is visually highlighted
   - Validate styling consistency with focused/unfocused states

3. **Testing**:
   - Ensure backup tests cover both success and failure paths
   - Verify UI tests check both state branches
   - Confirm tests use `#[serial]` for env var modifications

---

*This document defines HOW to write code. Last updated: Phase 9 (2026-02-04)*
