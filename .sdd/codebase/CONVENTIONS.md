# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

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
| Components | PascalCase | `ConnectionStatus.tsx`, `TokenForm.tsx` |
| Hooks | camelCase with `use` prefix | `useEventStore.ts`, `useWebSocket.ts` |
| Types | PascalCase in `types/` folder | `types/events.ts` contains `VibeteaEvent` |
| Utilities | camelCase | `utils/` exists (placeholder) |
| Constants | SCREAMING_SNAKE_CASE in const files | `MAX_EVENTS = 1000`, `TOKEN_STORAGE_KEY` |
| Test files | Same as source + `.test.ts` | `__tests__/events.test.ts` |
| Test directories | `__tests__/` at feature level | Co-located with related source |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Variables | camelCase | `sessionId`, `eventCount`, `wsRef`, `connectRef` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_EVENTS`, `DEFAULT_BUFFER_SIZE`, `TOKEN_STORAGE_KEY` |
| Functions | camelCase, verb prefix | `selectEventsBySession()`, `isSessionEvent()`, `parseEventMessage()`, `calculateBackoff()` |
| Classes | PascalCase (rare in modern React) | N/A |
| Interfaces | PascalCase, no `I` prefix | `EventStore`, `Session`, `VibeteaEvent`, `UseWebSocketReturn`, `ConnectionStatusProps` |
| Types | PascalCase | `VibeteaEvent<T>`, `EventPayload`, `ConnectionStatus`, `TokenStatus` |
| Type guards | `is` prefix | `isSessionEvent()`, `isValidEventType()` |
| Enums | PascalCase | N/A (use union types instead) |
| Refs | camelCase with `Ref` suffix | `wsRef`, `reconnectTimeoutRef`, `connectRef` |

### Rust/Server/Monitor

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `config.rs`, `error.rs`, `types.rs`, `watcher.rs`, `parser.rs`, `privacy.rs`, `crypto.rs`, `sender.rs`, `main.rs` |
| Types | PascalCase | `Config`, `Event`, `ServerError`, `MonitorError`, `PrivacyConfig`, `Crypto`, `Sender`, `Command` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT`, `DEFAULT_BUFFER_SIZE`, `SENSITIVE_TOOLS`, `PRIVATE_KEY_FILE`, `SHUTDOWN_TIMEOUT_SECS` |
| Test modules | `#[cfg(test)] mod tests` | In same file as implementation |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `from_env()`, `generate_event_id()`, `parse_jsonl_line()`, `extract_basename()`, `parse_args()` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT = 8080`, `SEED_LENGTH = 32`, `MAX_RETRY_DELAY_SECS = 60` |
| Structs | PascalCase | `Config`, `Event`, `PrivacyPipeline`, `Crypto`, `Sender`, `Command` |
| Enums | PascalCase | `EventType`, `SessionAction`, `ServerError`, `CryptoError`, `SenderError`, `Command` |
| Methods | snake_case | `.new()`, `.to_string()`, `.from_env()`, `.process()`, `.generate()`, `.load()`, `.save()`, `.sign()` |
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
| Cryptographic errors | String-based variants | `CryptoError::InvalidKey`, `CryptoError::KeyExists` |
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

**Crypto Example** (`monitor/src/crypto.rs` - Phase 6):

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
}
```

**Sender Example** (`monitor/src/sender.rs` - Phase 6):

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

### React Hook Pattern (TypeScript - Phase 7)

Custom React hooks follow a structured pattern with refs, callbacks, and effects:

**From `useWebSocket.ts`** (Phase 7):

