# Architecture

**Status**: Phase 6 incremental update - Monitor crypto, sender, and CLI modules
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Architecture Overview

VibeTea is a three-tier real-time event streaming system with clear separation of concerns:

- **Monitor** (Rust): Event producer that watches Claude Code session files and captures activity with privacy guarantees
- **Server** (Rust): Event hub that authenticates monitors and broadcasts to clients
- **Client** (TypeScript/React): Event consumer that displays sessions and activities

The system follows a hub-and-spoke pattern where monitors are trusted publishers and clients are passive subscribers. All communication is event-driven with no persistent state required on the server.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Monitors push events to the server, clients subscribe via WebSocket |
| **Event-Driven** | All state changes flow through immutable, versioned events |
| **Layered** | Monitor: CLI/Config → Watcher/Parser/Privacy → Crypto/Sender → Types; Server: Routes → Auth/Broadcast/RateLimit → Types; Client: Types → Hooks → Components |
| **Pub/Sub** | Server acts as event broker with asymmetric authentication (monitors sign, clients consume) |
| **Token Bucket** | Per-source rate limiting using token bucket algorithm with stale entry cleanup |
| **File Tailing** | Monitor uses position tracking to efficiently read only new content from JSONL files |
| **Privacy Pipeline** | Multi-stage data sanitization ensuring no sensitive data leaves the monitor (Phase 5) |
| **Command-Line Interface** | Monitor CLI with `init` and `run` subcommands for key generation and daemon execution (Phase 6) |
| **Event Buffering** | Monitor buffers events in memory (1000 max, FIFO) before batch transmission with exponential backoff retry (Phase 6) |

## Core Components

### Server Routes & HTTP Layer

**Purpose**: Handle HTTP requests and WebSocket upgrades
**Location**: `server/src/routes.rs`
**Responsibility**: Route handling, request parsing, response generation

**Key Functions**:
- `create_router()` - Axum router setup with three endpoints (POST /events, GET /ws, GET /health)
- `post_events()` - Event ingestion handler with signature verification and rate limiting
- `get_ws()` - WebSocket upgrade handler with token validation
- `get_health()` - Health check endpoint (uptime and subscriber count)
- `handle_websocket()` - WebSocket connection handler with event forwarding and filtering

**AppState Structure**:
```rust
pub struct AppState {
    pub config: Arc<Config>,           // Server configuration (auth settings, port)
    pub broadcaster: EventBroadcaster,  // Event distribution hub
    pub rate_limiter: RateLimiter,      // Per-source rate limiting
    pub start_time: Instant,            // Server uptime tracking
}
```

**API Contracts**:
- `POST /events` - Accepts single or batch events with Ed25519 signature verification
- `GET /ws` - WebSocket subscription with optional filtering (source, type, project)
- `GET /health` - Returns status, connection count, and uptime

### Authentication Module

**Purpose**: Verify Ed25519 signatures and validate bearer tokens
**Location**: `server/src/auth.rs`
**Responsibility**: Cryptographic verification and security

**Key Functions**:
- `verify_signature()` - Ed25519 signature verification against request body
- `validate_token()` - Constant-time token comparison for WebSocket clients
- Error handling with specific failure reasons (UnknownSource, InvalidSignature, InvalidBase64, InvalidPublicKey, InvalidToken)

**Security Features**:
- Constant-time comparison using `subtle` crate to prevent timing attacks
- Base64 decoding with error handling
- Configurable public keys from environment variables
- Per-source authentication for monitors

### Event Broadcasting

**Purpose**: Distribute events to multiple WebSocket subscribers
**Location**: `server/src/broadcast.rs`
**Responsibility**: Multi-producer, multi-consumer event distribution

**Key Components**:
- `EventBroadcaster` - Central hub using tokio broadcast channel
  - Default capacity: 1000 events
  - `broadcast()` - Send event to all subscribers
  - `subscribe()` - Create new receiver
  - `subscriber_count()` - Get active connection count

- `SubscriberFilter` - Optional filtering criteria using AND logic
  - Filter by source ID
  - Filter by event type
  - Filter by project name
  - Extraction of project field from event payload

**Design Pattern**: Tokio broadcast channel with overflow handling (oldest events dropped)

### Rate Limiting

**Purpose**: Protect against excessive requests from individual sources
**Location**: `server/src/rate_limit.rs`
**Responsibility**: Token bucket rate limiting with stale entry cleanup

