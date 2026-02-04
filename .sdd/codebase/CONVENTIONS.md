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

#### YAML (GitHub Actions)

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Enforced by GitHub Actions spec |
| Quotes | Single quotes for strings | `'value'` |
| Comments | `#` for explanations | Describe complex logic |
| Line length | 100 chars (soft) | Wrap long values with folded scalars |

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
| Modules | snake_case | `config.rs`, `error.rs`, `types.rs`, `watcher.rs`, `parser.rs`, `privacy.rs`, `crypto.rs`, `sender.rs`, `main.rs`, `project_tracker.rs` |
| Test files | `*_test.rs` in `tests/` directory | `env_key_test.rs`, `privacy_test.rs`, `sender_recovery_test.rs`, `key_export_test.rs` |
| Types | PascalCase | `Config`, `Event`, `ServerError`, `MonitorError`, `PrivacyConfig`, `Crypto`, `Sender`, `Command`, `ProjectTracker`, `ProjectActivityEvent` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT`, `DEFAULT_BUFFER_SIZE`, `SENSITIVE_TOOLS`, `PRIVATE_KEY_FILE`, `SHUTDOWN_TIMEOUT_SECS`, `STATS_DEBOUNCE_MS`, `PARSE_RETRY_DELAY_MS` |
| Test modules | `#[cfg(test)] mod tests` | In same file as implementation |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `from_env()`, `generate_event_id()`, `parse_jsonl_line()`, `extract_basename()`, `parse_args()`, `run_export_key()`, `parse_project_slug()`, `has_summary_event()` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT = 8080`, `SEED_LENGTH = 32`, `MAX_RETRY_DELAY_SECS = 60`, `ENV_VAR_NAME = "VIBETEA_PRIVATE_KEY"`, `ACTIVE_SESSION`, `COMPLETED_SESSION` |
| Structs | PascalCase | `Config`, `Event`, `PrivacyPipeline`, `Crypto`, `Sender`, `Command`, `ProjectTracker`, `ProjectTrackerConfig` |
| Enums | PascalCase | `EventType`, `SessionAction`, `ServerError`, `CryptoError`, `SenderError`, `Command`, `ProjectTrackerError` |
| Methods | snake_case | `.new()`, `.to_string()`, `.from_env()`, `.process()`, `.generate()`, `.load()`, `.save()`, `.sign()`, `.seed_base64()`, `.public_key_fingerprint()`, `.scan_projects()`, `.projects_dir()` |
| Lifetimes | Single lowercase letter | `'a`, `'static` |

### GitHub Actions/YAML

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Action directories | kebab-case | `.github/actions/vibetea-monitor/` |
| Action files | `action.yml` | `.github/actions/vibetea-monitor/action.yml` |
| Workflow files | kebab-case with `.yml` | `.github/workflows/ci.yml`, `.github/workflows/ci-with-monitor.yml` |
| Input parameters | kebab-case | `server-url`, `private-key`, `source-id` |
| Output parameters | kebab-case | `monitor-pid`, `monitor-started` |
| Environment variables | SCREAMING_SNAKE_CASE | `VIBETEA_PRIVATE_KEY`, `VIBETEA_SERVER_URL` |
| Step IDs | kebab-case | `download`, `start-monitor`, `save-cleanup-config` |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Input names | kebab-case with descriptions | `server-url`, `private-key`, `shutdown-timeout` |
| Output names | kebab-case with descriptions | `monitor-pid`, `monitor-started` |
| Step names | Title case with action purpose | `Download VibeTea Monitor`, `Start VibeTea Monitor` |
| Conditionals | GitHub context expressions | `if: always()`, `if: failure()` |
| Default values | Match input type | `default: 'latest'`, `default: '5'` |

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
| History.jsonl parsing (Phase 5) | Enum with specific variants | `HistoryParseError::InvalidJson`, `HistoryParseError::MissingDisplay`, `HistoryParseError::MissingTimestamp` |
| Skill tracker errors (Phase 5) | Enum with watcher/channel variants | `SkillTrackerError::WatcherError`, `SkillTrackerError::ChannelError` |
| Stats tracker errors (Phase 8) | Enum with watcher/parse/channel variants | `StatsTrackerError::WatcherInit`, `StatsTrackerError::Parse`, `StatsTrackerError::ChannelClosed` |
| Project tracker errors (Phase 11) | Enum with watcher/directory/channel variants | `ProjectTrackerError::WatcherInit`, `ProjectTrackerError::ClaudeDirectoryNotFound`, `ProjectTrackerError::ChannelClosed` |