```typescript
/**
 * WebSocket connection hook with auto-reconnect and exponential backoff.
 *
 * Manages WebSocket lifecycle and automatically dispatches received events
 * to the event store. Supports manual connection control and automatic
 * reconnection with exponential backoff (1s initial, 60s max, ±25% jitter).
 */
export function useWebSocket(url?: string): UseWebSocketReturn {
  // Refs for persistent values across renders
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectAttemptRef = useRef<number>(0);
  const shouldReconnectRef = useRef<boolean>(true);

  // Get store selectors and actions
  const addEvent = useEventStore((state) => state.addEvent);
  const setStatus = useEventStore((state) => state.setStatus);

  // Callbacks for event handlers
  const connect = useCallback(() => {
    // Establish connection logic
  }, [dependencies]);

  const disconnect = useCallback(() => {
    // Cleanup logic
  }, [dependencies]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // Cleanup resources
    };
  }, [dependencies]);

  return { connect, disconnect, isConnected: status === 'connected' };
}
```

Key patterns:
1. **Refs**: Use for WebSocket instance, timers, and mutable state that shouldn't trigger re-renders
2. **Callbacks**: Wrap event handlers in `useCallback` to prevent infinite effect loops
3. **Effects**: Handle setup/cleanup, reconnection scheduling, and external subscriptions
4. **Derived state**: Return computed values like `isConnected: status === 'connected'`
5. **Documentation**: Include JSDoc with examples showing usage patterns

### React Component Pattern (TypeScript - Phase 7)

Functional components follow a consistent structure:

**From `ConnectionStatus.tsx`** (Phase 7):

```typescript
/**
 * Props for the ConnectionStatus component.
 */
interface ConnectionStatusProps {
  /** Whether to show the status text label. Defaults to false. */
  readonly showLabel?: boolean;
  /** Additional CSS classes to apply to the container. */
  readonly className?: string;
}

/**
 * Displays the current WebSocket connection status.
 *
 * Uses selective Zustand subscription to only re-render when status changes,
 * preventing unnecessary updates during high-frequency event streams.
 */
export function ConnectionStatus({
  showLabel = false,
  className = '',
}: ConnectionStatusProps) {
  // Selective subscription: only re-render when status changes
  const status = useEventStore((state) => state.status);

  const config = STATUS_CONFIG[status];

  return (
    <div className={`inline-flex items-center gap-2 ${className}`}>
      {/* Component JSX */}
    </div>
  );
}
```

Key patterns:
1. **Props interface**: Define props with JSDoc annotations for optional fields and defaults
2. **Selective subscriptions**: Use Zustand selectors to minimize re-renders
3. **Constants**: Define configuration objects outside components (e.g., `STATUS_CONFIG`)
4. **Accessibility**: Include ARIA attributes and semantic roles
5. **Tailwind classes**: Use utility-first approach for styling

### Form Handling Pattern (TypeScript - Phase 7)

Form components manage state and handle submissions:

**From `TokenForm.tsx`** (Phase 7):

```typescript
/**
 * Form for managing the authentication token.
 *
 * Provides a password input for entering the token, with save and clear buttons.
 * Token is persisted to localStorage for use by the WebSocket connection.
 */
export function TokenForm({
  onTokenChange,
  className = '',
}: TokenFormProps) {
  // State management
  const [tokenInput, setTokenInput] = useState<string>('');
  const [status, setStatus] = useState<TokenStatus>(() =>
    hasStoredToken() ? 'saved' : 'not-saved'
  );

  // Listen to storage changes from other tabs
  useEffect(() => {
    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === TOKEN_STORAGE_KEY) {
        setStatus(event.newValue !== null ? 'saved' : 'not-saved');
      }
    };

    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, []);

  // Event handlers with proper types
  const handleSave = useCallback(
    (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const trimmedToken = tokenInput.trim();
      if (trimmedToken === '') return;

      localStorage.setItem(TOKEN_STORAGE_KEY, trimmedToken);
      setStatus('saved');
      setTokenInput('');
      onTokenChange?.();
    },
    [tokenInput, onTokenChange]
  );

  return (
    <form onSubmit={handleSave}>
      {/* Form fields */}
    </form>
  );
}
```

