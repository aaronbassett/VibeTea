# Project Structure

**Status**: Phase 8 incremental update - Virtual scrolling event stream and formatting utilities
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Directory Layout

```
VibeTea/
├── server/                     # Rust HTTP server and event hub
│   ├── src/
│   │   ├── main.rs            # Server entry point with graceful shutdown (217 lines)
│   │   ├── lib.rs             # Public API exports
│   │   ├── config.rs          # Environment variable configuration (425 lines)
│   │   ├── error.rs           # Error types and conversions (467 lines)
│   │   ├── types.rs           # Event definitions (401 lines)
│   │   ├── routes.rs          # HTTP routes and handlers (1124 lines)
│   │   ├── auth.rs            # Ed25519 verification and token validation (765 lines)
│   │   ├── broadcast.rs       # Event broadcasting with filtering (1040 lines)
│   │   ├── rate_limit.rs      # Token bucket rate limiting (718 lines)
│   │   └── Cargo.toml         # Rust dependencies
│   └── tests/
│       └── unsafe_mode_test.rs # Integration tests (Phase 3)
│
├── monitor/                    # Rust file watcher and event producer
│   ├── src/
│   │   ├── main.rs            # Monitor CLI with init/run commands (301 lines - Phase 6)
│   │   ├── lib.rs             # Public API exports (45 lines, updated Phase 6)
│   │   ├── config.rs          # Environment variable configuration (305 lines)
│   │   ├── error.rs           # Error types and conversions (173 lines)
│   │   ├── types.rs           # Event definitions (338 lines)
│   │   ├── watcher.rs         # File system watching with position tracking (944 lines - Phase 4)
│   │   ├── parser.rs          # JSONL parsing and event normalization (1098 lines - Phase 4)
│   │   ├── privacy.rs         # Privacy pipeline for event sanitization (1039 lines - Phase 5)
│   │   ├── crypto.rs          # Keypair management and event signing (438 lines - Phase 6 NEW)
│   │   ├── sender.rs          # HTTP client with buffering/retry (544 lines - Phase 6 NEW)
│   │   └── Cargo.toml         # Rust dependencies
│   └── tests/
│       └── privacy_test.rs     # Privacy compliance tests (Phase 5)
│
├── client/                     # TypeScript React web dashboard
│   ├── src/
│   │   ├── main.tsx           # React entry point (4 lines)
│   │   ├── App.tsx            # Root component (7 lines, placeholder)
│   │   ├── index.css          # Global styles
│   │   ├── types/
│   │   │   └── events.ts      # TypeScript event types (248 lines)
│   │   ├── hooks/
│   │   │   ├── useEventStore.ts      # Zustand event store (171 lines)
│   │   │   └── useWebSocket.ts       # WebSocket connection hook (321 lines - Phase 7 NEW)
│   │   ├── components/
│   │   │   ├── EventStream.tsx       # Virtual scrolling event list (425 lines - Phase 8 NEW)
│   │   │   ├── ConnectionStatus.tsx  # Connection status indicator (106 lines - Phase 7 NEW)
│   │   │   └── TokenForm.tsx         # Token input form (201 lines - Phase 7 NEW)
│   │   ├── utils/
│   │   │   └── formatting.ts         # Timestamp and duration formatting (331 lines - Phase 8 NEW)
│   │   └── __tests__/
│   │       └── events.test.ts        # Event type guard tests
│   ├── vite.config.ts         # Vite build configuration
│   ├── tsconfig.json          # TypeScript configuration
│   ├── package.json           # Dependencies and scripts
│   └── index.html             # HTML entry point
│
├── .sdd/
│   ├── codebase/
│   │   ├── STACK.md           # Technology stack
│   │   ├── INTEGRATIONS.md    # External services
│   │   ├── ARCHITECTURE.md    # System design patterns
│   │   ├── STRUCTURE.md       # This file
│   │   ├── CONVENTIONS.md     # Code style and naming
│   │   ├── TESTING.md         # Test strategy
│   │   ├── SECURITY.md        # Auth mechanisms
│   │   └── CONCERNS.md        # Tech debt and risks
│   └── memory/                # SDD memory files
│
├── specs/                      # Requirements and specifications
│   └── 001-vibetea/
│       ├── contracts/         # API contracts
│       ├── checklists/        # Task tracking
│       └── retro/             # Retrospectives
│
├── discovery/                  # Design notes and decisions
├── Cargo.toml                 # Workspace root (Rust monorepo)
├── Cargo.lock                 # Dependency lock file
└── target/                    # Rust build artifacts
```

## Key Directories

### `server/src/` - HTTP Server and Event Hub

