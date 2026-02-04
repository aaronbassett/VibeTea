# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-04

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

## Naming Conventions

### TypeScript/Client

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | `ConnectionStatus.tsx`, `TokenForm.tsx`, `EventStream.tsx`, `Heatmap.tsx`, `SessionOverview.tsx` |
| Hooks | camelCase with `use` prefix | `useEventStore.ts`, `useWebSocket.ts`, `useSessionTimeouts.ts` |
| Types | PascalCase in `types/` folder | `types/events.ts` contains `VibeteaEvent` |
| Utilities | camelCase | `utils/formatting.ts` |
| Constants | SCREAMING_SNAKE_CASE in const files | `MAX_EVENTS = 1000`, `TOKEN_STORAGE_KEY` |
| Test files | Same as source + `.test.ts` | `__tests__/events.test.ts`, `__tests__/formatting.test.ts` |
| Test directories | `__tests__/` at feature level | Co-located with related source |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Variables | camelCase | `sessionId`, `eventCount`, `wsRef`, `connectRef`, `displayEvents`, `isAutoScrollEnabled`, `recentEventCount`, `viewDays` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_EVENTS`, `DEFAULT_BUFFER_SIZE`, `TOKEN_STORAGE_KEY`, `ESTIMATED_ROW_HEIGHT`, `RECENT_EVENT_WINDOW_MS`, `LOW_ACTIVITY_THRESHOLD` |
| Functions | camelCase, verb prefix | `selectEventsBySession()`, `isSessionEvent()`, `parseEventMessage()`, `calculateBackoff()`, `formatTimestamp()`, `getEventDescription()`, `countRecentEventsBySession()`, `getActivityLevel()` |
| Classes | PascalCase (rare in modern React) | N/A |
| Interfaces | PascalCase, no `I` prefix | `EventStore`, `Session`, `VibeteaEvent`, `UseWebSocketReturn`, `ConnectionStatusProps`, `EventStreamProps`, `SessionOverviewProps`, `ActivityIndicatorProps` |
| Types | PascalCase | `VibeteaEvent<T>`, `EventPayload`, `ConnectionStatus`, `TokenStatus`, `EventType`, `ActivityLevel`, `SessionStatus` |
| Type guards | `is` prefix | `isSessionEvent()`, `isValidEventType()` |
| Enums | PascalCase | N/A (use union types instead) |
| Refs | camelCase with `Ref` suffix | `wsRef`, `reconnectTimeoutRef`, `connectRef`, `parentRef`, `previousEventCountRef` |
| Records/Maps | PascalCase for type, camelCase for variable | `EVENT_TYPE_ICONS`, `EVENT_TYPE_COLORS`, `STATUS_CONFIG`, `PULSE_ANIMATIONS` (const) |

### Rust/Server/Monitor

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `config.rs`, `error.rs`, `types.rs`, `watcher.rs`, `parser.rs`, `privacy.rs`, `crypto.rs`, `sender.rs`, `main.rs` |
| Test files | `*_test.rs` in `tests/` directory | `env_key_test.rs`, `privacy_test.rs`, `sender_recovery_test.rs`, `key_export_test.rs` |
| Types | PascalCase | `Config`, `Event`, `ServerError`, `MonitorError`, `PrivacyConfig`, `Crypto`, `Sender`, `Command` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT`, `DEFAULT_BUFFER_SIZE`, `SENSITIVE_TOOLS`, `PRIVATE_KEY_FILE`, `SHUTDOWN_TIMEOUT_SECS` |
| Test modules | `#[cfg(test)] mod tests` | In same file as implementation |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `from_env()`, `generate_event_id()`, `parse_jsonl_line()`, `extract_basename()`, `parse_args()`, `run_export_key()` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT = 8080`, `SEED_LENGTH = 32`, `MAX_RETRY_DELAY_SECS = 60`, `ENV_VAR_NAME = "VIBETEA_PRIVATE_KEY"` |
| Structs | PascalCase | `Config`, `Event`, `PrivacyPipeline`, `Crypto`, `Sender`, `Command` |
| Enums | PascalCase | `EventType`, `SessionAction`, `ServerError`, `CryptoError`, `SenderError`, `Command` |
| Methods | snake_case | `.new()`, `.to_string()`, `.from_env()`, `.process()`, `.generate()`, `.load()`, `.save()`, `.sign()`, `.seed_base64()`, `.public_key_fingerprint()` |
| Lifetimes | Single lowercase letter | `'a`, `'static` |

## Error Handling

### Error Patterns

#### TypeScript

Client error handling uses:
- Try/catch for async operations
- Type guards for runtime validation (e.g., `isValidEventType()`)
- Discriminated unions for safe event handling (see Common Patterns section)
- Null checks with explicit error paths (e.g., `parseEventMessage()` returns `null` on invalid input)
- Console logging for error visibility (e.g., `console.error()`, `console.warn()`)

#### Rust/Server

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `server/src/config.rs` defines `ConfigError` |
| Authentication errors | String-based variant in `ServerError` | `ServerError::Auth(String)` |
| Validation errors | String-based variant in `ServerError` | `ServerError::Validation(String)` |
| Rate limiting | Struct variant with fields | `ServerError::RateLimit { source, retry_after }` |
| WebSocket errors | String-based variant in `ServerError` | `ServerError::WebSocket(String)` |
| Internal errors | String-based variant in `ServerError` | `ServerError::Internal(String)` |

#### Rust/Monitor

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `monitor/src/config.rs` defines `ConfigError` |
| I/O errors | Use `#[from]` for automatic conversion | `MonitorError::Io(#[from] std::io::Error)` |
| JSON errors | Automatic conversion via `serde_json` | `MonitorError::Json(#[from] serde_json::Error)` |
| HTTP errors | String-based variants | `MonitorError::Http(String)` |
| Cryptographic errors | Custom enum with variants | `CryptoError::InvalidKey`, `CryptoError::KeyExists`, `CryptoError::EnvVar` |
| Sender errors | Enum with specific variants | `SenderError::AuthFailed`, `SenderError::RateLimited`, `SenderError::MaxRetriesExceeded` |
| File watching errors | String-based variants | `MonitorError::Watch(String)` |
| JSONL parsing errors | String-based variants | `MonitorError::Parse(String)` |