**Key Components**:
- `RateLimiter` - Thread-safe per-source tracking
  - Default: 100 requests/second per source, burst capacity 100
  - `check_rate_limit()` - Check if request is allowed
  - `cleanup_stale_entries()` - Remove inactive sources
  - `spawn_cleanup_task()` - Background cleanup every 30 seconds

- `TokenBucket` - Per-source bucket implementation
  - Refill at configurable rate
  - Constant capacity
  - Returns retry-after duration when exhausted

**Cleanup Strategy**: Removes sources inactive for >60 seconds to prevent memory growth

### Monitor File Watcher

**Purpose**: Detect changes to Claude Code JSONL session files
**Location**: `monitor/src/watcher.rs`
**Responsibility**: File system event detection and position tracking

**Key Components**:
- `FileWatcher` - Watches directory tree using `notify` crate
  - Monitors `~/.claude/projects/**/*.jsonl` files
  - Emits FileCreated, LinesAdded, FileRemoved events
  - Maintains position map to track last-read byte offset per file
  - Enables efficient tailing without re-reading content

**Key Methods**:
- `FileWatcher::new()` - Initialize watcher for directory
- `watch()` - Start watching and emit events to channel
- Position tracking via `RwLock<HashMap<PathBuf, u64>>`

**Design Pattern**: Notify-based recursive directory watching with file position caching

### Monitor JSONL Parser

**Purpose**: Extract structured events from Claude Code JSONL format
**Location**: `monitor/src/parser.rs`
**Responsibility**: Parse and normalize Claude Code events to VibeTea types

**Key Components**:
- `SessionParser` - Stateful parser that converts Claude Code events to VibeTea events
  - Extracts session ID from filename (UUID)
  - Extracts project name from file path (URL-decoded)
  - Tracks session start for first event

- `ParsedEventKind` - Normalized event types
  - ToolStarted { name, context }
  - ToolCompleted { name, success, context }
  - Activity
  - Summary
  - SessionStarted { project }

**Event Mapping**:
| Claude Code Type | VibeTea Event | Fields Extracted |
|------------------|---------------|------------------|
| `assistant` with `tool_use` | Tool started | tool name, context |
| `progress` with `PostToolUse` | Tool completed | tool name, success |
| `user` | Activity | timestamp only |
| `summary` | Summary | marks session end |
| First event in file | Session started | project from path |

**Privacy Strategy**: Extracts only metadata (tool names, timestamps, file basenames), never processes code content or prompts

### Monitor Privacy Pipeline

**Purpose**: Ensure no sensitive data (source code, file paths, prompts, commands) is transmitted to the server
**Location**: `monitor/src/privacy.rs`
**Responsibility**: Multi-stage data sanitization before event transmission

**Phase 5 New - Privacy Pipeline Pattern**:

The privacy module implements a **defense-in-depth sanitization pipeline** with multiple stages:

**Stage 1: Configuration** (`PrivacyConfig`)
- Loads allowlist from `VIBETEA_BASENAME_ALLOWLIST` environment variable
- Supports extension filtering (e.g., `.rs,.ts,.md` to allow only those files)
- All-or-nothing filtering: if allowlist is set, only matching extensions pass through
- Trims whitespace, auto-adds dots to extensions, filters empty entries

**Stage 2: Sensitive Tool Detection** (constant `SENSITIVE_TOOLS`)
- Bash: Shell commands may contain API keys, passwords, secrets
- Grep: Search patterns reveal user intent
- Glob: File patterns reveal project structure
- WebSearch, WebFetch: URLs and queries contain sensitive information
- These tools always have context stripped to `None`

**Stage 3: Path Sanitization** (`extract_basename()`)
- Converts full paths like `/home/user/project/src/auth.ts` → `auth.ts`
- Handles Unix absolute/relative paths, Windows paths, already-basenames
- Cross-platform using `std::path::Path`
- Returns `None` for invalid paths (empty, root-only)

**Stage 4: Context Processing** (`process_tool_context()`)
- Sensitive tools: context → None
- Other tools: extract basename, apply allowlist, transmit only if extension matches
- Non-matching extensions get context set to None (file not transmitted)

**Stage 5: Payload Transformation** (`process()`)
- Session events: pass through (project already sanitized at parse time)
- Activity events: pass through unchanged
- Tool events: context processed per stage 4
- Agent events: pass through unchanged
- Summary events: text replaced with "Session ended"
- Error events: pass through (category already sanitized)