#### GitHub Actions (YAML)

Error handling patterns in actions:

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Download failures | Conditional exit with warnings | `.github/actions/vibetea-monitor/action.yml` - Download step |
| Missing configuration | Warnings instead of failures | Start step checks for required inputs |
| Process startup failure | Non-blocking check with warning | Post-startup validation of PID |

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

**History Parser Example** (`monitor/src/trackers/skill_tracker.rs` - Phase 5):

```rust
#[derive(Debug, Error)]
pub enum HistoryParseError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("missing required field: display")]
    MissingDisplay,

    #[error("missing required field: timestamp")]
    MissingTimestamp,

    #[error("invalid timestamp value")]
    InvalidTimestamp,

    #[error("missing required field: sessionId")]
    MissingSessionId,
}
```

**Stats Tracker Example** (`monitor/src/trackers/stats_tracker.rs` - Phase 8):

```rust
#[derive(Error, Debug)]
pub enum StatsTrackerError {
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    #[error("failed to read stats cache: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse stats cache: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("claude directory not found: {0}")]
    ClaudeDirectoryNotFound(PathBuf),

    #[error("failed to send event: channel closed")]
    ChannelClosed,
}
```

**Project Tracker Example** (`monitor/src/trackers/project_tracker.rs` - Phase 11):

```rust
#[derive(Error, Debug)]
pub enum ProjectTrackerError {
    #[error("failed to create watcher: {0}")]
    WatcherInit(#[from] notify::Error),

    #[error("failed to read session file: {0}")]
    Io(#[from] std::io::Error),

    #[error("claude projects directory not found: {0}")]
    ClaudeDirectoryNotFound(PathBuf),

    #[error("failed to send event: channel closed")]
    ChannelClosed,
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

In GitHub Actions (YAML), use workflow commands for different levels:

```yaml
echo "::warning::VibeTea monitor binary download failed"
echo "::error::Failed to set environment variable"
echo "::notice::VibeTea monitor started successfully"
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

### GitHub Actions Composite Action Pattern (Phase 6)

GitHub Actions composite actions provide reusable workflow steps. The vibetea-monitor action (`.github/actions/vibetea-monitor/action.yml`) demonstrates this pattern:

```yaml
# Composite action metadata
name: 'VibeTea Monitor'
description: 'Start VibeTea monitor to track Claude Code events'
author: 'aaronbassett'

branding:
  icon: 'activity'
  color: 'green'

# Inputs with descriptions and defaults
inputs:
  server-url:
    description: 'URL of the VibeTea server'
    required: true
  private-key:
    description: 'Base64-encoded Ed25519 private key'
    required: true
  source-id:
    description: 'Custom source identifier'
    required: false
    default: ''
  version:
    description: 'Monitor version to download'
    required: false
    default: 'latest'
  shutdown-timeout:
    description: 'Timeout for graceful shutdown'
    required: false
    default: '5'

# Outputs for downstream steps
outputs:
  monitor-pid:
    description: 'Process ID of running monitor'
    value: ${{ steps.start-monitor.outputs.pid }}
  monitor-started:
    description: 'Whether monitor started successfully'
    value: ${{ steps.start-monitor.outputs.started }}

# Implementation as sequence of shell steps
runs:
  using: 'composite'
  steps:
    - name: Download VibeTea Monitor
      id: download
      shell: bash
      run: |
        # Download logic

    - name: Start VibeTea Monitor
      id: start-monitor
      shell: bash
      env:
        VIBETEA_PRIVATE_KEY: ${{ inputs.private-key }}
        VIBETEA_SERVER_URL: ${{ inputs.server-url }}
      run: |
        # Startup logic
```

Key conventions for GitHub Actions:
- **Descriptive names**: Action and step names clearly state their purpose
- **Required inputs**: Mark critical inputs as `required: true`
- **Sensible defaults**: Provide defaults for optional inputs (version, timeout)
- **Outputs**: Expose process ID and status for downstream steps
- **Error handling**: Use `::warning::` for non-critical failures, exit 0 to avoid blocking workflow
- **Environment variable safety**: Pass secrets via inputs, use step env vars
- **Step IDs**: Use kebab-case for reliable downstream reference
- **Shell specification**: Always specify `shell: bash` for portability
- **Non-blocking design**: Monitor startup failures should not fail the workflow
- **Cleanup guidance**: Document post-job cleanup via step comments