| File | Purpose | Lines |
|------|---------|-------|
| `main.rs` | Server bootstrap with logging, signal handling, graceful shutdown | 217 |
| `lib.rs` | Module re-exports for public API | ~46 |
| `config.rs` | Environment variable parsing (VIBETEA_PUBLIC_KEYS, VIBETEA_SUBSCRIBER_TOKEN, PORT, VIBETEA_UNSAFE_NO_AUTH) | 425 |
| `error.rs` | Error type definitions and Display implementations | 467 |
| `types.rs` | Event struct and enum definitions with serde | 401 |
| `routes.rs` | HTTP route handlers (POST /events, GET /ws, GET /health) and AppState | 1124 |
| `auth.rs` | Ed25519 signature verification and token validation with constant-time comparison | 765 |
| `broadcast.rs` | Event broadcaster with multi-subscriber support and filtering | 1040 |
| `rate_limit.rs` | Per-source token bucket rate limiting with stale entry cleanup | 718 |

**Configuration Variables**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_PUBLIC_KEYS` | Monitor public keys for signature verification (format: `source1:key1,source2:key2`) | If unsafe mode off | None |
| `VIBETEA_SUBSCRIBER_TOKEN` | Bearer token for client WebSocket connections | If unsafe mode off | None |
| `PORT` | HTTP server port | No | 8080 |
| `VIBETEA_UNSAFE_NO_AUTH` | Disable all authentication (dev only, set to 'true') | No | false |
| `RUST_LOG` | Log level filter (default: info) | No | info,tower_http=debug,axum::rejection=trace |

**Key Types & Constants**:
- `AppState` - Shared state containing config, broadcaster, rate limiter, and start time
- `RateLimitResult` - Enum: Allowed or Limited with retry-after duration
- `RateLimitResult::Allowed` - Request is within limit
- `RateLimitResult::Limited { retry_after_secs }` - Request exceeded limit, include Retry-After header
- `RATE_LIMITER_CLEANUP_INTERVAL` - 30 seconds
- `GRACEFUL_SHUTDOWN_TIMEOUT` - 30 seconds

### `server/src/routes.rs` - HTTP Endpoints

| Handler | Endpoint | Purpose |
|---------|----------|---------|
| `post_events()` | POST /events | Accept events with Ed25519 signature verification |
| `get_ws()` | GET /ws | WebSocket upgrade with token validation |
| `handle_websocket()` | WS connection | Forward events to client with filtering |
| `get_health()` | GET /health | Health check with status, connections, uptime |

**Request/Response Examples**:

POST /events:
```json
// Request body (single or array)
{
  "id": "evt_k7m2n9p4q1r6s3t8u5v0",
  "source": "monitor-1",
  "timestamp": "2026-02-02T14:30:00Z",
  "event_type": "session",
  "payload": { ... }
}