Key patterns:
1. **useState with lazy init**: Use callback for initialization based on localStorage
2. **useCallback dependencies**: Include all dependencies to prevent stale closures
3. **Event types**: Use React event types like `React.FormEvent<HTMLFormElement>`
4. **Optional callbacks**: Use optional chaining with callbacks (`onTokenChange?.()`)
5. **localStorage handling**: Abstract into helper functions (e.g., `hasStoredToken()`)
6. **Cross-tab sync**: Listen to `storage` events for multi-tab consistency

### Exponential Backoff Pattern (TypeScript - Phase 7)

Implement reconnection delays with jitter:

**From `useWebSocket.ts`** (Phase 7):

```typescript
/**
 * Calculate reconnection delay with exponential backoff and jitter.
 *
 * @param attempt - Current reconnection attempt number (0-indexed)
 * @returns Delay in milliseconds with jitter applied
 */
function calculateBackoff(attempt: number): number {
  // Exponential backoff: initial * 2^attempt, capped at max
  const exponentialDelay = Math.min(
    INITIAL_BACKOFF_MS * Math.pow(2, attempt),
    MAX_BACKOFF_MS
  );

  // Apply jitter: ±25% randomization
  const jitter = 1 + (Math.random() * 2 - 1) * JITTER_FACTOR;

  return Math.round(exponentialDelay * jitter);
}
```

Constants match Rust implementation:
- `INITIAL_BACKOFF_MS = 1000` (1 second)
- `MAX_BACKOFF_MS = 60000` (60 seconds)
- `JITTER_FACTOR = 0.25` (±25% randomization)

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

### Error Handling Pattern (Rust)

Create typed error enums with helper constructors:

```rust
impl ServerError {
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn rate_limit(source: impl Into<String>, retry_after: u64) -> Self {
        Self::RateLimit {
            source: source.into(),
            retry_after,
        }
    }

    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Auth(_) | Self::Validation(_) | Self::RateLimit { .. }
        )
    }
}
```

### Privacy Pipeline Pattern (Rust - Phase 5)

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

### Cryptographic Operations Pattern (Rust - Phase 6)

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

    // Saves keypair with secure file permissions (0600 for private key)
    pub fn save(&self, dir: &Path) -> Result<(), CryptoError> { ... }

    // Checks if keypair already exists
    pub fn exists(dir: &Path) -> bool { ... }

    // Signs a message and returns base64-encoded signature
    pub fn sign(&self, message: &[u8]) -> String { ... }

    // Signs and returns raw 64-byte signature
    pub fn sign_raw(&self, message: &[u8]) -> [u8; 64] { ... }
}
```

Key conventions in crypto module:
- **Key storage**: Private key stored as raw 32-byte seed in `key.priv`, public key as base64 in `key.pub`
- **File permissions**: Unix permissions set to 0600 (private key) and 0644 (public key)
- **Deterministic signing**: Ed25519 produces consistent signatures for same message
- **Error clarity**: Specific error types for I/O, invalid keys, base64 issues

### HTTP Sender Pattern (Rust - Phase 6)

The sender module (`monitor/src/sender.rs`) handles sending events to the server with buffering and retry logic:

```rust
// Configuration for the sender
pub struct SenderConfig {
    pub server_url: String,
    pub source_id: String,
    pub buffer_size: usize,  // Default: 1000
}

// HTTP event sender with buffering and retry logic
pub struct Sender {
    config: SenderConfig,
    crypto: Crypto,
    client: Client,
    buffer: VecDeque<Event>,
    current_retry_delay: Duration,
}

impl Sender {
    // Creates new sender with connection pooling via reqwest
    pub fn new(config: SenderConfig, crypto: Crypto) -> Self { ... }

    // Queues an event for buffering (evicts oldest if full)
    pub fn queue(&mut self, event: Event) -> usize { ... }

    // Sends a single event immediately without buffering
    pub async fn send(&mut self, event: Event) -> Result<(), SenderError> { ... }

    // Flushes all buffered events in a single batch
    pub async fn flush(&mut self) -> Result<(), SenderError> { ... }