Usage pattern:

```yaml
# In any workflow job
- uses: aaronbassett/VibeTea/.github/actions/vibetea-monitor@main
  with:
    server-url: ${{ secrets.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    source-id: "pr-${{ github.event.pull_request.number }}"

# Run CI steps while monitor captures events
- name: Run Tests
  run: cargo test

# Graceful shutdown (optional)
- name: Stop VibeTea Monitor
  if: always()
  run: |
    if [ -n "$VIBETEA_MONITOR_PID" ]; then
      kill -TERM $VIBETEA_MONITOR_PID 2>/dev/null || true
      sleep ${{ inputs.shutdown-timeout }}
    fi
```

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

### File Watching Pattern (Phase 11)

The project_tracker module demonstrates file watching best practices used throughout the codebase:

```rust
// monitor/src/trackers/project_tracker.rs
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

// 1. Create channels for async communication
let (change_tx, change_rx) = mpsc::channel::<PathBuf>(1000);

// 2. Spawn background task for processing
let watcher = RecommendedWatcher::new(
    move |res: Result<Event, notify::Error>| {
        handle_notify_event(res, &projects_dir, &change_tx);
    },
    Config::default(),
)?;

// 3. Watch directory recursively
watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

// 4. Process events in async task
tokio::spawn(async move {
    process_file_changes(change_rx, sender, projects_dir).await;
});
```

Key patterns:
- **Channel-based communication**: Decouples file watching from event processing
- **Async task spawning**: Uses `tokio::spawn` for concurrent processing
- **Debouncing**: Applied as needed (project_tracker uses 0ms for session files per research)
- **Error handling**: Graceful degradation for file read errors or missing files
- **Recursive watching**: Enables monitoring subdirectories for new sessions

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

### Inline Module Testing Pattern (Phase 11)

Each tracker module organizes tests using the `#[cfg(test)] mod tests` pattern with section markers:

```rust
// monitor/src/trackers/project_tracker.rs (1,822 lines with 69 tests)
#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // T230: Unit test for project path slug parsing
    // =========================================================================

    #[test]
    fn parse_project_slug_standard_path() {
        // Standard Unix path
        let slug = "-home-ubuntu-Projects-VibeTea";
        let path = parse_project_slug(slug);
        assert_eq!(path, "/home/ubuntu/Projects/VibeTea");
    }

    // ... more tests organized in sections

    // =========================================================================
    // T233/T234: ProjectTracker with file watching tests
    // =========================================================================

    use tempfile::TempDir;
    use tokio::time::{sleep, timeout, Duration as TokioDuration};

    #[tokio::test]
    async fn test_tracker_detects_new_session_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        // ... async test with tempfile isolation
    }
}
```

Key conventions:
- **Section markers**: Use `// =========` comments to group related tests (typically 3-10 tests per section)
- **Test naming**: Tests named with descriptive verbs (e.g., `parse_project_slug_standard_path`)
- **Async tests**: Marked with `#[tokio::test]` for async operations
- **Isolation**: Uses `tempfile::TempDir` for file system isolation
- **Constants**: Test data defined at top (e.g., `ACTIVE_SESSION`, `COMPLETED_SESSION`)
- **Helper functions**: Utility functions like `create_test_project` for setup

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

Example from `monitor/src/trackers/agent_tracker.rs` (Phase 4):