### Error Response Format

#### TypeScript (Standard for client responses)

```typescript
{
  error: {
    code: 'ERROR_CODE',
    message: 'Human readable message',
    details?: object
  }
}
```

#### Rust Error Messages

Errors use `thiserror::Error` with `#[error]` attributes for automatic `Display` impl:

**Server Example** (`server/src/error.rs`):

```rust
#[derive(Debug)]
pub enum ServerError {
    Config(ConfigError),
    Auth(String),
    Validation(String),
    RateLimit { source: String, retry_after: u64 },
    WebSocket(String),
    Internal(String),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "configuration error: {err}"),
            Self::Auth(msg) => write!(f, "authentication failed: {msg}"),
            // ... other variants
        }
    }
}
```

**Monitor Example** (`monitor/src/error.rs`):

```rust
#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Crypto Example** (`monitor/src/crypto.rs`):

```rust
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("key file already exists: {0}")]
    KeyExists(String),

    #[error("environment variable not set: {0}")]
    EnvVar(String),
}
```

**Sender Example** (`monitor/src/sender.rs`):

```rust
#[derive(Error, Debug)]
pub enum SenderError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("authentication failed: invalid signature or source ID")]
    AuthFailed,

    #[error("rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },
}
```

### Logging Conventions

The `tracing` crate is used for structured logging across async Rust code:

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Unrecoverable failures | `error!(unflushed_events = unflushed, "Some events could not be sent")` |
| warn | Recoverable issues | `warn!("Retry attempted")` |
| info | Important events | `info!("Starting VibeTea Monitor")` |
| debug | Development details | `debug!("Configuration loaded")` |

**Note**: Logging is initialized using `tracing_subscriber` with `EnvFilter` to control verbosity via `RUST_LOG` environment variable.

In TypeScript, use console methods with contextual prefixes:

```typescript
console.warn('[useWebSocket] No authentication token found in localStorage');
console.error('[useWebSocket] Connection error:', event);
console.error('[useWebSocket] Failed to create WebSocket:', error);
```

## Common Patterns

### CLI Subcommand Pattern (Phase 12)

The monitor binary uses clap for CLI commands with structured parsing:

```rust
// monitor/src/main.rs - Command enum with subcommands
#[derive(Subcommand, Debug)]
enum Command {
    /// Generate Ed25519 keypair for server authentication.
    Init {
        /// Force overwrite existing keys without confirmation.
        #[arg(short, long)]
        force: bool,
    },