    // Gracefully shuts down, attempting to flush remaining events
    pub async fn shutdown(&mut self, timeout: Duration) -> usize { ... }
}
```

Key conventions in sender module:
- **Buffering strategy**: FIFO queue with configurable size (default 1000 events)
- **Exponential backoff retry**: 1s initial delay → 60s max, with ±25% jitter
- **Rate limit handling**: Parses Retry-After header from 429 responses
- **Authentication**: Signs events using crypto module (Ed25519)
- **Structured logging**: Uses `tracing` crate for info/warn/debug/error logging

### CLI Pattern (Rust - Phase 6)

The main binary (`monitor/src/main.rs`) implements a simple command-line interface with async runtime management:

#### Command Enum and Parsing

```rust
#[derive(Debug)]
enum Command {
    Init { force: bool },
    Run,
    Help,
    Version,
}

fn parse_args() -> Result<Command> {
    // Manual argument parsing for: init, run, help, version
    // Supports: --force/-f for init, --help/-h, --version/-V
}
```

#### Async Runtime Initialization

The CLI uses explicit Tokio runtime creation for async commands:

```rust
fn main() -> Result<()> {
    let command = parse_args()?;

    match command {
        Command::Run => {
            // Initialize multi-threaded async runtime only for async commands
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("Failed to create tokio runtime")?;

            // Block on async function using the runtime
            runtime.block_on(run_monitor())
        }
        // Sync commands run directly
        Command::Init { force } => run_init(force),
        // ...
    }
}
```

#### Signal Handling

Graceful shutdown using `tokio::signal`:

```rust
async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for either signal
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
```

#### Logging Initialization

Configure structured logging with environment variable control:

```rust
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .init();
}
```

Key conventions in CLI:
- **Simple argument parsing**: No external CLI library, manual matching of command names
- **Error handling**: Uses `anyhow::Result` for ergonomic error propagation
- **Async runtime**: Explicit multi-threaded Tokio runtime created only when needed
- **Signal handling**: Handles both Ctrl+C (SIGINT) and SIGTERM, with platform-specific handling
- **Graceful shutdown**: Attempts to flush unsent events before exiting
- **Logging**: Uses `tracing` with environment-driven verbosity control
- **Help/version**: Standard `--help` and `--version` flags supported

## Import Ordering

### TypeScript

Standard import order (enforced conceptually, no linter config):

1. React and external packages (`react`, `react-dom`, `zustand`)
2. Internal modules (`./types/`, `./hooks/`)
3. Relative imports (`./App`, `../sibling`)
4. Type imports (`import type { ... }`)

Example from `useWebSocket.ts` (Phase 7):

```typescript
import { useCallback, useEffect, useRef } from 'react';

import type { VibeteaEvent } from '../types/events';
import { useEventStore } from './useEventStore';
```

Example from `ConnectionStatus.tsx` (Phase 7):

```typescript
import { useEventStore } from '../hooks/useEventStore';

import type { ConnectionStatus as ConnectionStatusType } from '../hooks/useEventStore';
```

Example from `useEventStore.ts`:

```typescript
import { create } from 'zustand';

import type { Session, VibeteaEvent } from '../types/events';
```

### Rust

Standard ordering:

1. `use` statements for external crates
2. `use` statements for internal modules
3. `use` statements for types and traits

Example from `server/src/error.rs`:

```rust
use std::error::Error;
use std::fmt;

use thiserror::Error as ThisError;
```

Example from `monitor/src/crypto.rs` (Phase 6):

```rust
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use base64::prelude::*;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::Rng;
use thiserror::Error;
```

Example from `monitor/src/main.rs` (Phase 6):

```rust
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use directories::BaseDirs;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use vibetea_monitor::config::Config;
use vibetea_monitor::crypto::Crypto;
use vibetea_monitor::sender::{Sender, SenderConfig};
```

## Comments & Documentation

### TypeScript

| Type | When to Use | Format |
|------|-------------|--------|
| JSDoc | Public functions, hooks, interfaces | `/** ... */` |
| Inline | Complex logic or non-obvious code | `// Explanation` |
| Section dividers | Logically group related code | `// -------` comment blocks |
| TODO | Planned work | `// TODO: description` |
| FIXME | Known issues | `// FIXME: description` |