```rust
//! Agent tracker for detecting Task tool agent spawns.
//!
//! This module extracts [`AgentSpawnEvent`] data from Task tool invocations
//! in Claude Code session JSONL files.
//!
//! # Task Tool Format
//!
//! When Claude Code spawns a subagent using the Task tool, the JSONL contains:
//! [example JSON structure]
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only the `subagent_type`
//! (as agent_type) and `description` are extracted. The `prompt` field is
//! never transmitted or stored.

/// Task tool input parameters.
///
/// Represents the `input` field of a Task tool_use content block.
/// Only the metadata fields needed for event creation are extracted;
/// the `prompt` field is intentionally omitted for privacy.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TaskToolInput {
    /// The type of subagent being spawned (e.g., "devs:rust-dev", "task").
    ///
    /// If not present in the input, defaults to "task".
    #[serde(default = "default_subagent_type")]
    pub subagent_type: String,

    /// Description of the task being delegated to the subagent.
    ///
    /// If not present in the input, defaults to an empty string.
    #[serde(default)]
    pub description: String,
}

/// Parses a Task tool_use input, extracting the relevant metadata.
///
/// This function takes the tool name and input from a `ContentBlock::ToolUse`
/// and returns `Some(TaskToolInput)` if the tool is the Task tool.
///
/// # Arguments
///
/// * `tool_name` - The name of the tool (must be "Task" for a match)
/// * `input` - The tool input as a JSON value
///
/// # Returns
///
/// * `Some(TaskToolInput)` if the tool is "Task" and the input can be parsed
/// * `None` if the tool is not "Task" or parsing fails
#[must_use]
pub fn parse_task_tool_use(tool_name: &str, input: &serde_json::Value) -> Option<TaskToolInput> {
    // Only process Task tool invocations
    if tool_name != "Task" {
        return None;
    }

    // Attempt to deserialize the input; return None on parse failure
    serde_json::from_value(input.clone()).ok()
}
```

Example from `monitor/src/trackers/skill_tracker.rs` (Phase 5):

```rust
//! Skill tracker for detecting skill/slash command invocations.
//!
//! This module watches `~/.claude/history.jsonl` for changes and emits
//! [`SkillInvocationEvent`]s for each new skill invocation.
//!
//! # History.jsonl Format
//!
//! When a user invokes a skill (slash command) in Claude Code, an entry is
//! appended to `~/.claude/history.jsonl`:
//!
//! ```json
//! {
//!   "display": "/commit -m \"fix: update docs\"",
//!   "timestamp": 1738567268363,
//!   "project": "/home/ubuntu/Projects/VibeTea",
//!   "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"
//! }
//! ```
//!
//! # Privacy
//!
//! This module follows the privacy-first principle: only the skill name
//! (extracted from `display`) and metadata are captured. Command arguments
//! are intentionally not transmitted.
//!
//! # Architecture
//!
//! The tracker uses the [`notify`] crate to watch for file changes. Since
//! `history.jsonl` is append-only, the tracker maintains a byte offset to
//! only read new lines (tail-like behavior). No debounce is used - events
//! are processed immediately per the research.md specification.

/// A parsed entry from history.jsonl.
///
/// Represents a single skill invocation record as stored by Claude Code.
/// The JSON uses camelCase field names which are mapped to snake_case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// The skill command as displayed (e.g., "/commit -m \"message\"").
    pub display: String,

    /// Unix timestamp in milliseconds when the skill was invoked.
    pub timestamp: i64,

    /// The project path where the skill was invoked.
    pub project: String,

    /// The session ID associated with this skill invocation.
    pub session_id: String,
}
```

Example from `monitor/src/trackers/project_tracker.rs` (Phase 11):

```rust
//! Project tracker for monitoring active Claude Code sessions per project.
//!
//! This module scans `~/.claude/projects/` to identify projects and their
//! session activity status by checking for the presence of summary events.
//!
//! # Directory Structure
//!
//! ```text
//! ~/.claude/projects/
//! +-- -home-ubuntu-Projects-VibeTea/
//! |   +-- 6e45a55c-3124-4cc8-ad85-040a5c316009.jsonl  # Active session
//! |   +-- a1b2c3d4-5678-90ab-cdef-1234567890ab.jsonl  # Completed session
//! +-- -home-ubuntu-Projects-SMILE/
//!     +-- 60fc5b5e-a285-4a6d-b9cc-9a315eb90ea8.jsonl
//! ```
//!
//! # Path Slug Format
//!
//! Project directories use a "slug" format where the absolute path has
//! forward slashes replaced with dashes:
//! - `/home/ubuntu/Projects/VibeTea` becomes `-home-ubuntu-Projects-VibeTea`
//!
//! # Session Activity Detection
//!
//! A session is considered **active** if its JSONL file does not contain
//! a summary event (`{"type": "summary", ...}`). Once a summary event
//! is written, the session is considered **completed**.