**Key Types**:
- `PrivacyConfig` - Controls extension allowlist configuration
- `PrivacyPipeline` - Main processor applying all transformations
- `extract_basename()` - Utility for path-to-basename conversion

**Configuration Variables**:
| Variable | Purpose | Default |
|----------|---------|---------|
| `VIBETEA_BASENAME_ALLOWLIST` | Comma-separated file extensions to allow (e.g., `.rs,.ts`) | None (allow all) |

**Privacy Guarantees**:
- ✓ No full file paths in Tool events (only basenames)
- ✓ No file contents or diffs
- ✓ No user prompts or assistant responses
- ✓ No actual Bash commands (only description field)
- ✓ No Grep/Glob search patterns
- ✓ No WebSearch/WebFetch URLs or queries
- ✓ Summary text replaced with neutral message
- ✓ Extension allowlist prevents restricted file types from leaving the monitor

### Monitor Cryptographic Module

**Purpose**: Manage Ed25519 keypairs for signing events sent to the server
**Location**: `monitor/src/crypto.rs`
**Responsibility**: Key generation, storage, retrieval, and event signing

**Phase 6 New - Crypto Module Pattern**:

The crypto module handles Ed25519 operations with security-first design:

**Key Management**:
- `Crypto::generate()` - Generate new Ed25519 keypair using OS RNG
- `Crypto::load(dir)` - Load keypair from `{dir}/key.priv` (32-byte seed)
- `Crypto::save(dir)` - Save keypair with correct permissions:
  - `key.priv`: 0600 (owner read/write only) - Raw 32-byte seed
  - `key.pub`: 0644 (public) - Base64-encoded public key
- `Crypto::exists(dir)` - Check if keypair exists

**Signing Operations**:
- `crypto.sign(&[u8])` - Sign message, return base64 signature for HTTP headers
- `crypto.sign_raw(&[u8])` - Sign message, return raw 64-byte signature
- `crypto.public_key_base64()` - Get base64-encoded public key for server registration
- `crypto.verifying_key()` - Get ed25519_dalek VerifyingKey

**Security Features**:
- File permissions enforce private key confidentiality (0600)
- Deterministic Ed25519 signing (same message = same signature)
- Public key encoding matches server's expected format
- Errors distinguish between IO, invalid key format, and base64 issues

**Key Types**:
- `Crypto` - Main struct holding SigningKey
- `CryptoError` - Enum for Io, InvalidKey, Base64, KeyExists errors

**Example Usage**:
```rust
// Generate a new keypair and save
let crypto = Crypto::generate();
crypto.save(Path::new("/home/user/.vibetea")).unwrap();

// Load existing keypair
let crypto = Crypto::load(Path::new("/home/user/.vibetea")).unwrap();

// Sign an event batch and get base64 signature for X-Signature header
let signature = crypto.sign(json_bytes);
```

### Monitor Sender Module

**Purpose**: Send privacy-filtered events to the VibeTea server with resilient transmission
**Location**: `monitor/src/sender.rs`
**Responsibility**: HTTP request handling, event buffering, retry logic, and graceful shutdown

**Phase 6 New - Sender Module Pattern**:

The sender module implements reliable event transmission with recovery:

**Event Buffering**:
- `VecDeque<Event>` buffer with configurable max capacity (default 1000)
- `queue(event)` - Add event, evict oldest if full, return count of evictions
- FIFO eviction policy when buffer overflow occurs
- `buffer_len()`, `is_empty()` - Query buffer state

**HTTP Transmission**:
- Connection pooling via reqwest (10 idle connections max)
- Batching: `send()` sends single event, `send_batch()` sends array
- Headers:
  - `Content-Type: application/json`
  - `X-Source-Id: {source_id}` - Monitor identifier
  - `X-Signature: {base64_signature}` - Ed25519 signature of body
- Endpoint: `POST {server_url}/events`

**Retry Strategy**:
- Initial delay: 1 second
- Max delay: 60 seconds
- Jitter: ±25% random variance
- Max attempts: 10
- Retry on: connection errors, timeouts, 5xx server errors, 429 rate limits
- Stop on: 401 auth failed, 4xx client errors
- Parse `Retry-After` header for 429 responses

**Rate Limit Handling**:
- Detect 429 Too Many Requests
- Extract `Retry-After` header (seconds)
- Sleep for specified duration before retry
- Fall back to current exponential backoff if header missing

**Graceful Shutdown**:
- `shutdown(timeout)` - Attempt to flush remaining events
- Timeout (default 5s) prevents indefinite hang
- Returns count of unflushed events
- Logs errors if flush fails or times out