// Response: 202 Accepted (empty body)
// or 401 Unauthorized / 429 Too Many Requests
```

GET /health:
```json
{
  "status": "ok",
  "connections": 42,
  "uptime_seconds": 3600
}
```

### `server/src/auth.rs` - Cryptographic Authentication

| Function | Purpose |
|----------|---------|
| `verify_signature()` | Ed25519 signature verification against request body |
| `validate_token()` | Constant-time bearer token comparison |

**Error Types**:
- `UnknownSource` - Source ID not in configured public keys
- `InvalidSignature` - Signature verification failed
- `InvalidBase64` - Base64 decoding failed for signature or public key
- `InvalidPublicKey` - Public key is malformed or wrong length
- `InvalidToken` - Token mismatch or empty

### `server/src/broadcast.rs` - Event Distribution

| Type | Purpose |
|------|---------|
| `EventBroadcaster` | Central hub for event distribution using tokio broadcast channel |
| `SubscriberFilter` - Optional filtering by source, event type, or project |

**Key Methods**:
- `EventBroadcaster::new()` - Create with default capacity (1000)
- `EventBroadcaster::with_capacity()` - Create with custom capacity
- `EventBroadcaster::broadcast()` - Send event to all subscribers
- `EventBroadcaster::subscribe()` - Get receiver for new connection
- `EventBroadcaster::subscriber_count()` - Get active connections
- `SubscriberFilter::matches()` - Check if event matches all criteria (AND logic)

### `server/src/rate_limit.rs` - Request Rate Limiting

| Type | Purpose |
|------|---------|
| `RateLimiter` | Thread-safe per-source rate limiting |
| `TokenBucket` - Per-source token bucket implementation |

**Constants**:
- `DEFAULT_RATE` - 100.0 tokens per second
- `DEFAULT_CAPACITY` - 100 tokens (burst size)
- `STALE_ENTRY_TIMEOUT` - 60 seconds (cleanup threshold)

**Key Methods**:
- `RateLimiter::new(rate, capacity)` - Create custom limiter
- `RateLimiter::default()` - Create with defaults
- `check_rate_limit(source_id)` - Check if request is allowed
- `cleanup_stale_entries()` - Remove inactive sources
- `spawn_cleanup_task(interval)` - Background cleanup every N seconds
- `source_count()` - Get number of tracked sources

### `monitor/src/` - File Watcher and Event Producer

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `main.rs` | Monitor CLI with init/run commands | 301 | Phase 6 NEW |
| `lib.rs` | Public API (exports modules) | 45 | Updated Phase 6 |
| `config.rs` | Environment variable parsing | 305 | Phase 3 |
| `error.rs` | Error types (Config, IO, JSON, HTTP, Crypto, Watch) | 173 | Phase 3 |
| `types.rs` | Event definitions with ID generation | 338 | Phase 3 |
| `watcher.rs` | File system watching with position tracking | 944 | Phase 4 |
| `parser.rs` | JSONL parsing and event normalization | 1098 | Phase 4 |
| `privacy.rs` | Privacy pipeline for event sanitization | 1039 | Phase 5 |
| `crypto.rs` | Keypair management and event signing | 438 | Phase 6 NEW |
| `sender.rs` | HTTP client with buffering/retry | 544 | Phase 6 NEW |

#### Monitor CLI - `main.rs` (Phase 6 New)

**Purpose**: Command-line interface for keypair generation and daemon execution

**Commands**:
- `vibetea-monitor init [--force/-f]` - Generate Ed25519 keypair
- `vibetea-monitor run` - Start the monitor daemon
- `vibetea-monitor help` - Show help message
- `vibetea-monitor version` - Show version

**Key Functions**:
- `parse_args()` - Parse command line arguments into Command enum
- `run_init(force)` - Generate and save keypair with interactive prompt
- `run_monitor()` - Bootstrap and run the daemon (async)
- `wait_for_shutdown()` - Handle SIGINT/SIGTERM signals
- `init_logging()` - Setup tracing with EnvFilter
- `get_key_directory()` - Resolve key path
- `get_default_source_id()` - Get hostname for default source ID

**Async Runtime**:
- `init`, `help`, `version` run synchronously
- `run` creates multi-threaded tokio runtime
- All async operations in `run` command

**Signal Handling**:
- SIGINT (Ctrl+C) and SIGTERM both trigger graceful shutdown
- Uses tokio::select! for cross-platform signal waiting

**Logging**:
- Tracing framework with EnvFilter
- Default level: info
- Respects RUST_LOG environment variable

#### Monitor Cryptographic Module - `crypto.rs` (Phase 6 New)

**Purpose**: Manage Ed25519 keypairs for signing events

**File Size**: 438 lines including test cases

**Key Types**:
- `Crypto` - Main struct holding SigningKey
- `CryptoError` - Enum for Io, InvalidKey, Base64, KeyExists errors

**Key Methods**:
- `Crypto::generate()` - Generate new Ed25519 keypair
- `Crypto::load(dir)` - Load from {dir}/key.priv (32-byte seed)
- `Crypto::save(dir)` - Save with permissions (0600/0644)
- `Crypto::exists(dir)` - Check if keypair exists
- `crypto.sign(&[u8])` - Sign message, return base64
- `crypto.sign_raw(&[u8])` - Sign message, return 64 raw bytes
- `crypto.public_key_base64()` - Get base64-encoded public key
- `crypto.verifying_key()` - Get ed25519_dalek VerifyingKey

**Key Files**:
- `key.priv` - Raw 32-byte Ed25519 seed (permissions 0600)
- `key.pub` - Base64-encoded public key with newline (permissions 0644)

**Test Coverage**:
- Keypair generation and validity
- Save/load roundtrip verification
- Existence checking
- Signature generation and verification
- File permission verification (Unix)
- Base64 encoding/decoding
- Error handling for invalid keys

#### Monitor Sender Module - `sender.rs` (Phase 6 New)

**Purpose**: HTTP event transmission with buffering and resilience

**File Size**: 544 lines including test cases

**Key Types**:
- `Sender` - Main struct with buffer, crypto, client, retry state
- `SenderConfig` - Configuration struct
- `SenderError` - Enum for error types

**Key Methods**:
- `Sender::new(config, crypto)` - Create sender with HTTP client
- `sender.queue(event)` - Add event to buffer, evict oldest if full
- `sender.buffer_len()` - Get buffered event count
- `sender.is_empty()` - Check if buffer empty
- `sender.send(event)` - Send single event immediately
- `sender.flush()` - Flush all buffered events
- `sender.shutdown(timeout)` - Graceful shutdown with flush

**Event Buffering**:
- VecDeque with configurable max capacity (default 1000)
- FIFO eviction when full
- `queue()` returns count of evicted events

**HTTP Transmission**:
- Connection pooling (10 idle max)
- Batching: single or array of events
- Headers:
  - `Content-Type: application/json`
  - `X-Source-Id: {source_id}`
  - `X-Signature: {base64_signature}`
- Endpoint: `POST {server_url}/events`

**Retry Strategy**:
- Initial delay: 1 second
- Max delay: 60 seconds
- Jitter: ±25% random
- Max attempts: 10
- Retry on: connection errors, timeouts, 5xx, 429
- Stop on: 401 auth failed, 4xx client errors
- Parse Retry-After header for 429 responses

**Test Coverage**:
- Event queueing and buffer overflow
- Retry delay calculations
- Jitter application bounds
- Configuration initialization
- Buffer state queries

**Configuration Variables**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | Server URL for event submission | Yes | None |
| `VIBETEA_SOURCE_ID` | Monitor identifier | No | Hostname |
| `VIBETEA_BUFFER_SIZE` | Event buffer capacity | No | 1000 |

#### Monitor File Watcher - `watcher.rs` (Phase 4)

**Purpose**: Monitor JSONL files in `~/.claude/projects/` for changes

**Key Types**:
- `FileWatcher` - Main watcher using `notify` crate
- `WatchEvent` - Events emitted (FileCreated, LinesAdded, FileRemoved)
- `WatcherError` - Error types

**Key Features**:
- Recursive directory monitoring for `.jsonl` files
- Position tracking with `RwLock<HashMap<PathBuf, u64>>`
- Incremental file reading (only new lines)
- Channel-based event emission

**Dependencies**:
- `notify` crate - File system event detection
- `tokio` - Async runtime and synchronization primitives
- `tracing` - Structured logging

#### Monitor JSONL Parser - `parser.rs` (Phase 4)

**Purpose**: Parse Claude Code JSONL format and normalize to VibeTea events

**Key Types**:
- `SessionParser` - Stateful parser with session tracking
- `ParsedEvent` - Normalized event with kind and timestamp
- `ParsedEventKind` - 5 event types (ToolStarted, ToolCompleted, Activity, Summary, SessionStarted)
- `RawClaudeEvent` - Raw deserialization struct

**Event Mapping**:
| Claude Type | Parsed Event | Fields |
|------------|--------------|--------|
| assistant + tool_use | ToolStarted | name, context |
| progress + PostToolUse | ToolCompleted | name, success, context |
| user | Activity | (timestamp only) |
| summary | Summary | (marks end) |
| First event | SessionStarted | project (from path) |

**Privacy Strategy**: Only extracts metadata (tool names, timestamps, file basenames)

**Key Methods**:
- `SessionParser::from_path()` - Create parser from file path
- `SessionParser::parse_line()` - Parse single JSONL line
- Extracts UUID from filename as session ID
- URL-decodes project name from path

#### Monitor Privacy Pipeline - `privacy.rs` (Phase 5 New)

**Purpose**: Sanitize event payloads before transmission to ensure no sensitive data leaves the monitor

**File Size**: 1039 lines including tests

**Key Types**:
- `PrivacyConfig` - Configuration for extension allowlist
- `PrivacyPipeline` - Main processor for event payload sanitization
- `extract_basename()` - Utility function for path-to-basename conversion

**Key Methods**:
- `PrivacyConfig::new()` - Create with optional extension allowlist
- `PrivacyConfig::from_env()` - Load allowlist from VIBETEA_BASENAME_ALLOWLIST env var
- `PrivacyConfig::is_extension_allowed()` - Check if basename extension is allowed
- `PrivacyPipeline::new()` - Create pipeline with config
- `PrivacyPipeline::process()` - Process event payload through sanitization stages
- `extract_basename()` - Convert full path to basename (Unix/Windows compatible)

**Sensitive Tools** (context always stripped):
- `Bash` - Commands may contain secrets
- `Grep` - Patterns reveal user intent
- `Glob` - Patterns reveal project structure
- `WebSearch` - Queries reveal intent
- `WebFetch` - URLs may contain sensitive information

**Configuration Variables**:
| Variable | Purpose | Default |
|----------|---------|---------|
| `VIBETEA_BASENAME_ALLOWLIST` | Comma-separated file extensions (e.g., `.rs,.ts,.md`) | None (allow all) |

**Privacy Guarantees**:
- No full file paths (only basenames)
- No file contents or diffs
- No user prompts or assistant responses
- No actual Bash commands
- No Grep/Glob search patterns
- No WebSearch/WebFetch URLs
- Summary text replaced with "Session ended"
- Extension allowlist filtering prevents restricted file types

**Example Usage**:
```rust
// Allow only Rust and TypeScript files
let config = PrivacyConfig::from_env();  // Reads VIBETEA_BASENAME_ALLOWLIST
let pipeline = PrivacyPipeline::new(config);