Example from `useWebSocket.ts` (Phase 7):

```typescript
/**
 * WebSocket connection hook for VibeTea client.
 *
 * Provides WebSocket connection management with automatic reconnection
 * using exponential backoff. Integrates with useEventStore for event dispatch.
 */

/**
 * Calculate reconnection delay with exponential backoff and jitter.
 *
 * @param attempt - Current reconnection attempt number (0-indexed)
 * @returns Delay in milliseconds with jitter applied
 */
function calculateBackoff(attempt: number): number {
  // Exponential backoff: initial * 2^attempt, capped at max
  const exponentialDelay = Math.min(
    INITIAL_BACKOFF_MS * Math.pow(2, attempt),
    MAX_BACKOFF_MS
  );

  // Apply jitter: ±25% randomization
  const jitter = 1 + (Math.random() * 2 - 1) * JITTER_FACTOR;

  return Math.round(exponentialDelay * jitter);
}
```

Example from `useEventStore.ts`:

```typescript
/**
 * Zustand store for managing WebSocket event state.
 *
 * Provides centralized state management for the VibeTea event stream,
 * with selective subscriptions to prevent unnecessary re-renders
 * during high-frequency event updates.
 */
```

Example from `types/events.ts`:

```typescript
/**
 * Type guard to check if an event is a session event.
 */
export function isSessionEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'session'> {
  return event.type === 'session';
}

/**
 * Valid event type values for runtime validation.
 */
const VALID_EVENT_TYPES = [
  'session',
  'activity',
  'tool',
  'agent',
  'summary',
  'error',
] as const;
```

### Rust

| Type | When to Use | Format |
|------|-------------|--------|
| Doc comments | All public items | `/// ...` or `//! ...` |
| Line comments | Internal logic | `// explanation` |
| Example blocks | Complex public APIs | `/// # Examples` section |
| Panics section | Functions that can panic | `/// # Panics` section |
| Errors section | Fallible functions | `/// # Errors` section |
| Section markers | Organize related tests | `// =========` multi-line headers |

Example from `server/src/config.rs`:

```rust
/// Server configuration parsed from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Map of source_id to base64-encoded Ed25519 public key.
    pub public_keys: HashMap<String, String>,

    /// Authentication token for subscriber clients.
    pub subscriber_token: Option<String>,

    /// HTTP server port.
    pub port: u16,
}

impl Config {
    /// Parse configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - Required environment variables are missing
    /// - Environment variables have invalid format
    /// - Port number is not a valid u16
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_server::config::Config;
    ///
    /// let config = Config::from_env().expect("Failed to load config");
    /// println!("Server will listen on port {}", config.port);
    /// ```
    pub fn from_env() -> Result<Self, ConfigError> {
        // ...
    }
}
```

Example from `monitor/src/crypto.rs` (Phase 6):

```rust
//! Cryptographic operations for VibeTea Monitor.
//!
//! This module handles Ed25519 keypair generation, storage, and event signing.
//! Keys are stored in the VibeTea directory (`~/.vibetea/` by default):
//!
//! - `key.priv`: Raw 32-byte Ed25519 seed (file mode 0600)
//! - `key.pub`: Base64-encoded public key (file mode 0644)

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
    pub fn generate() -> Self { ... }
}
```

Example from `monitor/src/sender.rs` (Phase 6):

```rust
//! HTTP sender for VibeTea Monitor.
//!
//! This module handles sending events to the VibeTea server with:
//!
//! - Connection pooling via reqwest
//! - Event buffering (1000 events max, FIFO eviction)
//! - Exponential backoff retry (1s → 60s max, ±25% jitter)
//! - Rate limit handling (429 with Retry-After header)

