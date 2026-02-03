# Architecture

**Status**: Phase 10 incremental update - Session Overview component and session state machine
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Architecture Overview

VibeTea is a three-tier real-time event streaming system with clear separation of concerns:

- **Monitor** (Rust): Event producer that watches Claude Code session files and captures activity with privacy guarantees
- **Server** (Rust): Event hub that authenticates monitors and broadcasts to clients
- **Client** (TypeScript/React): Event consumer that displays sessions and activities with WebSocket connection management

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
| **Client Connection Management** | WebSocket hook with auto-reconnect, exponential backoff, and bearer token authentication (Phase 7) |
| **Client State Management** | Zustand store for event buffer and session aggregation with selective subscriptions (Phase 7) |
| **Virtual Scrolling** | Efficient rendering of 1000+ events using @tanstack/react-virtual with auto-scroll behavior (Phase 8) |
| **Session State Machine** | Sessions transition through states (Active → Inactive → Ended → Removed) based on event activity and time thresholds (Phase 10) |
| **Session Overview Display** | Dedicated component showing active sessions with real-time activity indicators and status badges (Phase 10) |
| **Activity Heatmap** | Visualization of event frequency over time using CSS Grid with color intensity mapping (Phase 9) |

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

### Client WebSocket Connection Hook

**Purpose**: Manage WebSocket lifecycle with automatic reconnection and bearer token authentication
**Location**: `client/src/hooks/useWebSocket.ts`
**Responsibility**: Connection management, reconnection logic, event message parsing and dispatch

**Phase 7 New - WebSocket Hook Pattern**:

The hook provides automatic connection management with exponential backoff reconnection:

**Connection Management**:
- `connect()` - Establish WebSocket connection
  - Validates token from localStorage before connecting
  - Prevents multiple simultaneous connection attempts
  - Sets connection status to 'connecting' or 'connected'
- `disconnect()` - Graceful disconnection
  - Disables auto-reconnect flag
  - Clears pending reconnection timeouts
  - Closes WebSocket if open
  - Sets status to 'disconnected'

**Reconnection Strategy**:
- Exponential backoff: 1s initial, 2^attempt formula, capped at 60s
- Jitter: ±25% randomization to prevent thundering herd
- Automatic reconnection on connection loss
- Manual disconnect disables auto-reconnect
- Reconnect attempt counter resets on successful connection

**Token Authentication**:
- Bearer token stored in localStorage (key: `vibetea_token`)
- Token included in WebSocket URL as query parameter
- Token validation occurs before connection attempt
- If token missing, logs warning and sets status to 'disconnected'

**Event Message Handling**:
- Parses incoming WebSocket messages as VibeteaEvent JSON
- Basic structural validation (id, source, timestamp, type, payload fields)
- Dispatches valid events to useEventStore via `addEvent()`
- Silently discards invalid/malformed messages (logs nothing)

**Connection Status States**:
- `'connecting'` - WebSocket creation initiated, handshake in progress
- `'connected'` - WebSocket open, connection established
- `'disconnecting'` - (implicit, no separate state)
- `'disconnected'` - WebSocket closed or never connected
- `'reconnecting'` - Auto-reconnect scheduled after connection loss