// Process event before transmission
let sanitized = pipeline.process(event);
```

**Test Coverage** (Privacy Test Suite - 951 lines):
- Configuration parsing tests
- Extension allowlist filtering tests
- Path-to-basename conversion tests
- Sensitive tool context stripping tests
- Summary text replacement tests
- All event type payload transformations
- Cross-platform path handling
- Environment variable parsing edge cases

**Configuration Variables**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | VibeTea server URL (e.g., `https://vibetea.fly.dev`) | Yes | None |
| `VIBETEA_SOURCE_ID` | Monitor identifier | No | System hostname |
| `VIBETEA_KEY_PATH` | Directory containing `key.priv` and `key.pub` | No | `~/.vibetea` |
| `VIBETEA_CLAUDE_DIR` | Claude Code directory to watch | No | `~/.claude` |
| `VIBETEA_BUFFER_SIZE` | Event buffer capacity before flush | No | 1000 |
| `VIBETEA_BASENAME_ALLOWLIST` | Comma-separated file patterns to allow | No | All files |

### `client/src/` - React TypeScript Dashboard

| File | Purpose | Lines | Status |
|------|---------|-------|--------|
| `main.tsx` | React DOM render entry point | 4 | Phase 3 |
| `App.tsx` | Root component (placeholder) | 7 | Phase 3 |
| `index.css` | Global styles | ~50 | Phase 3 |
| `types/events.ts` | TypeScript event type definitions with type guards | 248 | Phase 3 |
| `hooks/useEventStore.ts` | Zustand store for event state and session management | 171 | Phase 7 |
| `hooks/useWebSocket.ts` | WebSocket connection hook with auto-reconnect | 321 | Phase 7 |
| `components/EventStream.tsx` | Virtual scrolling event list with auto-scroll | 425 | Phase 8 NEW |
| `components/ConnectionStatus.tsx` | Connection status visual indicator | 106 | Phase 7 |
| `components/TokenForm.tsx` | Token input and persistence form | 201 | Phase 7 |
| `utils/formatting.ts` | Timestamp and duration formatting utilities | 331 | Phase 8 NEW |
| `__tests__/events.test.ts` | Event type guard tests | | Phase 3 |