**Key Types**:
- `SenderConfig` - Configuration struct with server URL, source ID, buffer size
- `Sender` - Main struct with buffer, crypto, HTTP client, retry state
- `SenderError` - Enum for Http, ServerError, AuthFailed, RateLimited, BufferOverflow, MaxRetriesExceeded, Json

**Configuration Variables**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | Server URL for event submission | Yes | None |
| `VIBETEA_SOURCE_ID` | Monitor identifier (must match server registration) | No | Hostname |
| `VIBETEA_BUFFER_SIZE` | Event buffer capacity before eviction | No | 1000 |

**Example Usage**:
```rust
let crypto = Crypto::load(Path::new("/home/user/.vibetea")).unwrap();
let config = SenderConfig::new(
    "https://vibetea.fly.dev".to_string(),
    "my-monitor".to_string(),
    1000,
);
let mut sender = Sender::new(config, crypto);

// Queue events
sender.queue(event1);
sender.queue(event2);

// Flush when ready
sender.flush().await.unwrap();

// Graceful shutdown
let unflushed = sender.shutdown(Duration::from_secs(5)).await;
```

### Monitor CLI Component

**Purpose**: Provide command-line interface for keypair generation and daemon execution
**Location**: `monitor/src/main.rs`
**Responsibility**: CLI parsing, key initialization, and monitor bootstrap

**Phase 6 New - CLI Implementation**:

The monitor binary provides two main commands:

**Commands**:
- `vibetea-monitor init` - Generate Ed25519 keypair interactively
  - Checks for existing keys at `~/.vibetea/key.priv`
  - Prompts to overwrite if keys exist (unless `--force/-f` flag)
  - Saves keys with correct permissions (0600/0644)
  - Displays public key for server registration
  - Shows example VIBETEA_PUBLIC_KEYS export command

- `vibetea-monitor run` - Start the monitor daemon
  - Loads configuration from environment variables
  - Loads Ed25519 keypair from disk
  - Initializes file watcher and event parser
  - Creates sender with buffering and retry logic
  - Waits for SIGINT/SIGTERM signals
  - Gracefully shuts down with event flush timeout (5s)

**Help Commands**:
- `vibetea-monitor help` - Show help message
- `vibetea-monitor --help` / `-h` - Show help message
- `vibetea-monitor version` - Show version
- `vibetea-monitor --version` / `-V` - Show version

**Async Runtime**:
- `init` and `help`/`version` run synchronously
- `run` command creates multi-threaded tokio runtime
- Async operations: watcher, parser, sender, signal handling

**Key Functions**:
- `parse_args()` - Parse command line arguments into Command enum
- `run_init(force)` - Generate and save keypair with interactive prompt
- `run_monitor()` - Bootstrap and run the daemon
- `wait_for_shutdown()` - Handle SIGINT/SIGTERM signals
- `init_logging()` - Setup tracing with EnvFilter
- `get_key_directory()` - Resolve key path (VIBETEA_KEY_PATH or ~/.vibetea)

**Signal Handling**:
- SIGINT (Ctrl+C) - Graceful shutdown
- SIGTERM - Graceful shutdown
- Uses tokio::select! to wait for either signal

**Logging**:
- Tracing framework with adjustable log levels
- Default level: info
- Respects RUST_LOG environment variable
- Includes target and level in output

**Configuration Flow** (run command):
1. Parse command line arguments
2. Load Config from environment (VIBETEA_SERVER_URL required)
3. Load Crypto keys from disk (fails if not initialized)
4. Create Sender with buffering
5. TODO: Initialize FileWatcher and SessionParser
6. Wait for shutdown signal
7. Attempt graceful flush with 5s timeout

**Example Usage**:
```bash
# Generate keypair and register with server
vibetea-monitor init
# Output: Shows public key to register

# Start the monitor (requires VIBETEA_SERVER_URL)
export VIBETEA_SERVER_URL=https://vibetea.fly.dev
vibetea-monitor run

# Show help
vibetea-monitor help
```

### Monitor Component

**Purpose**: Captures Claude Code session activity with privacy guarantees and transmits to server
**Location**: `monitor/src/`
**Technologies**: Rust, tokio, file watching, JSONL parsing, Ed25519 cryptography, privacy pipeline, HTTP client