    /// Export private key for GitHub Actions.
    ///
    /// Outputs the base64-encoded private key seed to stdout.
    /// Use this to set the VIBETEA_PRIVATE_KEY secret in GitHub Actions.
    ExportKey {
        /// Directory containing keypair.
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Start the monitor daemon.
    Run,
}

impl Cli {
    fn run() -> Result<()> {
        match cli.command {
            Command::Init { force } => run_init(force),
            Command::ExportKey { path } => run_export_key(path),
            Command::Run => run_monitor(),
        }
    }
}
```

Key conventions:
- **Documentation**: Each subcommand has a doc comment describing its purpose
- **Arguments**: Structured using clap attributes (`#[arg]`)
- **Error handling**: Returns `Result<()>` with context-rich errors
- **Stdout vs stderr**: Diagnostics go to stderr, output data to stdout (e.g., keys)

### GitHub Actions Environment Pattern (Phase 5)

When deploying monitor in CI/CD, use standard environment variable conventions:

```yaml
# .github/workflows/ci-with-monitor.yml
env:
  VIBETEA_PRIVATE_KEY: ${{ secrets.VIBETEA_PRIVATE_KEY }}
  VIBETEA_SERVER_URL: ${{ secrets.VIBETEA_SERVER_URL }}
  VIBETEA_SOURCE_ID: "github-${{ github.repository }}-${{ github.run_id }}"
```

Key conventions:
- **Secrets**: Private key and server URL stored as GitHub repository secrets
- **Source ID**: Includes repository and run ID for traceability (format: `github-owner/repo-run-id`)
- **Background execution**: Monitor runs in background with `./vibetea-monitor run &`
- **Graceful shutdown**: Uses `kill -TERM` for non-blocking flush of buffered events
- **Error resilience**: Monitor failure doesn't block workflow (wrapped in conditional checks)

Workflow structure (from `.github/workflows/ci-with-monitor.yml`):

```yaml
steps:
  # 1. Download or build monitor binary
  - name: Download VibeTea Monitor
    run: curl -fsSL -o vibetea-monitor "https://github.com/aaronbassett/VibeTea/releases/latest/..."

  # 2. Start monitor in background
  - name: Start VibeTea Monitor
    run: |
      if [ -f vibetea-monitor ] && [ -n "$VIBETEA_PRIVATE_KEY" ]; then
        ./vibetea-monitor run &
        MONITOR_PID=$!
        echo "MONITOR_PID=$MONITOR_PID" >> $GITHUB_ENV
      fi

  # 3. Run CI steps (events captured during this time)
  - name: Run tests
    run: cargo test --workspace -- --test-threads=1

  # 4. Graceful shutdown
  - name: Stop VibeTea Monitor
    if: always()
    run: |
      if [ -n "$MONITOR_PID" ]; then
        kill -TERM $MONITOR_PID 2>/dev/null || true
        sleep 2
      fi
```

### Event-Driven Architecture

The codebase uses discriminated unions for type-safe event handling:

#### TypeScript

```typescript
// Type-safe event handling in TypeScript
type VibeteaEvent<T extends EventType = EventType> = {
  id: string;
  source: string;
  timestamp: string;
  type: T;
  payload: EventPayloadMap[T]; // Auto-typed based on T
};

// Type guards for runtime checks
function isSessionEvent(event: VibeteaEvent): event is VibeteaEvent<'session'> {
  return event.type === 'session';
}

// Validation type guard
function isValidEventType(value: unknown): value is EventType {
  return (
    typeof value === 'string' &&
    (VALID_EVENT_TYPES as readonly string[]).indexOf(value) !== -1
  );
}
```

#### Rust

```rust
// Rust equivalent with untagged enums
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    Tool { session_id: Uuid, tool: String, status: ToolStatus, ... },
    Session { session_id: Uuid, action: SessionAction, project: String },
    Activity { session_id: Uuid, project: Option<String> },
    // ... other variants, ordered from most specific to least specific
}

// Tagged wrapper for the full event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub payload: EventPayload,
}
```

### RAII Pattern for Test Cleanup (Phase 11)

The `EnvGuard` pattern saves and restores environment variables automatically:

```rust
// monitor/tests/env_key_test.rs
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

    fn set(&self, value: &str) {
        env::set_var(&self.name, value);
    }

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

// Usage in tests
#[test]
#[serial]
fn test_env_var_handling() {
    let guard = EnvGuard::new("VIBETEA_PRIVATE_KEY");
    guard.set("test_value");
    // Test runs with modified env var
    // EnvGuard drops and restores original value
}
```

Key benefits:
1. **Automatic restoration**: Environment variables are restored even if test panics
2. **No manual cleanup required**: Drop trait handles cleanup
3. **Safe for nested guards**: Multiple EnvGuards can be created safely
4. **Thread-safe when combined with #[serial]**: Prevents test interference

### Test Parallelism with serial_test (Phase 11)

Tests modifying environment variables must use `#[serial]` from the `serial_test` crate:

```rust
#[test]
#[serial]  // Prevents concurrent test execution
fn load_valid_base64_key_from_env() {
    let guard = EnvGuard::new("VIBETEA_PRIVATE_KEY");
    // ... test code
}
```

The CI enforces this with `--test-threads=1`:

```bash
cargo test --package vibetea-monitor -- --test-threads=1
```

**Why this matters**:
- Environment variables are process-wide state
- Concurrent tests can interfere with each other
- `#[serial]` ensures tests run sequentially
- The macro also works with `#[tokio::test]` for async tests

From `.github/workflows/ci.yml`:

```yaml
- name: Run tests
  run: cargo test --package ${{ matrix.crate }} -- --test-threads=1
```

### Test Documentation Pattern (Phase 11)

Tests document the requirement they verify:

```rust
/// Verifies that a valid base64-encoded 32-byte seed can be loaded from
/// the `VIBETEA_PRIVATE_KEY` environment variable.
///
/// FR-001: Load Ed25519 private key seed from `VIBETEA_PRIVATE_KEY` env var
/// as base64-encoded string.
#[test]
#[serial]
fn load_valid_base64_key_from_env() {
    // ...
}
```

Pattern: Each test documents its feature requirement (FR-###) from the spec.

### Test Organization Pattern (Phase 11)

Integration tests are organized in `tests/` directory with meaningful names:

```
monitor/tests/
├── env_key_test.rs       # 21 tests for env var key loading (FR-001, FR-002, FR-004, etc.)
├── privacy_test.rs       # Tests for privacy compliance
├── sender_recovery_test.rs # Tests for error recovery
└── key_export_test.rs    # 12 tests for export-key subcommand (Phase 12)
```

Each test file is a complete integration test that can run independently:

```rust
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
```

### Test Helper Pattern (Phase 11)

Helper functions organize common test setup:

```rust
// Test Helpers section at top of test file
const ENV_VAR_NAME: &str = "VIBETEA_PRIVATE_KEY";

/// Generates a valid 32-byte seed and returns it base64-encoded.
fn generate_valid_base64_seed() -> (String, [u8; 32]) {
    let mut seed = [0u8; 32];
    use rand::Rng;
    rand::rng().fill(&mut seed);
    let base64_seed = BASE64_STANDARD.encode(&seed);
    (base64_seed, seed)
}

/// Environment variable name for the private key.
#[test]
#[serial]
fn load_valid_base64_key_from_env() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    let (base64_seed, _seed) = generate_valid_base64_seed();
    // ...
}
```

### Round-Trip Testing Pattern (Phase 11)

Crypto tests use round-trip patterns to verify full workflows:

```rust
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

    // Step 2: Export the seed as base64
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

    // Step 7: Verify the signature
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
```

### CLI Testing Pattern (Phase 12)

Integration tests for CLI commands use subprocess execution:

```rust
//! Integration tests for the `export-key` subcommand.
//!
//! These tests verify the following requirements:
//! - FR-003: Monitor MUST provide `export-key` subcommand to output ONLY
//!   the base64-encoded private key followed by a single newline
//! - FR-023: All diagnostic and error messages from `export-key` MUST go to stderr;
//!   only the key itself goes to stdout
//! - FR-026: Exit codes: 0 for success, 1 for configuration error, 2 for runtime error
//! - FR-027/FR-028: Round-trip verification (generate -> export -> load -> sign -> verify)

use std::process::Command;

/// Runs the vibetea-monitor export-key command with the given path.
fn run_export_key_command(key_path: &std::path::Path) -> std::process::Output {
    Command::new(get_monitor_binary_path())
        .arg("export-key")
        .arg("--path")
        .arg(key_path.to_string_lossy().as_ref())
        .output()
        .expect("Failed to execute vibetea-monitor binary")
}

/// Verifies the complete round-trip using the export-key command.
#[test]
#[serial]
fn roundtrip_generate_export_command_import_sign_verify() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Step 1 & 2: Generate and save keypair
    let original_crypto = Crypto::generate();
    original_crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");
    let original_pubkey = original_crypto.public_key_base64();

    // Step 3: Export via export-key command
    let output = run_export_key_command(temp_dir.path());

    // Step 4: Command should succeed with exit code 0
    assert!(
        output.status.success(),
        "export-key should exit with code 0, got: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 5: Get the exported key from stdout and set as env var
    let exported_key =
        String::from_utf8(output.stdout.clone()).expect("stdout should be valid UTF-8");
    let exported_key_trimmed = exported_key.trim();

    // Verify the exported key matches the original seed
    let original_seed = original_crypto.seed_base64();
    assert_eq!(
        exported_key_trimmed, original_seed,
        "Exported key should match the original seed"
    );

    // ... continue with env var loading and signature verification
}
```

Key conventions in CLI testing:
- **Subprocess execution**: Tests spawn the actual binary using `Command::new()`
- **Exit code validation**: Tests verify expected exit codes (0 success, 1 config error, 2 runtime error)
- **Output stream separation**: Verify output goes to correct stream (stdout for data, stderr for diagnostics)
- **Format validation**: Tests verify output format exactly (e.g., base64 key + single newline)
- **Base64 validation**: Exported keys are verified to decode to exactly 32 bytes

### Error Message Testing Pattern (Phase 11)

Tests verify error messages are clear and actionable:

```rust
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
```

Pattern: Each test verifies both the error AND the clarity of the error message.

### Zustand Store Pattern (TypeScript)

Client state management uses Zustand with selector functions:

```typescript
export const useEventStore = create<EventStore>()((set) => ({
  // State
  status: 'disconnected',
  events: [],
  sessions: new Map(),

  // Actions (immutable updates)
  addEvent: (event: VibeteaEvent) => {
    set((state) => {
      // Calculate new state from current state
      const newEvents = [event, ...state.events].slice(0, MAX_EVENTS);
      // Return partial updates
      return { events: newEvents };
    });
  },
}));

// Selector utilities to extract derived state
export function selectActiveSessions(state: EventStore): Session[] {
  return Array.from(state.sessions.values()).filter(s => s.status !== 'ended');
}
```

### Configuration Pattern (Rust)

Config loads from environment variables with sensible defaults:

**Server Example** (`server/src/config.rs`):

```rust
pub struct Config {
    pub public_keys: HashMap<String, String>,
    pub subscriber_token: Option<String>,
    pub port: u16,
    pub unsafe_no_auth: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port = parse_port()?;
        let public_keys = parse_public_keys()?;
        let subscriber_token = env::var("VIBETEA_SUBSCRIBER_TOKEN").ok();
        let unsafe_no_auth = parse_bool_env("VIBETEA_UNSAFE_NO_AUTH");

        let config = Self { public_keys, subscriber_token, port, unsafe_no_auth };
        config.validate()?;
        Ok(config)
    }
}
```

**Monitor Example** (`monitor/src/config.rs`):

```rust
pub struct Config {
    pub server_url: String,
    pub source_id: String,
    pub key_path: PathBuf,
    pub claude_dir: PathBuf,
    pub buffer_size: usize,
    pub basename_allowlist: Option<Vec<String>>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server_url = env::var("VIBETEA_SERVER_URL")
            .map_err(|_| ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".into()))?;
        // ... parse other vars with defaults
    }
}
```

### Privacy Pipeline Pattern (Rust)

The privacy module (`monitor/src/privacy.rs`) implements a privacy-by-design approach using composable pipeline components:

```rust
// Configuration object controlling privacy behavior
pub struct PrivacyConfig {
    basename_allowlist: Option<HashSet<String>>,
}

impl PrivacyConfig {
    pub fn from_env() -> Self {
        // Reads VIBETEA_BASENAME_ALLOWLIST environment variable
        // Format: ".rs,.ts,.md" (comma-separated extensions)
    }

    pub fn is_extension_allowed(&self, basename: &str) -> bool {
        // Returns true if extension is in allowlist or no allowlist is set
    }
}

// Pipeline struct encapsulating privacy transformations
pub struct PrivacyPipeline {
    config: PrivacyConfig,
}

impl PrivacyPipeline {
    pub fn process(&self, payload: EventPayload) -> EventPayload {
        // Applies privacy transformations:
        // 1. Strips context from sensitive tools (Bash, Grep, Glob, WebSearch, WebFetch)
        // 2. Extracts basenames from file paths for safe tools (Read, Write, Edit)
        // 3. Applies allowlist filtering based on file extensions
        // 4. Neutralizes summary text to "Session ended"
        // 5. Passes through Session, Activity, Agent, Error payloads unchanged
    }
}

// Utility function for basename extraction
pub fn extract_basename(path: &str) -> Option<String> {
    // Safely extracts filename from any path format
    // Returns None for invalid paths (empty, root, trailing separators)
}
```

Key conventions in privacy module:
- **Immutable operations**: Privacy pipeline creates new payloads rather than modifying in-place
- **Graceful degradation**: Invalid paths return `None` rather than panicking
- **Configuration flexibility**: Uses environment variables for runtime control
- **Comprehensive documentation**: Every public item has detailed doc comments with examples
- **Privacy-first defaults**: Default config allows all extensions (no data loss), allowlist can be set to restrict

### Cryptographic Operations Pattern (Rust)

The crypto module (`monitor/src/crypto.rs`) handles Ed25519 keypair generation, storage, and event signing:

```rust
// Handles Ed25519 cryptographic operations
pub struct Crypto {
    signing_key: SigningKey,
}

impl Crypto {
    // Generates a new Ed25519 keypair using OS RNG
    pub fn generate() -> Self { ... }

    // Loads an existing keypair from directory
    pub fn load(dir: &Path) -> Result<Self, CryptoError> { ... }

    // Loads from env var with fallback to file
    pub fn load_with_fallback(dir: &Path) -> Result<(Crypto, KeySource), CryptoError> { ... }

    // Loads from VIBETEA_PRIVATE_KEY env var
    pub fn load_from_env() -> Result<(Crypto, KeySource), CryptoError> { ... }

    // Saves keypair with secure file permissions (0600 for private key)
    pub fn save(&self, dir: &Path) -> Result<(), CryptoError> { ... }

    // Checks if keypair already exists
    pub fn exists(dir: &Path) -> bool { ... }

    // Signs a message and returns base64-encoded signature
    pub fn sign(&self, message: &[u8]) -> String { ... }

    // Signs and returns raw 64-byte signature
    pub fn sign_raw(&self, message: &[u8]) -> [u8; 64] { ... }

    // Export seed as base64 (used by export-key subcommand)
    pub fn seed_base64(&self) -> String { ... }

    // Get public key as base64
    pub fn public_key_base64(&self) -> String { ... }

    // Get fingerprint of public key (short identifier for logging)
    pub fn public_key_fingerprint(&self) -> String { ... }

    // Get verifying key for signature verification
    pub fn verifying_key(&self) -> VerifyingKey { ... }
}

// Indicates where the private key was loaded from
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    EnvironmentVariable,
    File(PathBuf),
}
```

Key conventions in crypto module:
- **Key storage**: Private key stored as raw 32-byte seed in `key.priv`, public key as base64 in `key.pub`
- **File permissions**: Unix permissions set to 0600 (private key) and 0644 (public key)
- **Deterministic signing**: Ed25519 produces consistent signatures for same message
- **Error clarity**: Specific error types for I/O, invalid keys, base64 issues
- **Key source tracking**: Returns `KeySource` enum to indicate origin (env var or file)
- **Public key methods**: `seed_base64()` used by export-key command, `public_key_fingerprint()` for logging

## Import Ordering

### TypeScript

Standard import order (enforced conceptually, no linter config):

1. React and external packages (`react`, `react-dom`, `zustand`, `@tanstack/react-virtual`)
2. Internal modules (`./types/`, `./hooks/`, `./utils/`)
3. Relative imports (`./App`, `../sibling`)
4. Type imports (`import type { ... }`)

### Rust

Standard ordering:

1. `use` statements for external crates
2. `use` statements for internal modules
3. `use` statements for types and traits

## Comments & Documentation

### TypeScript

| Type | When to Use | Format |
|------|-------------|--------|
| JSDoc | Public functions, hooks, interfaces | `/** ... */` |
| Inline | Complex logic or non-obvious code | `// Explanation` |
| Section dividers | Logically group related code | `// -------` comment blocks |
| TODO | Planned work | `// TODO: description` |
| FIXME | Known issues | `// FIXME: description` |

### Rust

| Type | When to Use | Format |
|------|-------------|--------|
| Doc comments | All public items | `/// ...` or `//! ...` |
| Line comments | Internal logic | `// explanation` |
| Example blocks | Complex public APIs | `/// # Examples` section |
| Panics section | Functions that can panic | `/// # Panics` section |
| Errors section | Fallible functions | `/// # Errors` section |
| Section markers | Organize related tests | `// =========` multi-line headers |

Example from integration tests:

```rust
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
    // ...
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
    // ...
}
```

## Git Conventions

### Commit Messages

Format: `type(scope): description`

| Type | Usage | Example |
|------|-------|---------|
| feat | New feature | `feat(monitor): add export-key subcommand for GitHub Actions` |
| fix | Bug fix | `fix(client): add vite-env.d.ts for ImportMeta.env types` |
| docs | Documentation | `docs: add GitHub Actions setup section to README` |
| style | Formatting | `style: fix indentation in error.rs` |
| refactor | Code restructure | `refactor(monitor): rename load_with_env to load_with_fallback` |
| test | Adding/updating tests | `test(monitor): add export-key integration tests (12 tests)` |
| chore | Maintenance | `chore: update Cargo.lock for zeroize dependency` |
| security | Security improvements | `security(monitor): zero intermediate key material buffers` |
| ci | CI/CD pipeline changes | `ci: add example workflow with VibeTea monitoring` |

### Branch Naming

Format: `{type}/{ticket}-{description}`

Example: `feat/004-monitor-gh-actions`

---

## What Does NOT Belong Here

- Test strategies → TESTING.md
- Security practices → SECURITY.md
- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md

---

*This document defines HOW to write code. Update when conventions change.*