**Dependencies**:
- React 19.2.4 - UI framework
- TypeScript 5.x - Type safety
- Zustand 5.0.11 - State management
- @tanstack/react-virtual 3.13.18 - Virtual scrolling (Phase 8 NEW)
- Vite 7.3.1 - Build tool and dev server

#### Client Virtual Scrolling - `components/EventStream.tsx` (Phase 8 New)

**Purpose**: Efficiently render large event streams with auto-scroll and jump-to-latest

**File Size**: 425 lines including helper components

**Key Sub-components**:
- `EventRow` - Single event row with icon, badge, description, timestamp
- `JumpToLatestButton` - Button to jump to latest events with pending count
- `EmptyState` - Placeholder when no events available

**Key Functions**:
- `formatTimestamp(timestamp)` - Format RFC 3339 to HH:MM:SS
- `getEventDescription(event)` - Create human-readable event summary
- `handleScroll()` - Detect user scroll position for auto-scroll control
- `handleJumpToLatest()` - Jump to bottom and reset auto-scroll

**Virtual Scrolling Features**:
- Uses @tanstack/react-virtual for efficient rendering
- Estimated row height: 64px
- Overscan: 5 items for smooth scrolling
- Only renders visible items + overscan buffer

**Auto-Scroll Behavior**:
- Automatically scrolls to bottom when new events arrive
- Disables auto-scroll when user scrolls up 50px+ from bottom
- "Jump to Latest" button appears when auto-scroll is disabled
- Shows count of pending new events on button
- Re-enables auto-scroll when user clicks jump button

**Event Display**:
- Event type icons (🔧 tool, 💬 activity, 🚀 session, 📋 summary, ⚠️ error, 🤖 agent)
- Color-coded type badges (blue/green/purple/cyan/red/amber)
- Formatted timestamp (HH:MM:SS)
- Source and session ID (first 8 chars)
- Event description with payload-specific details

**Props**:
- `className` (string, optional) - Additional CSS classes for container

**Accessibility**:
- `role="log"` with `aria-live="polite"`
- `role="list"` on scrollable container
- `role="listitem"` on each event row
- Proper semantic HTML with `<time>` elements
- ARIA labels for screen readers

**Example Usage**:
```tsx
// Basic usage with full height
<EventStream className="h-full" />

// Custom sizing
<EventStream className="h-96 border border-gray-700 rounded-lg" />
```

#### Client Formatting Utilities - `utils/formatting.ts` (Phase 8 New)

**Purpose**: Consistent timestamp and duration formatting across the client

**File Size**: 331 lines including examples and constants

**Exported Functions**:

Timestamp formatting:
- `formatTimestamp(timestamp: string)` - Format to HH:MM:SS
  - Example: "2026-02-02T14:30:00Z" → "14:30:00"
  - Returns "--:--:--" for invalid input

- `formatTimestampFull(timestamp: string)` - Format to YYYY-MM-DD HH:MM:SS
  - Example: "2026-02-02T14:30:00Z" → "2026-02-02 14:30:00"
  - Returns "----/--/-- --:--:--" for invalid input

- `formatRelativeTime(timestamp: string, now?: Date)` - Human-readable relative time
  - Examples: "just now", "5m ago", "2h ago", "yesterday", "3d ago", "2w ago"
  - Returns "unknown" for invalid input

Duration formatting:
- `formatDuration(milliseconds: number)` - Human format with two significant units
  - Examples: "1h 30m", "5m 30s", "30s"
  - Returns "0s" for zero, negative, or invalid input

- `formatDurationShort(milliseconds: number)` - Digital clock format
  - Examples: "1:30:00", "5:30", "0:30"
  - Returns "0:00" for zero, negative, or invalid input