**Module Hierarchy**:
```
monitor/src/
├── main.rs       - CLI entry point with init/run commands (Phase 6)
├── lib.rs        - Public API exports
├── config.rs     - Environment variable parsing
├── types.rs      - Event definitions
├── error.rs      - Error hierarchy
├── watcher.rs    - File system watching (Phase 4)
├── parser.rs     - JSONL parsing (Phase 4)
├── privacy.rs    - Privacy pipeline (Phase 5)
├── crypto.rs     - Keypair management and signing (Phase 6 NEW)
└── sender.rs     - HTTP client with buffering/retry (Phase 6 NEW)
```

**Key Features (Phase 6)**:
- CLI with `init` (keypair generation) and `run` (daemon) subcommands
- File system watching for `.jsonl` files
- Incremental parsing with position tracking
- Claude Code event format normalization
- Privacy pipeline with multi-stage sanitization
- Extension allowlist filtering
- Sensitive tool detection and context stripping
- Ed25519 keypair generation, storage, and event signing (Phase 6)
- HTTP event transmission with buffering and exponential backoff retry (Phase 6)
- Graceful shutdown with event flush timeout (Phase 6)

### Client Component

**Purpose**: Subscribes to server events, displays sessions and activities
**Location**: `client/src/`
**Technologies**: TypeScript, React, Zustand, Vite

**Key Modules**:
- `types/events.ts` - TypeScript definitions matching Rust types with type guards
- `hooks/useEventStore.ts` - Zustand store for event state management
- `App.tsx` - Root component (Phase 3 placeholder)
- `main.tsx` - React entry point

## Data Flow

### Monitor → Server Flow (Phase 6)

```
Claude Code Session Activity
         ↓
    File System (JSONL files)
         ↓
    FileWatcher (notify crate)
         ↓
    WatchEvent (FileCreated/LinesAdded)
         ↓
    SessionParser
         ↓
    ParsedEvent (normalized)
         ↓
    VibeTea Event Construction
         ↓
    PrivacyPipeline Processing
         ↓
    Sanitized Event Payload
         ↓
    Sender.queue(event)
         ↓
    Event Buffer (VecDeque, 1000 max)
         ↓
    Sign with Ed25519 Private Key (Crypto::sign)
         ↓
    Batch and Buffer (FIFO, oldest evicted on overflow)
         ↓
    Sender.flush() or timer trigger
         ↓
    HTTPS POST to /events with headers:
      - X-Source-ID: {source_id}
      - X-Signature: {base64_signature}
      - Content-Type: application/json
         ↓
    Server Route Handler
         ↓
    Verify Signature (Auth Module)
         ↓
    Check Rate Limit (RateLimiter)
         ↓
    Validate Event Schema
         ↓
    Broadcast via WebSocket (EventBroadcaster)
```

**Flow Steps**:
1. FileWatcher detects changes to JSONL files in `~/.claude/projects/`
2. WatchEvent is emitted (FileCreated or LinesAdded with new content)
3. SessionParser reads new lines from tracked position
4. Parser extracts Claude Code events and converts to normalized ParsedEvent
5. ParsedEvent is converted to VibeTea Event with session ID, timestamp, source
6. PrivacyPipeline processes event payload:
   - Strips context from sensitive tools (Bash, Grep, Glob, WebSearch, WebFetch)
   - Converts full paths to basenames
   - Applies extension allowlist filtering
   - Replaces summary text with neutral message
7. Sender queues event in buffer (max 1000, FIFO eviction)
8. When flush triggered (manual or timer), events are signed and batched
9. Ed25519 signature created from JSON batch using private key (Crypto::sign)
10. Signed batch sent to server's `/events` endpoint via HTTPS POST
11. X-Signature header contains base64-encoded signature
12. X-Source-ID header contains monitor source ID
13. Server route handler extracts source ID from X-Source-ID header
14. Auth module verifies signature using configured public key
15. Rate limiter checks request allowance for source
16. Event schema is validated
17. EventBroadcaster immediately forwards to all subscribed WebSocket clients
18. On 429: parse Retry-After header, exponential backoff with jitter
19. On failure: retry up to 10 times, max 60s delay between attempts

### Server → Client Flow

```
Authenticated Event
        ↓
   Route Handler
        ↓
   EventBroadcaster
        ↓
   WebSocket Handler
        ↓
   SubscriberFilter
        ↓
   Client (TypeScript)
        ↓
   Zustand Store Update
        ↓
   Session Aggregation
        ↓
   Component Re-render
        ↓
   Display in UI
```