**Key Functions**:
- `calculateBackoff(attempt)` - Compute exponential delay with jitter
- `buildWebSocketUrl(baseUrl, token)` - Add token as query parameter
- `getDefaultUrl()` - Construct URL from current window location (ws:// or wss://)
- `parseEventMessage(data)` - Parse and validate incoming JSON message

**Key Types**:
- `UseWebSocketReturn` - Hook return interface with connect, disconnect, isConnected
- Constants: INITIAL_BACKOFF_MS (1000), MAX_BACKOFF_MS (60000), JITTER_FACTOR (0.25)

**Example Usage**:
```tsx
function EventMonitor() {
  const { connect, disconnect, isConnected } = useWebSocket();

  useEffect(() => {
    connect();
    return () => disconnect();
  }, [connect, disconnect]);

  return <div>Status: {isConnected ? 'Connected' : 'Disconnected'}</div>;
}
```

**Memory Management**:
- Uses React useRef for WebSocket and timeout references
- Cleanup on unmount: closes WebSocket, clears timeouts, disables reconnect
- Prevents memory leaks through proper ref cleanup

### Client State Management Hook

**Purpose**: Centralized Zustand store for event buffer and session aggregation
**Location**: `client/src/hooks/useEventStore.ts`
**Responsibility**: Event state management with session aggregation and selective subscriptions

**Phase 10 Integration - Session State Machine**:

The Zustand store now includes session state machine with automatic transitions:

**Session State Machine**:
- **New File Detected** → `Active` (first event)
- **Active** → `Inactive` (no events for 5 minutes)
- **Inactive** → `Active` (event received)
- **Active/Inactive** → `Ended` (summary event received)
- **Ended/Inactive** → `Removed` (30 minutes since last event)

**Store State**:
- `status` - Current connection status (connecting, connected, disconnected, reconnecting)
- `events` - Event buffer (last 1000 events, newest first, FIFO eviction)
- `sessions` - Session map keyed by sessionId, tracking project, activity, event count, status

**Actions**:
- `addEvent(event)` - Add event to buffer, update session state
  - Enforces FIFO eviction when buffer exceeds 1000 events
  - Creates new session entry on first event
  - Updates session with latest project, timestamp, status, event count
  - Marks session as 'ended' on summary event type
  - Reactivates inactive session on new non-summary event
- `setStatus(status)` - Update connection status
- `clearEvents()` - Clear event buffer and sessions (for testing/reset)
- `updateSessionStates()` - Transition sessions based on time thresholds (called periodically by useSessionTimeouts)

**Selector Utilities**:
- `selectEventsBySession(state, sessionId)` - Get events for specific session
- `selectActiveSessions(state)` - Get sessions with status !== 'ended'
- `selectSession(state, sessionId)` - Get single session by ID

**Session Status Values**:
- `'active'` - Session has received events in last 5 minutes
- `'inactive'` - Session has not received events for 5+ minutes
- `'ended'` - Summary event received for session
- Sessions are removed from store 30+ minutes after last event

**Key Types**:
- `ConnectionStatus` - Union: 'connecting' | 'connected' | 'disconnected' | 'reconnecting'
- `Session` - Contains sessionId, source, project, startedAt, lastEventAt, status, eventCount
- `EventStore` - Full state and action interface

**Performance Optimization**:
- Selective subscriptions: components subscribe to specific fields (e.g., `state.status`)
- Prevents re-renders when other fields update
- Map-based session storage for O(1) lookups
- Event buffer limited to 1000 to prevent unbounded memory growth

**Example Usage**:
```tsx
// Subscribe to sessions only (re-renders only on session changes)
const sessions = useEventStore((state) => state.sessions);

// Get session state update function
const updateSessionStates = useEventStore((state) => state.updateSessionStates);

// Get actions (don't trigger re-renders)
const addEvent = useEventStore((state) => state.addEvent);
```

### Client Session Timeout Hook

**Purpose**: Initialize and manage session state machine transitions
**Location**: `client/src/hooks/useSessionTimeouts.ts`
**Responsibility**: Periodic session state updates based on time thresholds

**Phase 10 New - Session Timeout Hook**:

The hook sets up a periodic interval to check and transition session states:

**Functionality**:
- Calls `updateSessionStates()` every 30 seconds (SESSION_CHECK_INTERVAL_MS)
- Transitions active sessions to inactive after 5 minutes without events
- Removes inactive/ended sessions after 30 minutes without events
- Cleans up interval on component unmount

**Usage Pattern**:
- Must be called once at app root level (App.tsx)
- Non-rendering hook (returns void)
- Sets up cleanup on unmount

**Example Usage**:
```tsx
// In App.tsx (call once at root level)
export default function App() {
  useSessionTimeouts();
  return <Dashboard />;
}
```

**Integration with useEventStore**:
- Uses selective subscription to get `updateSessionStates` action
- Does not trigger re-renders (action subscription)
- Clean dependency management

### Client Session Overview Component

**Purpose**: Display active AI assistant sessions with real-time activity indicators
**Location**: `client/src/components/SessionOverview.tsx`
**Responsibility**: Session display, activity visualization, filtering, accessibility

**Phase 10 New - Session Overview Component**:

The component provides a comprehensive view of all sessions with activity-based animations:

**Features**:
- Real-time activity indicators with variable pulse rates
- Session duration and last active time tracking
- Status badges (Active, Idle, Ended)
- Dimmed styling for inactive/ended sessions
- Click to filter events by session
- Accessible with ARIA labels and keyboard navigation

**Activity Indicator System**:
- Pulse rate varies based on event volume:
  - 1-5 events/min: 1Hz pulse (slow)
  - 6-15 events/min: 2Hz pulse (medium)
  - 16+ events/min: 3Hz pulse (fast)
  - No events for 60s: no pulse

**Component Hierarchy**:
- `SessionOverview` - Main component displaying all sessions
- `SessionCard` - Individual session card with metadata
- `ActivityIndicator` - Pulsing dot showing activity level
- `StatusBadge` - Status label (Active, Idle, Ended)
- `EmptyState` - Placeholder when no sessions available

**Key Functions**:
- `getActivityLevel()` - Determine pulse rate from event count
- `getSessionDuration()` - Calculate elapsed time since session start
- `sortSessions()` - Sort by status (active first) then by most recent activity
- `countRecentEventsBySession()` - Count events in last 60 seconds per session

**Props**:
- `className` (string, optional) - Additional CSS classes for container
- `onSessionClick` (callback, optional) - Called when session card is clicked with sessionId

**Styling**:
- Dark theme with Tailwind classes (bg-gray-800/900)
- Color-coded status badges (green/yellow/gray)
- Opacity scaling for inactive sessions
- Hover effects on clickable cards
- Focus ring styling for keyboard navigation

**Accessibility**:
- `role="region"` on main container
- `aria-label` with session info for card
- `role="list"` and `role="listitem"` for semantic list structure
- Keyboard navigation (Tab, Enter, Space)
- ARIA attributes for activity indicator
- Decorative elements marked `aria-hidden="true"`

**Performance**:
- Selective Zustand subscription to sessions and events
- Memoized helper functions and constants
- Efficient event counting with time-based windowing
- Map-based session lookup

**Example Usage**:
```tsx
// Basic usage
<SessionOverview />

// With click handler for filtering
<SessionOverview
  onSessionClick={(sessionId) => {
    console.log(`Filter to session: ${sessionId}`);
  }}
/>

// With custom styling
<SessionOverview className="p-4 bg-gray-800 rounded-lg" />
```

### Client Activity Heatmap Component

**Purpose**: Visualize event frequency over time with color intensity mapping
**Location**: `client/src/components/Heatmap.tsx`
**Responsibility**: Heatmap grid rendering, cell interaction, timezone-aware bucketing

**Phase 9 New - Activity Heatmap Component**:

The component provides a visual representation of session activity patterns:

**Features**:
- CSS Grid layout with hours on X-axis and days on Y-axis
- Color scale from dark (0 events) to bright green (51+ events)
- Toggle between 7-day and 30-day views
- Timezone-aware hour bucketing using local time
- Cell click filtering to select events from specific hour
- Accessible with proper ARIA labels and keyboard navigation

**Heatmap Grid**:
- X-axis: 24 hours (0-23)
- Y-axis: 7 or 30 days
- Hour labels displayed at 0, 6, 12, 18 for readability
- Day names abbreviated (Sun, Mon, Tue, etc.)

**Color Scale**:
- 0 events: Dark gray (opacity)
- 1-5 events: Light green
- 6-10 events: Medium green
- 11-20 events: Bright green
- 21-50 events: Very bright green
- 51+ events: Intense bright green

**Key Functions**:
- `calculateDateRange()` - Get dates for 7 or 30-day view
- `countEventsInHourBucket()` - Count events for specific date/hour
- `getColorIntensity()` - Map event count to color class

**Props**:
- `className` (string, optional) - Additional CSS classes for container
- `onCellClick` (callback, optional) - Called when cell is clicked with start/end time

**Styling**:
- CSS Grid layout for aligned heatmap
- Tailwind color classes for intensity mapping
- Hover effects on cells
- Focus ring styling for keyboard navigation

**Accessibility**:
- `role="region"` on main container
- `aria-label` with grid information
- Cell tooltips showing date, hour, count
- Keyboard navigation support
- ARIA live region for selected time range

**Example Usage**:
```tsx
// Basic 7-day heatmap
<Heatmap />

// 30-day view with click handler
<Heatmap
  onCellClick={(startTime, endTime) => {
    console.log(`Selected: ${startTime} to ${endTime}`);
  }}
/>

// Custom styling
<Heatmap className="p-4 border border-gray-700 rounded-lg" />
```

### Client Connection Status Component

**Purpose**: Visual indicator of WebSocket connection state
**Location**: `client/src/components/ConnectionStatus.tsx`
**Responsibility**: Display connection status with colored indicator and optional label

**Phase 7 New - Status Component**:

Provides visual feedback for WebSocket connection state:

**Display States**:
- `'connected'` - Green dot with "Connected" label
- `'connecting'` - Yellow dot with "Connecting" label
- `'reconnecting'` - Yellow dot with "Reconnecting" label
- `'disconnected'` - Red dot with "Disconnected" label

**Props**:
- `showLabel` (boolean, default false) - Whether to display status text
- `className` (string, default '') - Additional CSS classes for container

**Styling**:
- Inline flex layout with gap-2
- Tailwind classes: bg-green-500, bg-yellow-500, bg-red-500 for status
- Responsive typography for labels
- ARIA attributes for accessibility (role="status", aria-label with status)

**Performance**:
- Selective Zustand subscription to `state.status` only
- Prevents re-renders during high-frequency event streams
- Memoized STATUS_CONFIG mapping for O(1) lookups

**Accessibility**:
- `role="status"` announces dynamic status changes
- `aria-label` provides description for screen readers
- `aria-hidden="true"` on decorative dot

**Example Usage**:
```tsx
// Compact indicator only
<ConnectionStatus />

// With status text
<ConnectionStatus showLabel />

// With custom positioning
<ConnectionStatus className="absolute top-4 right-4" showLabel />
```

### Client Token Input Form Component

**Purpose**: Manage WebSocket authentication token with persistent storage
**Location**: `client/src/components/TokenForm.tsx`
**Responsibility**: Token input, validation, persistence, and lifecycle management

**Phase 7 New - Token Form Component**:

Provides UI for managing authentication tokens with localStorage integration:

**Functionality**:
- Save token to localStorage (key: `vibetea_token`)
- Clear token from localStorage
- Display token save status (saved/not-saved)
- Cross-tab awareness: updates when token changes in another tab
- Optional callback on token change for reconnection

**Props**:
- `onTokenChange` (callback, optional) - Called after token save or clear
- `className` (string, default '') - Additional CSS classes

**Form Fields**:
- Password input for token entry
- Placeholder changes based on save status ("Enter new token" vs "Enter your token")
- Autocomplete disabled to prevent browser leaking tokens
- Auto-clear input after save

**Status Indicator**:
- Green dot + "Token saved" when localStorage has token
- Gray dot + "No token saved" when localStorage empty
- Updates immediately on save/clear
- Listens to storage events for cross-tab changes

**Buttons**:
- "Save Token" - Enabled when input not empty, disabled when input empty
- "Clear" - Enabled when token saved, disabled when empty
- Both buttons have hover effects and focus rings

**Styling**:
- Dark theme with Tailwind classes
- Form layout with spacing and labels
- Button states: hover, active, disabled, focus
- Password input with focus ring styling

**Storage Integration**:
- Reads/writes to `localStorage.getItem('vibetea_token')`
- Listens for storage events (cross-tab sync)
- Trims whitespace from input
- Validates non-empty before save

**Example Usage**:
```tsx
function Settings() {
  const { connect } = useWebSocket();

  return (
    <TokenForm
      onTokenChange={() => {
        // Reconnect when token changes
        connect();
      }}
    />
  );
}
```

### Client Virtual Scrolling Event Stream Component

**Purpose**: Efficiently render large event streams with auto-scroll and jump-to-latest functionality
**Location**: `client/src/components/EventStream.tsx`
**Responsibility**: Virtual scrolling, event display, auto-scroll behavior, user interaction

**Phase 8 New - Virtual Scrolling Component**:

Provides efficient rendering of 1000+ events with intuitive scrolling behavior:

**Virtual Scrolling Engine**:
- Uses `@tanstack/react-virtual` for efficient rendering
- Estimated row height: 64px for layout calculation
- Overscan: 5 items for smooth scrolling
- Only renders visible rows plus overscan buffer
- Prevents performance degradation with large event lists

**Auto-Scroll Behavior**:
- Automatically scrolls to bottom when new events arrive
- Disables auto-scroll when user scrolls up 50px+ from bottom
- "Jump to Latest" button appears when auto-scroll is disabled
- Shows count of pending new events on jump button
- Re-enables auto-scroll when user clicks jump button

**Event Display**:
- Event type icons (🔧 tool, 💬 activity, 🚀 session, 📋 summary, ⚠️ error, 🤖 agent)
- Color-coded type badges (blue/green/purple/cyan/red/amber)
- Formatted timestamp (HH:MM:SS)
- Source and session ID (first 8 chars)
- Event description with payload-specific details

**Event Description Formatting**:
- Session events: "Session started/ended: {project}"
- Activity events: "Activity in {project}" or "Activity heartbeat"
- Tool events: "{tool_name} started/completed: {context}"
- Agent events: "Agent state: {state}"
- Summary events: First 80 chars of summary text
- Error events: "Error: {category}"

**Sub-components**:
- `EventRow` - Single event row with icon, badge, description, timestamp
- `JumpToLatestButton` - Button to jump to latest events with pending count
- `EmptyState` - Placeholder when no events available

**Props**:
- `className` (string, optional) - Additional CSS classes for container

**Accessibility**:
- `role="log"` on main container with `aria-live="polite"`
- `role="list"` on scrollable container
- `role="listitem"` on each event row
- `aria-label` with event type, time, and description
- Proper semantic HTML with `<time>` elements for timestamps
- Decorative SVGs marked with `aria-hidden="true"`

**Styling**:
- Dark theme with Tailwind classes (bg-gray-900)
- Hover effects on event rows (bg-gray-800/50)
- Color-coded badges with borders
- Smooth transitions
- Focus ring styling on jump button

**Performance**:
- Virtual scrolling prevents rendering all 1000+ events
- Selective Zustand subscription to events only
- Memoized helper functions (formatTimestamp, getEventDescription)
- Event row reverse for newest-first display matching user expectations

**Key Constants**:
- `ESTIMATED_ROW_HEIGHT` - 64px per event
- `AUTO_SCROLL_THRESHOLD` - 50px from bottom to disable auto-scroll
- `EVENT_TYPE_ICONS` - Map of event type to Unicode emoji
- `EVENT_TYPE_COLORS` - Map of event type to Tailwind color classes

**Key Functions**:
- `formatTimestamp(timestamp)` - Format RFC 3339 to HH:MM:SS
- `getEventDescription(event)` - Create human-readable event summary
- `handleScroll()` - Detect user scroll position for auto-scroll control
- `handleJumpToLatest()` - Jump to bottom and reset auto-scroll

**Example Usage**:
```tsx
// Basic usage with full height
<EventStream className="h-full" />

// Custom sizing
<EventStream className="h-96 border border-gray-700 rounded-lg" />
```

### Client Formatting Utilities

**Purpose**: Consistent timestamp and duration formatting across the client application
**Location**: `client/src/utils/formatting.ts`
**Responsibility**: Format conversion for display with graceful error handling

**Phase 8 New - Formatting Utilities**:

Provides pure, testable formatting functions for timestamps and durations:

**Timestamp Formatting Functions**:
- `formatTimestamp(timestamp: string)` - Format to HH:MM:SS (time only)
  - Example: "2026-02-02T14:30:00Z" → "14:30:00"
  - Returns "--:--:--" for invalid input
  - Uses local timezone

- `formatTimestampFull(timestamp: string)` - Format to YYYY-MM-DD HH:MM:SS
  - Example: "2026-02-02T14:30:00Z" → "2026-02-02 14:30:00"
  - Returns "----/--/-- --:--:--" for invalid input
  - Uses local timezone

- `formatRelativeTime(timestamp: string, now?: Date)` - Human-readable relative time
  - Examples: "just now", "5m ago", "2h ago", "yesterday", "3d ago", "2w ago"
  - Handles future timestamps (shows as "just now")
  - Returns "unknown" for invalid input
  - Useful for displaying "last activity" in session lists

**Duration Formatting Functions**:
- `formatDuration(milliseconds: number)` - Human format with two significant units
  - Examples: "1h 30m", "5m 30s", "30s"
  - Returns "0s" for zero, negative, or invalid input
  - Shows up to two units (hours+minutes, or minutes+seconds, or seconds only)

- `formatDurationShort(milliseconds: number)` - Digital clock format (H:MM:SS or M:SS)
  - Examples: "1:30:00", "5:30", "0:30"
  - Returns "0:00" for zero, negative, or invalid input
  - Compact format for timer or duration displays

**Helper Functions**:
- `parseTimestamp(timestamp: string)` - Parse RFC 3339 to Date (returns null if invalid)
- `padZero(value: number, width: number)` - Pad number with leading zeros
- `isSameDay(date1: Date, date2: Date)` - Check if dates are on same calendar day
- `isYesterday(date1: Date, date2: Date)` - Check if date1 is yesterday relative to date2

**Constants**:
- `INVALID_TIMESTAMP_FALLBACK` - "--:--:--"
- `INVALID_TIMESTAMP_FULL_FALLBACK` - "----/--/-- --:--:--"
- `INVALID_RELATIVE_TIME_FALLBACK` - "unknown"
- `INVALID_DURATION_FALLBACK` - "0s"
- `INVALID_DURATION_SHORT_FALLBACK` - "0:00"
- Time unit constants: MS_PER_SECOND, MS_PER_MINUTE, MS_PER_HOUR, MS_PER_DAY, MS_PER_WEEK

**Error Handling**:
- All functions handle invalid input gracefully with appropriate fallbacks
- No exceptions thrown; returns fallback values for invalid timestamps or durations
- Type checking for NaN and non-string/non-number inputs

**Usage Examples**:
```typescript
// In EventStream component
const formattedTime = formatTimestamp(event.timestamp);

// In session list for "last activity"
const relativeTime = formatRelativeTime(session.lastEventAt);

// In session duration display
const duration = formatDuration(endTime - startTime);
const shortDuration = formatDurationShort(totalMs);
```

### Client Component

**Purpose**: Subscribes to server events, displays sessions and activities
**Location**: `client/src/`
**Technologies**: TypeScript, React, Zustand, Vite, @tanstack/react-virtual

**Key Modules**:
- `types/events.ts` - TypeScript definitions matching Rust types with type guards
- `hooks/useEventStore.ts` - Zustand store for event state management with session machine
- `hooks/useWebSocket.ts` - WebSocket connection hook with auto-reconnect (Phase 7)
- `hooks/useSessionTimeouts.ts` - Session state transition hook (Phase 10 NEW)
- `components/SessionOverview.tsx` - Session cards with activity indicators (Phase 10 NEW)
- `components/Heatmap.tsx` - Activity visualization grid (Phase 9)
- `components/EventStream.tsx` - Virtual scrolling event list with auto-scroll (Phase 8)
- `components/ConnectionStatus.tsx` - Connection status visual indicator (Phase 7)
- `components/TokenForm.tsx` - Token input and persistence form (Phase 7)
- `utils/formatting.ts` - Timestamp and duration formatting utilities (Phase 8)
- `App.tsx` - Root component (Phase 3+ with Phase 10 integration)
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

### Server → Client Flow (Phase 10)

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
   WebSocket Message Event
        ↓
   useWebSocket Hook
        ↓
   parseEventMessage() validates JSON
        ↓
   addEvent() dispatches to Zustand store
        ↓
   useEventStore processes event
        ↓
   Session creation or update (state machine)
        ↓
   updateSessionStates() transitions states (periodic)
        ↓
   Component re-render (selective subscription)
        ↓
   SessionOverview displays updated session
        ↓
   ActivityIndicator shows current pulse rate
        ↓
   EventStream virtual rendering
        ↓
   Display in UI with auto-scroll
```

**Flow Steps (Phase 10 Client)**:
1. Server's EventBroadcaster sends event to all WebSocket subscriptions
2. Client WebSocket receives message with serialized event JSON
3. useWebSocket hook's `onmessage` handler is triggered
4. `parseEventMessage()` validates incoming message structure
5. If valid, `addEvent()` is called, dispatching to Zustand store
6. `useEventStore.addEvent()` processes event:
   - Adds to event buffer (oldest evicted if >1000)
   - Creates new session on first event (status='active')
   - Updates existing session with latest project, timestamps, event count
   - Marks session as 'ended' on summary event
   - Reactivates session if event received while 'inactive'
7. useSessionTimeouts calls updateSessionStates() every 30 seconds:
   - Transitions active sessions to 'inactive' after 5 minutes without events
   - Removes sessions after 30 minutes without any events
8. Zustand triggers selective notifications to subscribers
9. SessionOverview component receives updated sessions
10. SessionOverview counts recent events (last 60 seconds) per session
11. ActivityIndicator determines pulse rate based on event count
12. Activity indicators update with appropriate CSS animation class
13. Status badges update based on session state
14. Duration and last-active time display updated
15. EventStream component receives updated events array
16. Virtual scrolling recalculates item positions
17. Auto-scroll checks if enabled; if yes, jumps to latest
18. EventRow components render only visible items + overscan
19. User sees new event at bottom with smooth scroll
20. If user scrolls up, auto-scroll disables and jump button appears
21. Jump button shows count of pending new events

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
| **Client WebSocket Hook** | Connection management, reconnection, message parsing | localStorage (token), Browser WebSocket API | Direct component state |
| **Client Event Store** | State management, session aggregation, state machine | Event types | Direct API calls, localStorage (read-only) |
| **Client Session Timeouts** | Periodic session state transitions | useEventStore (updateSessionStates) | Event fetching, direct time tracking |
| **Client Session Overview** | Session display with activity indicators | useEventStore (sessions, events) | Direct API calls, WebSocket |
| **Client Virtual Scrolling** | Efficient rendering, auto-scroll, jump-to-latest | useEventStore (events), @tanstack/react-virtual | Direct API calls |
| **Client Formatting Utilities** | Timestamp/duration display formatting | Nothing (pure functions) | Component state, stores |
| **Client Components** | Display UI, handle user actions | Store hooks, types | WebSocket layer directly |

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
- **Client WebSocket manages connection**: Only useWebSocket hook manages WebSocket instance and reconnection
- **Store owns event state**: Only useEventStore manages events and sessions, not individual components
- **Session state transitions are automatic**: useSessionTimeouts ensures periodic state machine updates
- **Token managed separately**: Authentication token kept in localStorage, accessed by useWebSocket and TokenForm
- **Virtual scrolling is view layer**: EventStream owns rendering; formatting utilities are pure dependencies
- **Formatting utilities are pure**: No side effects, no state, pure input → output transformations

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

**Message Format** (server → client, text frames):
```json
{
  "id": "evt_...",
  "source": "monitor-1",
  "timestamp": "2026-02-02T14:30:00Z",
  "type": "tool",
  "payload": { ... }
}
```

### Client WebSocket Hook Contract (Phase 7)

**TypeScript Interface**:
```typescript
export interface UseWebSocketReturn {
  readonly connect: () => void;      // Manually establish connection
  readonly disconnect: () => void;   // Gracefully disconnect
  readonly isConnected: boolean;     // Current connection status
}

export function useWebSocket(url?: string): UseWebSocketReturn
```

**Usage Pattern**:
```tsx
const { connect, disconnect, isConnected } = useWebSocket();

// On mount
useEffect(() => {
  connect();
  return () => disconnect();
}, [connect, disconnect]);
```

### Client Event Store Contract (Phase 10)

**TypeScript Interface**:
```typescript
export interface EventStore {
  readonly status: ConnectionStatus;
  readonly events: readonly VibeteaEvent[];
  readonly sessions: Map<string, Session>;

  readonly addEvent: (event: VibeteaEvent) => void;
  readonly setStatus: (status: ConnectionStatus) => void;
  readonly clearEvents: () => void;
  readonly updateSessionStates: () => void;  // Phase 10 NEW
}

export const useEventStore = create<EventStore>()((set) => ({ ... }));
```

**Selective Subscription Pattern**:
```tsx
// Only re-render when sessions change
const sessions = useEventStore((state) => state.sessions);

// Only re-render when events change
const events = useEventStore((state) => state.events);

// Get actions (don't trigger re-renders)
const addEvent = useEventStore((state) => state.addEvent);
const updateSessionStates = useEventStore((state) => state.updateSessionStates);
```

### Client Session Timeouts Contract (Phase 10)

**TypeScript Interface**:
```typescript
export function useSessionTimeouts(): void
```

**Usage Pattern**:
```tsx
// Call once at app root level
export default function App() {
  useSessionTimeouts();
  return <Dashboard />;
}
```

### Client SessionOverview Component Contract (Phase 10)

**TypeScript Interface**:
```typescript
interface SessionOverviewProps {
  readonly className?: string;
  readonly onSessionClick?: (sessionId: string) => void;
}

export function SessionOverview(props: SessionOverviewProps): JSX.Element
```

### Client Heatmap Component Contract (Phase 9)

**TypeScript Interface**:
```typescript
interface HeatmapProps {
  readonly className?: string;
  readonly onCellClick?: (startTime: Date, endTime: Date) => void;
}

export function Heatmap(props: HeatmapProps): JSX.Element
```

### Client EventStream Component Contract (Phase 8)

**TypeScript Interface**:
```typescript
interface EventStreamProps {
  readonly className?: string;  // Optional custom CSS classes
}

export function EventStream({ className = '' }: EventStreamProps): JSX.Element
```

**Helper Function Contracts** (Phase 8):
```typescript
function formatTimestamp(timestamp: string): string  // RFC3339 → HH:MM:SS
function getEventDescription(event: VibeteaEvent): string  // Event → human text
function handleScroll(): void  // Detect user scroll position
function handleJumpToLatest(): void  // Jump to bottom and reset state
```

### Client Formatting Utilities Contract (Phase 8)

**TypeScript Interfaces**:
```typescript
export function formatTimestamp(timestamp: string): string  // → "HH:MM:SS"
export function formatTimestampFull(timestamp: string): string  // → "YYYY-MM-DD HH:MM:SS"
export function formatRelativeTime(timestamp: string, now?: Date): string  // → "5m ago"
export function formatDuration(milliseconds: number): string  // → "1h 30m"
export function formatDurationShort(milliseconds: number): string  // → "1:30:00"
```

**Error Handling**:
- All functions return appropriate fallback strings for invalid input
- No exceptions thrown
- Pure functions with no side effects

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
| **Client WebSocket Instance** | useWebSocket Hook (ref) | Single WS per hook instance | Hook lifetime |
| **Client Events** | Zustand store | Event buffer + session derivation | Client session lifetime |
| **Client Connection Status** | Zustand store | Discrete state enum | Until disconnect |
| **Client Session States** | Zustand store with state machine | Active/Inactive/Ended/Removed transitions | Sessions duration + 30min |
| **Client Authentication Token** | Browser localStorage | Persisted between sessions | User clears or overwrites |
| **Client Virtual Scrolling** | EventStream (local state) | isAutoScrollEnabled, newEventCount | Component lifetime |
| **Client Event Display Order** | EventStream (computed) | Reversed from store (newest-first internally) | Component render |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Error Handling** | Result types with ErrorResponse JSON | `routes.rs` (error_to_response pattern) |
| **Authentication** | Ed25519 for monitors, bearer tokens for clients | `auth.rs` (monitors), client localStorage (clients) |
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
| **Client Connection Management** | WebSocket reconnection with exponential backoff | `useWebSocket.ts` (Phase 7) |
| **Client State Persistence** | Token storage in localStorage | `TokenForm.tsx` (Phase 7) |
| **Client UI Rendering** | Selective Zustand subscriptions | `ConnectionStatus.tsx`, `TokenForm.tsx` (Phase 7) |
| **Session State Machine** | Automatic state transitions with time thresholds | `useEventStore.ts`, `useSessionTimeouts.ts` (Phase 10) |
| **Session Display** | Activity indicators and status badges | `SessionOverview.tsx` (Phase 10) |
| **Activity Visualization** | Heatmap grid with color intensity mapping | `Heatmap.tsx` (Phase 9) |
| **Virtual Scrolling Rendering** | @tanstack/react-virtual for 1000+ item lists | `EventStream.tsx` (Phase 8) |
| **Auto-Scroll Management** | Scroll position tracking and auto-scroll toggle | `EventStream.tsx` (Phase 8) |
| **Timestamp Formatting** | RFC3339 → display formats with fallbacks | `formatting.ts` (Phase 8) |

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

**Client Tests**: Located in `client/src/__tests__/`
- Event type guard tests (events.test.ts)
- WebSocket hook tests (useWebSocket integration, reconnection logic)
- Event store tests (useEventStore session aggregation, state machine)
- Session timeout tests (useSessionTimeouts state transitions)
- Session overview tests (SessionOverview component rendering)
- Component tests (ConnectionStatus, TokenForm, Heatmap)
- Virtual scrolling tests (EventStream rendering and auto-scroll)
- Formatting utility tests (timestamp and duration formatting)

**Integration Tests**: Located in `server/tests/unsafe_mode_test.rs`
- End-to-end scenarios in unsafe mode (auth disabled)

**Running Tests**:
```bash
cargo test --package vibetea-server  # All server tests
cargo test --package vibetea-server routes  # Route tests only
cargo test --package vibetea-monitor  # All monitor tests
cargo test --package vibetea-monitor crypto  # Crypto tests only
cargo test --package vibetea-monitor privacy  # Privacy tests only
npm test --prefix client  # Client tests
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