**Helper Functions**:
- `parseTimestamp(timestamp: string)` - Parse RFC 3339 to Date
- `padZero(value: number, width: number)` - Pad with leading zeros
- `isSameDay(date1: Date, date2: Date)` - Check same calendar day
- `isYesterday(date1: Date, date2: Date)` - Check if yesterday

**Constants**:
- Time unit constants: MS_PER_SECOND, MS_PER_MINUTE, MS_PER_HOUR, MS_PER_DAY, MS_PER_WEEK
- Fallback strings for invalid input

**Error Handling**:
- All functions handle invalid input gracefully
- No exceptions thrown
- Returns appropriate fallback values
- Type checking for NaN and non-string/non-number inputs

**Usage Examples**:
```typescript
// In EventStream component
const formattedTime = formatTimestamp(event.timestamp);

// In session list for "last activity"
const relativeTime = formatRelativeTime(session.lastEventAt);

// In session duration display
const duration = formatDuration(endTime - startTime);
```

## Module Boundaries

### Server Module Structure (Phase 3+)

**Public API** (`src/lib.rs`):
```rust
pub mod auth;
pub mod broadcast;
pub mod config;
pub mod error;
pub mod rate_limit;
pub mod routes;
pub mod types;
```

**Module Dependencies**:
```
routes.rs
  ├── uses: config, error, types, auth, broadcast, rate_limit
  └── provides: HTTP API and AppState

auth.rs
  ├── uses: config (public_keys, subscriber_token)
  └── provides: verify_signature, validate_token

broadcast.rs
  ├── uses: types (Event)
  └── provides: EventBroadcaster, SubscriberFilter

rate_limit.rs
  ├── uses: (self-contained)
  └── provides: RateLimiter, TokenBucket, RateLimitResult

config.rs
  ├── uses: (environment only)
  └── provides: Config struct

error.rs
  ├── uses: config (error conversions)
  └── provides: Error types

types.rs
  ├── uses: (no dependencies)
  └── provides: Event, EventType, EventPayload

main.rs
  ├── uses: config, routes, rate_limit
  └── provides: Server bootstrap and graceful shutdown
```

**Responsibility Separation**:
- `config` ← Configuration loading and validation from environment
- `error` ← Error type definitions and conversions
- `types` ← Event schema and serialization
- `routes` ← HTTP request/response handling and AppState creation
- `auth` ← Cryptographic verification and token validation
- `broadcast` ← Event distribution to multiple clients
- `rate_limit` ← Per-source request rate limiting
- `main` ← Server startup, logging setup, signal handling, graceful shutdown

### Monitor Module Structure (Phase 6)

**Public API** (`src/lib.rs`):
```rust
pub mod config;
pub mod crypto;
pub mod error;
pub mod parser;
pub mod privacy;
pub mod sender;
pub mod types;
pub mod watcher;

pub use config::Config;
pub use crypto::{Crypto, CryptoError};
pub use error::{MonitorError, Result};
pub use parser::{ParsedEvent, ParsedEventKind, SessionParser};
pub use privacy::{extract_basename, PrivacyConfig, PrivacyPipeline};
pub use sender::{Sender, SenderConfig, SenderError};
pub use types::{Event, EventPayload, EventType, SessionAction, ToolStatus};
pub use watcher::{FileWatcher, WatchEvent, WatcherError};
```

**Module Dependencies**:
```
main.rs
  ├── uses: config, crypto, sender (CLI entry point)
  └── provides: Command-line interface and daemon bootstrap

watcher.rs
  ├── uses: (filesystem + tokio)
  └── provides: FileWatcher, WatchEvent

parser.rs
  ├── uses: types (Event, EventPayload)
  └── provides: ParsedEvent, SessionParser

privacy.rs
  ├── uses: types (EventPayload)
  └── provides: PrivacyConfig, PrivacyPipeline, extract_basename

crypto.rs (Phase 6 NEW)
  ├── uses: (cryptographic libraries only)
  └── provides: Crypto, CryptoError

sender.rs (Phase 6 NEW)
  ├── uses: crypto, types (events), config (server URL)
  └── provides: Sender, SenderConfig, SenderError

config.rs
  ├── uses: (environment only)
  └── provides: Config struct

error.rs
  ├── uses: (self-contained)
  └── provides: MonitorError, Result

types.rs
  ├── uses: (no dependencies)
  └── provides: Event, EventPayload, EventType
```

**Responsibility Separation**:
- `config` ← Configuration loading and validation from environment
- `error` ← Error type definitions and conversions
- `types` ← Event schema and serialization
- `watcher` ← File system monitoring and position tracking (Phase 4)
- `parser` ← JSONL parsing and event normalization (Phase 4)
- `privacy` ← Event payload sanitization before transmission (Phase 5)
- `crypto` ← Keypair management and message signing (Phase 6 NEW)
- `sender` ← HTTP transmission, buffering, retry logic (Phase 6 NEW)
- `main` ← CLI parsing, daemon bootstrap, signal handling (Phase 6 NEW)

### Client Module Structure (Phase 8)