**Flow Steps**:
1. Route handler accepts and validates events
2. EventBroadcaster sends event to all active WebSocket subscriptions
3. WebSocket handler forwards event to client
4. Client's event listener receives WebSocket message
5. SubscriberFilter matches event against connection criteria
6. Event is added to Zustand store
7. Store performs session aggregation
8. Components subscribed to store state re-render
9. UI displays updated sessions and activities

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|----------|---------------|
| **Server Routes** | HTTP request handling, response formatting, WebSocket upgrade | Config, Auth, Broadcast, RateLimit | Database, external services |
| **Auth Module** | Signature verification, token validation | Config (public keys, tokens) | Routes, broadcast |
| **Broadcast Module** | Event distribution, subscriber filtering | Types (event schema) | Routes, auth, config |
| **RateLimit Module** | Token bucket tracking, stale entry cleanup | None (self-contained) | Routes, auth, broadcast |
| **Monitor CLI** | Command parsing, user interaction, daemon bootstrap | Config, Crypto, Sender, Watcher, Parser, Privacy | WebSocket, direct server access |
| **Monitor File Watcher** | File system observation, position tracking | Filesystem | Parser directly (events via channel) |
| **Monitor Parser** | JSONL parsing, event normalization | Types | Filesystem directly |
| **Monitor Privacy Pipeline** | Event payload sanitization | Types (EventPayload) | Filesystem, auth, server communication |
| **Monitor Cryptographic** | Keypair management, message signing | types (public key format) | Sender (crypto is stateless) |
| **Monitor Sender** | HTTP transmission, buffering, retry logic | Config (server URL), Crypto (signing), Types (events) | Filesystem, watcher, parser directly |
| **Client Component** | Display UI, handle user actions | Store hooks, types | WebSocket layer directly |

## Dependency Rules

- **No circular dependencies**: Routes → (Auth, Broadcast, RateLimit), Auth → Config, Broadcast → Types, RateLimit → (no deps)
- **Auth is read-only**: Auth module never modifies config, only reads public keys and tokens
- **Rate limiting is stateless to requests**: Each request gets its own check, no persistent memory beyond buckets
- **Broadcast is fire-and-forget**: Events are sent but no confirmation/acknowledgment required
- **Type safety**: All three languages use strong typing; event schema is enforced at compile time
- **Asymmetric auth**: Monitors authenticate with cryptographic signatures; clients authenticate with bearer tokens
- **File watcher isolation**: Watcher and parser communicate via channels, no direct file access from parser
- **Privacy pipeline is mandatory**: All events must be processed through privacy pipeline before transmission
- **Privacy is immutable**: PrivacyConfig and PrivacyPipeline are immutable after creation
- **Crypto is stateless**: Crypto module has no mutable state, pure signing operations
- **Sender owns buffering**: Only sender manages event queue, other modules queue through sender interface
- **CLI bootstraps all**: main.rs coordinates config, crypto, sender, watcher, parser initialization

## Key Interfaces & Contracts

### Server Routes → Auth Contract

```rust
// Route handler calls auth to verify signature
verify_signature(
    source_id: &str,           // From X-Source-ID header
    signature_base64: &str,    // From X-Signature header
    message: &[u8],            // Request body bytes
    public_keys: &HashMap<String, String>, // From config
) -> Result<(), AuthError>
```

### Server Routes → Broadcast Contract

```rust
// Route handler sends event to broadcaster
broadcaster.broadcast(event: Event) -> usize  // Returns subscriber count

// WebSocket handler subscribes
let mut rx = broadcaster.subscribe() // Returns Receiver<Event>
```

### Server Routes → RateLimit Contract

```rust
// Route handler checks rate limit
rate_limiter.check_rate_limit(source_id: &str).await -> RateLimitResult
// Returns Allowed or Limited { retry_after_secs }
```

### Monitor FileWatcher → Channel Contract

```rust
// Watcher sends events through channel
pub enum WatchEvent {
    FileCreated(PathBuf),
    LinesAdded { path: PathBuf, lines: Vec<String> },
    FileRemoved(PathBuf),
}
```

### Monitor Parser Contract

```rust
// Parser converts Claude Code lines to VibeTea events
pub struct ParsedEvent {
    pub kind: ParsedEventKind,
    pub timestamp: DateTime<Utc>,
}

pub enum ParsedEventKind {
    ToolStarted { name: String, context: Option<String> },
    ToolCompleted { name: String, success: bool, context: Option<String> },
    Activity,
    Summary,
    SessionStarted { project: String },
}
```

