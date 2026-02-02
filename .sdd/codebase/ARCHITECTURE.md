# Architecture

**Status**: Phase 4 incremental update - Monitor parser and file watcher implementation
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Architecture Overview

VibeTea is a three-tier real-time event streaming system with clear separation of concerns:

- **Monitor** (Rust): Event producer that watches Claude Code session files and captures activity
- **Server** (Rust): Event hub that authenticates monitors and broadcasts to clients
- **Client** (TypeScript/React): Event consumer that displays sessions and activities

The system follows a hub-and-spoke pattern where monitors are trusted publishers and clients are passive subscribers. All communication is event-driven with no persistent state required on the server.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Monitors push events to the server, clients subscribe via WebSocket |
| **Event-Driven** | All state changes flow through immutable, versioned events |
| **Layered** | Monitor: Config/Watcher/Parser → Types; Server: Routes → Auth/Broadcast/RateLimit → Types; Client: Types → Hooks → Components |
| **Pub/Sub** | Server acts as event broker with asymmetric authentication (monitors sign, clients consume) |
| **Token Bucket** | Per-source rate limiting using token bucket algorithm with stale entry cleanup |
| **File Tailing** | Monitor uses position tracking to efficiently read only new content from JSONL files |

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

### Monitor Component

**Purpose**: Captures Claude Code session activity and transmits to server
**Location**: `monitor/src/`
**Technologies**: Rust, tokio, file watching, JSONL parsing, Ed25519 cryptography

**Module Hierarchy**:
```
monitor/src/
├── main.rs       - Entry point (Phase 4 placeholder)
├── lib.rs        - Public API exports
├── config.rs     - Environment variable parsing
├── types.rs      - Event definitions
├── error.rs      - Error hierarchy
├── watcher.rs    - File system watching (Phase 4 new)
└── parser.rs     - JSONL parsing (Phase 4 new)
```

**Key Features (Phase 4)**:
- File system watching for `.jsonl` files
- Incremental parsing with position tracking
- Claude Code event format normalization
- Privacy-first approach (metadata only)

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

### Monitor → Server Flow (Phase 4)

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
    Sign with Ed25519 Private Key
         ↓
    Batch and Buffer
         ↓
    HTTPS POST to /events
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
6. Events are buffered and signed with monitor's private key
7. Signed batch is sent to server's `/events` endpoint via HTTPS POST
8. Server route handler extracts source ID from X-Source-ID header
9. Auth module verifies signature using configured public key
10. Rate limiter checks request allowance for source
11. Event schema is validated
12. EventBroadcaster immediately forwards to all subscribed WebSocket clients

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
| **Monitor File Watcher** | File system observation, position tracking | Filesystem | Parser directly (events via channel) |
| **Monitor Parser** | JSONL parsing, event normalization | Types | Filesystem directly |
| **Client Component** | Display UI, handle user actions | Store hooks, types | WebSocket layer directly |

## Dependency Rules

- **No circular dependencies**: Routes → (Auth, Broadcast, RateLimit), Auth → Config, Broadcast → Types, RateLimit → (no deps)
- **Auth is read-only**: Auth module never modifies config, only reads public keys and tokens
- **Rate limiting is stateless to requests**: Each request gets its own check, no persistent memory beyond buckets
- **Broadcast is fire-and-forget**: Events are sent but no confirmation/acknowledgment required
- **Type safety**: All three languages use strong typing; event schema is enforced at compile time
- **Asymmetric auth**: Monitors authenticate with cryptographic signatures; clients authenticate with bearer tokens
- **File watcher isolation**: Watcher and parser communicate via channels, no direct file access from parser

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

## Testing Strategy

**Server Tests**: Located in `routes.rs` using axum test utilities
- Health endpoint tests (uptime reporting, subscriber counting)
- Event ingestion tests (single/batch, with/without auth)
- Authentication tests (valid signature, missing header, invalid signature, unknown source)
- Rate limiting tests (under limit, over limit, retry-after)
- WebSocket filter tests (source, event type, project filtering)
- AppState initialization tests

**Integration Tests**: Located in `server/tests/unsafe_mode_test.rs`
- End-to-end scenarios in unsafe mode (auth disabled)

**Running Tests**:
```bash
cargo test --package vibetea-server  # All server tests
cargo test --package vibetea-server routes  # Route tests only
cargo test --package vibetea-monitor  # All monitor tests
```

## Graceful Shutdown Flow

```
Signal (SIGTERM/SIGINT)
         ↓
shutdown_signal() async
         ↓
Log shutdown initiation
         ↓
axum graceful shutdown
         ↓
Abort cleanup task
         ↓
Allow in-flight requests to complete (30s timeout)
         ↓
Exit with success
```

**Shutdown Timeout**: 30 seconds for in-flight requests to complete
**Cleanup Interval**: Rate limiter cleans up every 30 seconds

---

*This document describes HOW the system is organized. Keep focus on patterns and relationships.*