**Module Organization**:
- `types/events.ts` - All event type definitions and type guards
- `hooks/useEventStore.ts` - Zustand store for event state and session management
- `hooks/useWebSocket.ts` - WebSocket connection hook with auto-reconnect (Phase 7)
- `components/EventStream.tsx` - Virtual scrolling event list (Phase 8 NEW)
- `components/ConnectionStatus.tsx` - Connection status visual indicator (Phase 7)
- `components/TokenForm.tsx` - Token input and persistence form (Phase 7)
- `utils/formatting.ts` - Timestamp and duration formatting (Phase 8 NEW)
- `App.tsx` - Root component (Phase 3+)
- `main.tsx` - React entry point

**Module Dependencies (Phase 8)**:
```
App.tsx
  ├── uses: useWebSocket, useEventStore, EventStream, ConnectionStatus, TokenForm
  └── provides: Root component

EventStream.tsx (Phase 8 NEW)
  ├── uses: useEventStore (events subscription), @tanstack/react-virtual, formatting
  └── provides: Virtual scrolling event display with auto-scroll

useWebSocket.ts (Phase 7)
  ├── uses: types (VibeteaEvent), hooks (useEventStore)
  └── provides: WebSocket connection management, auto-reconnect

ConnectionStatus.tsx (Phase 7)
  ├── uses: hooks (useEventStore for status subscription)
  └── provides: Visual status indicator component

TokenForm.tsx (Phase 7)
  ├── uses: browser localStorage, React hooks
  └── provides: Token input and persistence UI

useEventStore.ts
  ├── uses: types (VibeteaEvent)
  └── provides: Zustand store for event state and session aggregation

formatting.ts (Phase 8 NEW)
  ├── uses: (no dependencies, pure functions)
  └── provides: Timestamp and duration formatting utilities

types/events.ts
  ├── uses: (no dependencies)
  └── provides: VibeteaEvent, EventType, EventPayload, Session types
```

**Responsibility Separation**:
- `types/events.ts` ← Type definitions matching Rust event schema
- `hooks/useEventStore.ts` ← Centralized state management with session aggregation
- `hooks/useWebSocket.ts` (Phase 7) ← WebSocket lifecycle and auto-reconnect logic
- `components/EventStream.tsx` (Phase 8) ← Virtual scrolling rendering and auto-scroll
- `components/ConnectionStatus.tsx` (Phase 7) ← Visual connection status display
- `components/TokenForm.tsx` (Phase 7) ← Token input and localStorage management
- `utils/formatting.ts` (Phase 8) ← Pure formatting functions for display
- `App.tsx` ← Root component orchestration (Phase 3+)

## Where to Add New Code

| If you're adding... | Put it in... | Example |
|---------------------|--------------|---------|
| New server feature logic | `server/src/{feature}.rs` in public mod or routes | `auth.rs`, `broadcast.rs`, `rate_limit.rs` |
| New HTTP route | `server/src/routes.rs` function | `post_events`, `get_ws`, `get_health` |
| New error type | `server/src/error.rs` enum variant | `AuthError`, `RateLimitError` |
| New monitor feature | `monitor/src/{feature}.rs` | `watcher.rs`, `parser.rs` (Phase 4), `privacy.rs` (Phase 5) |
| File watching logic | `monitor/src/watcher.rs` | Extend FileWatcher (Phase 4) |
| JSONL parsing logic | `monitor/src/parser.rs` | Extend SessionParser (Phase 4) |
| Privacy filtering logic | `monitor/src/privacy.rs` | Extend PrivacyPipeline (Phase 5) |
| Signing/crypto logic | `monitor/src/crypto.rs` | Extend Crypto (Phase 6) |
| Event transmission logic | `monitor/src/sender.rs` | Extend Sender with new retry strategies (Phase 6) |
| CLI commands | `monitor/src/main.rs` | New Command enum variant and handler (Phase 6) |
| Monitor main logic | `monitor/src/main.rs` | Watch directory, parse files, sanitize, send events (Phase 6+) |
| New React component | `client/src/components/{feature}/` | `client/src/components/sessions/SessionList.tsx` |
| New event display | `client/src/components/` | Extends EventStream with new event type display (Phase 8) |
| New client hook | `client/src/hooks/` | `client/src/hooks/useWebSocket.ts` (Phase 7), `client/src/hooks/useSession.ts` |
| Client connection logic | `client/src/hooks/useWebSocket.ts` | Extend reconnection or message parsing (Phase 7) |
| Client UI indicator | `client/src/components/` | Status display, error handling components (Phase 7) |
| Client authentication | `client/src/components/` | Token form, credential management (Phase 7) |
| New formatting function | `client/src/utils/formatting.ts` | Add new timestamp or duration format (Phase 8) |
| New utility function | `client/src/utils/` | `client/src/utils/sessionHelpers.ts` |
| New type definition | `client/src/types/` | `client/src/types/api.ts` |
| Server integration tests | `server/tests/` | `server/tests/unsafe_mode_test.rs` |
| Monitor tests | `monitor/tests/` | `monitor/tests/privacy_test.rs` (Phase 5) |
| Client unit tests | `client/src/__tests__/` | `client/src/__tests__/events.test.ts` |
| Virtual scrolling customization | `client/src/components/EventStream.tsx` | Adjust ESTIMATED_ROW_HEIGHT, AUTO_SCROLL_THRESHOLD |