### Monitor Privacy Pipeline Contract

```rust
// Privacy pipeline processes event payloads
pub struct PrivacyPipeline {
    config: PrivacyConfig,
}

impl PrivacyPipeline {
    pub fn new(config: PrivacyConfig) -> Self { ... }
    pub fn process(&self, payload: EventPayload) -> EventPayload { ... }
}

pub struct PrivacyConfig {
    basename_allowlist: Option<HashSet<String>>,
}

impl PrivacyConfig {
    pub fn new(basename_allowlist: Option<HashSet<String>>) -> Self { ... }
    pub fn from_env() -> Self { ... }
    pub fn is_extension_allowed(&self, basename: &str) -> bool { ... }
}

// Path sanitization utility
pub fn extract_basename(path: &str) -> Option<String> { ... }
```

### Monitor Crypto Contract

```rust
// Crypto module for keypair and signing operations
pub struct Crypto {
    signing_key: SigningKey,
}

impl Crypto {
    pub fn generate() -> Self { ... }
    pub fn load(dir: &Path) -> Result<Self, CryptoError> { ... }
    pub fn save(&self, dir: &Path) -> Result<(), CryptoError> { ... }
    pub fn exists(dir: &Path) -> bool { ... }
    pub fn sign(&self, message: &[u8]) -> String { ... }  // Base64
    pub fn sign_raw(&self, message: &[u8]) -> [u8; 64] { ... }  // Raw bytes
    pub fn public_key_base64(&self) -> String { ... }
    pub fn verifying_key(&self) -> VerifyingKey { ... }
}

pub enum CryptoError {
    Io(std::io::Error),
    InvalidKey(String),
    Base64(base64::DecodeError),
    KeyExists(String),
}
```

### Monitor Sender Contract

```rust
// Sender module for event transmission with buffering and retry
pub struct Sender {
    config: SenderConfig,
    crypto: Crypto,
    client: Client,
    buffer: VecDeque<Event>,
    current_retry_delay: Duration,
}

impl Sender {
    pub fn new(config: SenderConfig, crypto: Crypto) -> Self { ... }
    pub fn queue(&mut self, event: Event) -> usize { ... }  // Returns evicted count
    pub fn buffer_len(&self) -> usize { ... }
    pub fn is_empty(&self) -> bool { ... }
    pub async fn send(&mut self, event: Event) -> Result<(), SenderError> { ... }
    pub async fn flush(&mut self) -> Result<(), SenderError> { ... }
    pub async fn shutdown(&mut self, timeout: Duration) -> usize { ... }  // Returns unflushed count
}

pub struct SenderConfig {
    pub server_url: String,
    pub source_id: String,
    pub buffer_size: usize,
}

pub enum SenderError {
    Http(reqwest::Error),
    ServerError { status: u16, message: String },
    AuthFailed,
    RateLimited { retry_after_secs: u64 },
    BufferOverflow { evicted_count: usize },
    MaxRetriesExceeded { attempts: u32 },
    Json(serde_json::Error),
}
```

### Monitor ↔ Server HTTP Contract

**Endpoint**: `POST /events`
**Required Headers**:
- `X-Source-ID` - Monitor identifier
- `X-Signature` - Base64-encoded Ed25519 signature of request body

**Response Codes**:
- `202 Accepted` - Events received and queued for broadcast
- `400 Bad Request` - Invalid event format
- `401 Unauthorized` - Missing/invalid source ID or signature
- `429 Too Many Requests` - Rate limit exceeded (includes `Retry-After` header)

### Server ↔ Client WebSocket Contract

**Endpoint**: `GET /ws`
**Query Parameters** (optional):
- `token` - Authentication token
- `source` - Filter by source ID
- `type` - Filter by event type
- `project` - Filter by project name

**Response Codes**:
- `101 Switching Protocols` - WebSocket upgrade successful
- `401 Unauthorized` - Invalid or missing token

## State Management