/// HTTP event sender with buffering and retry logic.
pub struct Sender {
    config: SenderConfig,
    crypto: Crypto,
    client: Client,
    buffer: VecDeque<Event>,
    current_retry_delay: Duration,
}

impl Sender {
    /// Creates a new sender with the given configuration and cryptographic context.
    ///
    /// # Arguments
    ///
    /// * `config` - Sender configuration
    /// * `crypto` - Cryptographic context for signing events
    #[must_use]
    pub fn new(config: SenderConfig, crypto: Crypto) -> Self { ... }
}
```

Example from `monitor/src/main.rs` (Phase 6):

```rust
//! VibeTea Monitor - Claude Code session watcher.
//!
//! This binary watches Claude Code session files and forwards privacy-filtered
//! events to the VibeTea server.
//!
//! # Commands
//!
//! - `vibetea-monitor init`: Generate Ed25519 keypair for server authentication
//! - `vibetea-monitor run`: Start the monitor daemon
//!
//! # Environment Variables
//!
//! See the [`config`] module for available configuration options.

/// CLI command.
#[derive(Debug)]
enum Command {
    /// Initialize keypair.
    Init { force: bool },
    /// Run the monitor.
    Run,
    /// Show help.
    Help,
    /// Show version.
    Version,
}
```

Example from `monitor/src/privacy.rs` (Phase 5):

```rust
//! Privacy pipeline for VibeTea Monitor.
//!
//! This module ensures no sensitive data (source code, file contents, full paths,
//! prompts, commands) is ever transmitted to the server.
//!
//! # Privacy Guarantees
//!
//! The privacy pipeline provides the following guarantees:
//! - **Path-to-basename conversion**: Full paths like `/home/user/src/auth.ts` → `auth.ts`
//! - **Content stripping**: File contents and code never transmitted
//! - **Sensitive tool masking**: Bash, Grep, Glob, WebSearch, WebFetch context always stripped
//! - **Extension allowlist filtering**: Optional filtering by file extension

/// Tools whose context should always be stripped for privacy.
///
/// These tools may contain sensitive information:
/// - `Bash`: Contains shell commands which may include secrets, passwords, or API keys
/// - `Grep`: Contains search patterns which may reveal what the user is looking for
/// - `Glob`: Contains file patterns which may reveal project structure
/// - `WebSearch`: Contains search queries which may reveal user intent
/// - `WebFetch`: Contains URLs which may contain sensitive information
const SENSITIVE_TOOLS: &[&str] = &["Bash", "Grep", "Glob", "WebSearch", "WebFetch"];
```

## Git Conventions

### Commit Messages

Format: `type(scope): description`

| Type | Usage | Example |
|------|-------|---------|
| feat | New feature | `feat(client): add event store` |
| fix | Bug fix | `fix(server): handle missing env var` |
| docs | Documentation | `docs: update conventions` |
| style | Formatting changes | `style: fix ESLint warnings` |
| refactor | Code restructure | `refactor(config): simplify validation` |
| test | Adding/updating tests | `test(client): add initial event type tests` |
| chore | Maintenance, dependencies | `chore: ignore TypeScript build artifacts` |

Examples with Phase 7:
- `feat(client): add WebSocket connection hook with auto-reconnect`
- `feat(client): add connection status indicator component`
- `feat(client): add token form for authentication`

Examples with Phase 6:
- `feat(monitor): implement CLI with init and run commands`
- `feat(monitor): add HTTP sender with retry and buffering`
- `feat(monitor): add Ed25519 keypair generation and signing`

### Branch Naming

Format: `{type}/{ticket}-{description}`

Example: `feat/001-event-types`

---

## What Does NOT Belong Here

- Test strategies → TESTING.md
- Security practices → SECURITY.md
- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md

---

*This document defines HOW to write code. Update when conventions change.*