## Import Paths

### Rust (Monorepo Workspace)

All crates are defined in root `Cargo.toml`:
```toml
[workspace]
members = ["server", "monitor"]
```

Within each crate, use relative imports:
```rust
use crate::config::Config;
use crate::error::ServerError;
use crate::types::Event;
use crate::routes::{create_router, AppState};
use crate::auth::verify_signature;
use crate::broadcast::EventBroadcaster;
use crate::rate_limit::RateLimiter;
use crate::watcher::FileWatcher;
use crate::parser::SessionParser;
use crate::privacy::{PrivacyConfig, PrivacyPipeline, extract_basename};
use crate::crypto::{Crypto, CryptoError};
use crate::sender::{Sender, SenderConfig, SenderError};
```

### TypeScript

Configure path aliases in `client/tsconfig.json`:
```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@types/*": ["src/types/*"],
      "@hooks/*": ["src/hooks/*"],
      "@components/*": ["src/components/*"],
      "@utils/*": ["src/utils/*"]
    }
  }
}
```

Example usage:
```typescript
import type { VibeteaEvent } from '@types/events';
import { useEventStore } from '@hooks/useEventStore';
import { useWebSocket } from '@hooks/useWebSocket';
import { EventStream } from '@components/EventStream';
import { ConnectionStatus } from '@components/ConnectionStatus';
import { TokenForm } from '@components/TokenForm';
import { formatTimestamp, formatRelativeTime } from '@utils/formatting';
```

## Entry Points

| File | Purpose | Status |
|------|---------|--------|
| `server/src/main.rs` | Server application bootstrap with graceful shutdown | Phase 3 complete |
| `server/src/lib.rs` | Server public library API | Exports all modules |
| `monitor/src/main.rs` | Monitor CLI with init/run commands | Phase 6 complete |
| `monitor/src/lib.rs` | Monitor public library API | Exports watcher, parser, privacy, crypto, sender, types (Phase 6) |
| `client/src/main.tsx` | React DOM render | Renders App into #root |
| `client/src/App.tsx` | Root React component | Placeholder (Phase 3) |
| `client/index.html` | HTML template | Vite entry point |

## Generated Files

Files that are auto-generated or compile-time artifacts:

| Location | Generator | Notes |
|----------|-----------|-------|
| `target/debug/` | `cargo build` | Rust debug binaries and artifacts |
| `target/release/` | `cargo build --release` | Rust release binaries and artifacts |
| `client/dist/` | `npm run build` (Vite) | Bundled client JavaScript and CSS |
| `Cargo.lock` | `cargo` | Dependency lock file (committed) |

## Phase 8 Implementation Summary

The following modules were added/updated in Phase 8 for the client:

**Client New Modules (Phase 8)**:

- `src/components/EventStream.tsx` - Virtual scrolling event list (NEW - Phase 8)
  - Efficient rendering of 1000+ events using @tanstack/react-virtual
  - Auto-scroll to bottom when new events arrive
  - "Jump to Latest" button when user scrolls up
  - Event type icons and color-coded badges
  - Formatted timestamps and event descriptions
  - Accessibility features (ARIA labels, semantic HTML)
  - 425 lines implementing full component with sub-components

- `src/utils/formatting.ts` - Timestamp and duration formatting (NEW - Phase 8)
  - Timestamp formatting: HH:MM:SS, YYYY-MM-DD HH:MM:SS, relative time
  - Duration formatting: human readable (1h 30m), digital clock (1:30:00)
  - Helper functions for date comparison and padding
  - Graceful error handling with fallback values
  - Pure functions suitable for testing
  - 331 lines implementing all formatting utilities

**Client Phase 8 Architecture**:

The new Phase 8 client modules implement efficient event streaming UI with:

1. **Virtual Scrolling**: EventStream component uses @tanstack/react-virtual to render only visible events
2. **Auto-Scroll Management**: Automatically jumps to latest events, pauses when user scrolls up
3. **Event Display**: Type-specific icons, color-coded badges, formatted timestamps, event descriptions
4. **Performance**: Selective Zustand subscriptions prevent unnecessary re-renders
5. **Formatting**: Reusable, tested formatting functions for consistent display across components
6. **Accessibility**: Proper ARIA labels, semantic HTML, screen reader support

**Integration Points**:

- EventStream ← useEventStore: Displays events from store
- EventStream ← formatting utilities: Formats timestamps and displays event descriptions
- App.tsx (Phase 3+): Orchestrates all components including EventStream
- TokenForm → localStorage: Persists authentication token
- ConnectionStatus → useEventStore: Displays current connection state

---

*This document shows WHERE code lives. Update when directory structure or module organization changes.*