| State Type | Location | Pattern | Scope |
|-----------|----------|---------|-------|
| **Request Validation** | Routes layer | Immediate rejection on invalid format | Single request |
| **Rate Limit State** | RateLimiter (token buckets) | Per-source token tracking | All requests for source |
| **Event Buffer** | EventBroadcaster (broadcast channel) | FIFO with capacity 1000 | All subscribed clients |
| **File Positions** | FileWatcher (RwLock HashMap) | Position map per file | Monitor session lifetime |
| **Subscriber Filters** | Per WebSocket connection | Builder pattern applied at connect | Single connection lifetime |
| **Server Config** | AppState (Arc<Config>) | Immutable, loaded at startup | Server process lifetime |
| **Event Send Buffer** | Sender (VecDeque) | FIFO queue, max 1000, oldest evicted | Monitor session lifetime |
| **Retry State** | Sender (current_retry_delay) | Exponential backoff, reset on success | Sender instance lifetime |
| **Keypair** | Crypto (SigningKey) | Immutable after creation | Monitor process lifetime |
| **Client State** | Zustand store | Event buffer + session derivation | Client session lifetime |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Error Handling** | Result types with ErrorResponse JSON | `routes.rs` (error_to_response pattern) |
| **Authentication** | Ed25519 for monitors, bearer tokens for clients | `auth.rs` |
| **Rate Limiting** | Token bucket per source | `rate_limit.rs` |
| **Logging** | Structured JSON logging with tracing | `main.rs` (init_logging) + route handlers |
| **Graceful Shutdown** | Signal handling (SIGTERM/SIGINT) with timeout | `main.rs` (shutdown_signal) |
| **Cleanup Tasks** | Background cleanup of stale rate limit entries | `main.rs` (spawn_cleanup_task) |
| **WebSocket Protocol** | Ping/pong handling, text messages, close frames | `routes.rs` (handle_websocket) |
| **File Watching** | Recursive directory monitoring, change detection | `watcher.rs` (notify-based) |
| **Event Parsing** | Claude Code JSONL normalization, privacy filtering | `parser.rs` |
| **Privacy Pipeline** | Multi-stage data sanitization before transmission | `privacy.rs` (Phase 5) |
| **Event Transmission** | HTTP POST with buffering and retry logic | `sender.rs` (Phase 6) |
| **Key Generation** | Ed25519 keypair generation and storage | `crypto.rs` (Phase 6) |
| **CLI Interface** | Command parsing and daemon bootstrapping | `main.rs` (Phase 6) |

## Testing Strategy

**Server Tests**: Located in `routes.rs` using axum test utilities
- Health endpoint tests (uptime reporting, subscriber counting)
- Event ingestion tests (single/batch, with/without auth)
- Authentication tests (valid signature, missing header, invalid signature, unknown source)
- Rate limiting tests (under limit, over limit, retry-after)
- WebSocket filter tests (source, event type, project filtering)
- AppState initialization tests

**Monitor Tests**: Located in `monitor/tests/privacy_test.rs` (Phase 5)
- Privacy pipeline validation tests
- Path-to-basename conversion tests
- Extension allowlist filtering tests
- Sensitive tool context stripping tests
- Summary text replacement tests
- Configuration parsing tests (environment variables)

**Monitor Crypto Tests**: Inline in `monitor/src/crypto.rs` (Phase 6)
- Keypair generation tests
- Save/load roundtrip tests
- Key existence checks
- Signature verification tests
- File permission tests (Unix)
- Base64 encoding tests

**Monitor Sender Tests**: Inline in `monitor/src/sender.rs` (Phase 6)
- Event queueing and eviction tests
- Retry delay calculation tests
- Jitter application tests
- Configuration tests
- Buffer state tests

**Integration Tests**: Located in `server/tests/unsafe_mode_test.rs`
- End-to-end scenarios in unsafe mode (auth disabled)

**Running Tests**:
```bash
cargo test --package vibetea-server  # All server tests
cargo test --package vibetea-server routes  # Route tests only
cargo test --package vibetea-monitor  # All monitor tests
cargo test --package vibetea-monitor crypto  # Crypto tests only
cargo test --package vibetea-monitor privacy  # Privacy tests only
cargo test --workspace --test-threads=1  # All tests with single thread (important for env vars)
```

## Graceful Shutdown Flow

```
Signal (SIGTERM/SIGINT)
         ↓
shutdown_signal() async (Monitor: wait_for_shutdown)
         ↓
Log shutdown initiation
         ↓
Sender.shutdown(timeout) - Attempt to flush remaining events
         ↓
Flush remaining buffer to server with timeout
         ↓
Abort cleanup task
         ↓
Allow in-flight requests to complete (5s timeout)
         ↓
Exit with success
```

**Shutdown Timeout**: 5 seconds for event buffer flush to complete
**Cleanup Interval**: Rate limiter cleans up every 30 seconds

---

*This document describes HOW the system is organized. Keep focus on patterns and relationships.*