/// Parses a project directory slug back to its original absolute path.
///
/// Project directories in `~/.claude/projects/` use a "slug" format where
/// forward slashes in the path are replaced with dashes. This function
/// reverses that transformation.
///
/// # Arguments
///
/// * `slug` - The project directory name (e.g., `-home-ubuntu-Projects-VibeTea`)
///
/// # Returns
///
/// The original absolute path (e.g., `/home/ubuntu/Projects/VibeTea`)
///
/// # Examples
///
/// ```
/// use vibetea_monitor::trackers::project_tracker::parse_project_slug;
///
/// let path = parse_project_slug("-home-ubuntu-Projects-VibeTea");
/// assert_eq!(path, "/home/ubuntu/Projects/VibeTea");
/// ```
#[must_use]
pub fn parse_project_slug(slug: &str) -> String {
    // The slug format replaces '/' with '-'
    // A leading dash represents the root '/'
    slug.replace('-', "/")
}
```

Example from `monitor/src/trackers/stats_tracker.rs` (Phase 8):

```rust
//! Stats cache tracker for monitoring Claude Code's token usage statistics.
//!
//! This module watches `~/.claude/stats-cache.json` for changes and emits
//! [`StatsEvent`]s containing both [`SessionMetricsEvent`] and [`TokenUsageEvent`]
//! data.
//!
//! # File Format
//!
//! The stats-cache.json file has the following structure:
//!
//! ```json
//! {
//!   "totalSessions": 150,
//!   "totalMessages": 2500,
//!   "totalToolUsage": 8000,
//!   "longestSession": "00:45:30",
//!   "hourCounts": { "0": 10, "1": 5, ..., "23": 50 },
//!   "modelUsage": {
//!     "claude-sonnet-4-20250514": {
//!       "inputTokens": 1500000,
//!       "outputTokens": 300000,
//!       "cacheReadInputTokens": 800000,
//!       "cacheCreationInputTokens": 100000
//!     }
//!   }
//! }
//! ```
//!
//! # Architecture
//!
//! The tracker uses the [`notify`] crate to watch for file changes, with a 200ms
//! debounce to coalesce rapid file updates. When a change is detected:
//!
//! 1. The JSON file is parsed with retry on failure (file may be mid-write)
//! 2. A [`SessionMetricsEvent`] is emitted once per file read
//! 3. A [`TokenUsageEvent`] is emitted for each model in modelUsage
```

### GitHub Actions (YAML)

| Type | When to Use | Format |
|------|-------------|--------|
| Action description | Every action metadata | `description:` field |
| Step names | Every workflow step | Clear, descriptive title |
| Step comments | Complex logic | YAML comments with `#` |
| Inline docs | In action implementation | Shell comments explaining logic |

Example from `.github/actions/vibetea-monitor/action.yml`:

```yaml
# Composite action metadata
name: 'VibeTea Monitor'
description: 'Start VibeTea monitor to track Claude Code events during GitHub Actions workflows'
author: 'aaronbassett'

# Detailed input descriptions
inputs:
  server-url:
    description: 'URL of the VibeTea server'
    required: true
  private-key:
    description: 'Base64-encoded Ed25519 private key (from vibetea-monitor export-key)'
    required: true

# Steps with clear names
steps:
  - name: Download VibeTea Monitor
    id: download
    shell: bash
    run: |
      # Download logic with inline comments explaining steps
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

Examples with Phase 11:
- `feat(monitor): add project_tracker for monitoring Claude Code project sessions`
- `test(monitor): add 69 unit tests for project_tracker (slug parsing, activity detection, file watching)`

Examples with Phase 5:
- `feat(monitor): implement skill_tracker file watching for history.jsonl`
- `feat(monitor): add skill name extraction from history.jsonl display field`
- `test(monitor): add 20+ unit tests for skill_tracker parsing and event creation`

Examples with Phase 4:
- `feat(monitor): add agent_tracker for Task tool agent spawn extraction`
- `test(monitor): add 28 unit tests for agent_tracker parsing and event creation`
- `feat(parser): emit AgentSpawned events for Task tool invocations`

### Branch Naming

Format: `{type}/{ticket}-{description}`

Example: `feat/011-project-tracker` or `feat/005-monitor-enhanced-tracking`

---

## What Does NOT Belong Here

- Test strategies → TESTING.md
- Security practices → SECURITY.md
- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md

---

*This document defines HOW to write code. Update when conventions change.*
