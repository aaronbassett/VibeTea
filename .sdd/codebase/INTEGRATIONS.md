# External Integrations

**Status**: Phase 7 Implementation - Client WebSocket connection, token management, connection status UI
**Last Updated**: 2026-02-02

## Summary

VibeTea is designed as a distributed event system with three components:
- **Monitor**: Captures Claude Code session events from local JSONL files, applies privacy sanitization, signs with Ed25519, and transmits to server via HTTP
- **Server**: Receives, validates, verifies Ed25519 signatures, and broadcasts events via WebSocket
- **Client**: Subscribes to server events via WebSocket for visualization with token-based authentication

All integrations use standard protocols (HTTPS, WebSocket) with cryptographic message authentication and privacy-by-design data handling.

## File System Integration

### Claude Code JSONL Files

**Source**: `~/.claude/projects/**/*.jsonl`
**Format**: JSON Lines (one JSON object per line)
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Parser Location**: `monitor/src/parser.rs` (SessionParser, ParsedEvent, ParsedEventKind)
**Watcher Location**: `monitor/src/watcher.rs` (FileWatcher, WatchEvent)
**Privacy Pipeline**: `monitor/src/privacy.rs` (PrivacyConfig, PrivacyPipeline, extract_basename)

**Privacy-First Approach**:
- Only metadata extracted: tool names, timestamps, file basenames
- Never processes code content, prompts, or assistant responses
- File path parsing for project name extraction (slugified format)
- All event payloads pass through PrivacyPipeline before transmission

**Session File Structure**:
```
~/.claude/projects/<project-slug>/<session-uuid>.jsonl
```

**Supported Event Types** (from Claude Code JSONL):
| Claude Code Type | Parsed As | VibeTea Event | Fields Extracted |
|------------------|-----------|---------------|------------------|
| `assistant` with `tool_use` | Tool invocation | ToolStarted | tool name, context |
| `progress` with `PostToolUse` | Tool completion | ToolCompleted | tool name, success |
| `user` | User activity | Activity | timestamp only |
| `summary` | Session end marker | Summary | session metadata |
| File creation | Session start | SessionStarted | project from path |

**Watcher Behavior**:
- Monitors `~/.claude/projects/` directory recursively
- Detects file creation, modification, deletion events
- Maintains position map for efficient tailing (no re-reading)
- Emits WatchEvent::FileCreated, WatchEvent::LinesAdded, WatchEvent::FileRemoved
- Automatic cleanup of removed files from tracking state

**Configuration** (`monitor/src/config.rs`):
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | Claude Code directory to monitor |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated file extensions to watch |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |

## Privacy & Data Sanitization

### Privacy Pipeline Architecture

**Location**: `monitor/src/privacy.rs` (1039 lines)

**Core Components**:

1. **PrivacyConfig** - Configuration management
   - Optional extension allowlist (e.g., `.rs`, `.ts`)
   - Loaded from `VIBETEA_BASENAME_ALLOWLIST` environment variable
   - Supports comma-separated format: `.rs,.ts,.md` or `rs,ts,md` (auto-dots)
   - Whitespace-tolerant: ` .rs , .ts ` normalized to `[".rs", ".ts"]`
   - Empty entries filtered: `.rs,,.ts,,,` becomes `[".rs", ".ts"]`

2. **PrivacyPipeline** - Event sanitization processor
   - Processes EventPayload before transmission to server
   - Strips sensitive contexts from dangerous tools
   - Extracts basenames from file paths
   - Applies extension allowlist filtering
   - Neutralizes session summary text

3. **extract_basename()** - Path safety function
   - Converts full paths to secure basenames
   - Handles Unix: `/home/user/src/auth.ts` → `auth.ts`
   - Handles Windows: `C:\Users\user\src\auth.ts` → `auth.ts`
   - Handles relative: `src/auth.ts` → `auth.ts`
   - Returns `None` for invalid/empty paths

**Sensitive Tools** (context always stripped):
- `Bash` - Commands may contain secrets, passwords, API keys
- `Grep` - Patterns reveal what user is searching for
- `Glob` - File patterns reveal project structure
- `WebSearch` - Queries reveal user intent
- `WebFetch` - URLs may contain sensitive parameters

**Privacy Processing Rules**:

| Payload Type | Processing |
|--------------|-----------|
| Session | Pass through (project already sanitized at parse time) |
| Activity | Pass through unchanged |
| Tool (sensitive) | Context set to `None` |
| Tool (other) | Context → basename, apply allowlist, pass if allowed else `None` |
| Agent | Pass through unchanged |
| Summary | Summary text replaced with "Session ended" |
| Error | Pass through (category already sanitized) |

**Extension Allowlist Filtering**:
- When `VIBETEA_BASENAME_ALLOWLIST` is not set: All extensions allowed
- When set to `.rs,.ts`: Only `.rs` and `.ts` files transmitted; others filtered to `None`
- If no extension and allowlist set: Context filtered to `None`
- Examples:
  - `file.rs` with allowlist `.rs,.ts` → ALLOWED
  - `file.py` with allowlist `.rs,.ts` → FILTERED
  - `Makefile` with allowlist `.rs,.ts` → FILTERED (no extension)

**Example Privacy Processing**:
```
Input:  Tool { context: Some("/home/user/project/src/auth.rs"), tool: "Read", ... }
Output: Tool { context: Some("auth.rs"), tool: "Read", ... }

Input:  Tool { context: Some("rm -rf /home"), tool: "Bash", ... }
Output: Tool { context: None, tool: "Bash", ... }  # Sensitive tool

Input:  Tool { context: Some("/home/user/config.py"), tool: "Read", allowlist: [.rs,.ts] }
Output: Tool { context: None, tool: "Read", ... }  # Filtered by allowlist
```

### Privacy Test Suite

**Location**: `monitor/tests/privacy_test.rs` (951 lines)

**Coverage**: 18+ comprehensive privacy compliance tests
**Validates**: Constitution I (Privacy by Design)

**Test Categories**:
1. **Path Sanitization**
   - No full paths in output (Unix, Windows, relative)
   - Basenames correctly extracted
   - Hidden files handled

2. **Sensitive Tool Stripping**
   - Bash commands removed entirely
   - Grep patterns omitted
   - Glob patterns stripped
   - WebSearch queries removed
   - WebFetch URLs removed

3. **Content Stripping**
   - File contents never transmitted
   - Diffs excluded from payloads
   - Code excerpts removed

4. **Prompt/Response Stripping**
   - User prompts not included
   - Assistant responses excluded
   - Message content sanitized

5. **Command Argument Removal**
   - Arguments separated from descriptions
   - Descriptions allowed for Bash context
   - Actual commands never sent

6. **Summary Neutralization**
   - Summary text set to generic "Session ended"
   - Original text discarded
   - No content leakage

7. **Extension Allowlist Filtering**
   - Correct files allowed through
   - Disallowed extensions filtered
   - No-extension files handled properly

8. **Sensitive Pattern Detection**
   - Path patterns never appear (e.g., `/home/`, `/Users/`, `C:\`)
   - Command patterns removed (e.g., `rm -rf`, `sudo`, `curl -`, `Bearer`)
   - Credentials not transmitted

## Cryptographic Authentication & Key Management

### Phase 6: Monitor Cryptographic Operations

**Module Location**: `monitor/src/crypto.rs` (438 lines)

**Crypto Module Features**:

1. **Keypair Generation**
   - `Crypto::generate()` creates new Ed25519 keypair
   - Uses OS cryptographically secure RNG via `rand` crate
   - Returns Crypto struct managing SigningKey

2. **Key Persistence**
   - `save(dir)` writes keypair to files
   - Private key: `key.priv` (raw 32-byte seed, permissions 0600)
   - Public key: `key.pub` (base64-encoded, permissions 0644)
   - Creates directory if not present
   - Error on invalid file permissions (Unix)

3. **Key Loading**
   - `load(dir)` reads existing keypair
   - Validates private key is exactly 32 bytes
   - Returns CryptoError if format invalid
   - Reconstructs SigningKey from seed bytes

4. **Key Existence Check**
   - `exists(dir)` checks if private key file present
   - Used to prevent accidental overwrite

5. **Public Key Export**
   - `public_key_base64()` returns base64-encoded public key
   - Format suitable for `VIBETEA_PUBLIC_KEYS` environment variable
   - Derived from SigningKey via VerifyingKey

6. **Event Signing**
   - `sign(message)` returns base64-encoded Ed25519 signature
   - Message is JSON-encoded event payload (bytes)
   - Signature verifiable by server with public key
   - Uses RFC 8032 compliant signing via ed25519-dalek

**CryptoError Types**:
- `Io` - File system errors
- `InvalidKey` - Seed not 32 bytes or malformed
- `Base64` - Public key decoding error
- `KeyExists` - Files already present (can be overwritten)

**File Locations** (configurable):
- Default key directory: `~/.vibetea/`
- Override with `VIBETEA_KEY_PATH` environment variable
- Private key: `{key_dir}/key.priv`
- Public key: `{key_dir}/key.pub`

### Monitor → Server Authentication

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Ed25519 digital signatures | Rust `ed25519-dalek` crate |
| **Protocol** | HTTPS POST with signed payload | Event signatures in X-Signature header |
| **Key Management** | Source-specific public key registration | `VIBETEA_PUBLIC_KEYS` env var |
| **Key Format** | Base64-encoded Ed25519 public keys | `source1:pubkey1,source2:pubkey2` |
| **Verification** | Constant-time comparison using `subtle` crate | `server/src/auth.rs` |
| **Flow** | Monitor signs event → Server validates signature | `server/src/auth.rs`, `server/src/routes.rs` |
| **Fallback** | Unsafe no-auth mode (dev only) | `VIBETEA_UNSAFE_NO_AUTH=true` |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_PUBLIC_KEYS` (required unless unsafe mode enabled)
- Parses `VIBETEA_UNSAFE_NO_AUTH` (dev-only authentication bypass)
- Validates on every server startup with comprehensive error messages
- Supports multiple comma-separated source:key pairs

**Example Key Format**:
```
VIBETEA_PUBLIC_KEYS=monitor-prod:dGVzdHB1YmtleTEx,monitor-dev:dGVzdHB1YmtleTIy
```

**Implementation Details**:
- Uses `HashMap<String, String>` to map source_id to base64-encoded keys
- Public keys stored in plain text (no decryption needed)
- Empty public_keys map allowed if unsafe_no_auth is enabled
- Error handling with ConfigError enum for missing/invalid formats
- Constant-time comparison prevents timing attacks on signature verification

**Signature Verification Process** (`server/src/auth.rs`):
- Decode base64 signature from X-Signature header
- Decode base64 public key from configuration
- Extract Ed25519 VerifyingKey from public key bytes
- Use `ed25519_dalek::Signature::verify()` for verification
- Apply `subtle::ConstantTimeEq` to compare results

### Client Authentication (Server → Client)

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Bearer token in WebSocket headers | Static token per deployment |
| **Protocol** | WebSocket upgrade with `Authorization: Bearer <token>` | Client sends on connect |
| **Token Type** | Opaque string (no expiration in Phase 4) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| **Scope** | All clients use the same token | No per-user differentiation |
| **Validation** | Server-side validation only | In-memory, no persistence |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_SUBSCRIBER_TOKEN` (required unless unsafe mode enabled)
- Token required for all WebSocket connections
- No token refresh mechanism in Phase 5
- Stored as `Option<String>` in Config struct

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

## HTTP Sender & Event Transmission

### Phase 6: Event Sender Module

**Module Location**: `monitor/src/sender.rs` (544 lines)

**Sender Features**:

1. **HTTP Client Configuration**
   - Built with `reqwest` Client
   - Connection pooling: 10 max idle connections per host
   - Request timeout: 30 seconds
   - Automatic redirect handling

2. **Event Buffering**
   - VecDeque-based buffer with FIFO eviction
   - Default capacity: 1000 events
   - Configurable via `buffer_size` parameter
   - Tracks buffer overflow events with warnings
   - Supports queuing before sending

3. **Exponential Backoff Retry**
   - Initial delay: 1 second
   - Maximum delay: 60 seconds
   - Jitter: ±25% per attempt
   - Max retry attempts: 10 per batch
   - Resets on successful send

4. **Rate Limit Handling**
   - Recognizes HTTP 429 (Too Many Requests)
   - Reads `Retry-After` header from server
   - Respects server-provided delay
   - Falls back to exponential backoff if no header

5. **Event Signing**
   - Signs JSON event payload with Ed25519
   - X-Signature header contains base64-encoded signature
   - X-Source-ID header contains monitor source identifier
   - Compatible with server `auth.rs` verification

6. **Batch Sending**
   - `send_batch()` for efficient transmission
   - Single HTTP request with event array or single event
   - JSON request body with event(s)
   - 202 Accepted response expected

7. **Buffer Management**
   - `queue(event)` - Add to buffer
   - `flush()` - Send all buffered events
   - `send(event)` - Send single event immediately
   - `buffer_len()` - Current buffer size
   - `is_empty()` - Check if buffer empty

8. **Graceful Shutdown**
   - `shutdown(timeout)` - Flushes remaining events
   - Returns count of unflushed events
   - Waits for timeout before giving up
   - Allows time for final retry attempts

**SenderConfig**:
```rust
pub struct SenderConfig {
    pub server_url: String,     // e.g., https://vibetea.fly.dev
    pub source_id: String,      // e.g., hostname
    pub buffer_size: usize,     // e.g., 1000
}
```

**SenderError Types**:
- `Http` - HTTP client error (network, TLS, etc.)
- `ServerError { status, message }` - Non-202 response
- `AuthFailed` - 401 Unauthorized (invalid signature)
- `RateLimited { retry_after_secs }` - 429 with delay
- `BufferOverflow { evicted_count }` - Events evicted
- `MaxRetriesExceeded { attempts }` - All retries failed
- `Json` - Event serialization error

**Connection Details**:
- Server URL from `VIBETEA_SERVER_URL` env var
- POST to `{server_url}/events` endpoint
- HTTPS recommended for production
- HTTP allowed for local development

## CLI & Key Management

### Phase 6: Monitor CLI

**Module Location**: `monitor/src/main.rs` (301 lines)

**Command Structure**:

1. **init Command**: Generate Ed25519 keypair
   ```bash
   vibetea-monitor init [--force]
   ```
   - Generates new keypair using `Crypto::generate()`
   - Saves to `~/.vibetea/` or `VIBETEA_KEY_PATH`
   - Displays public key for server registration
   - Prompts for overwrite confirmation (unless --force)
   - Provides copy-paste ready export command

2. **run Command**: Start monitor daemon
   ```bash
   vibetea-monitor run
   ```
   - Loads configuration from environment variables
   - Loads cryptographic keys from disk
   - Creates sender with buffering and retry
   - Initializes file watcher (future: Phase 7)
   - Waits for shutdown signal
   - Graceful shutdown with event flushing

3. **help Command**: Show documentation
   ```bash
   vibetea-monitor help
   vibetea-monitor --help
   vibetea-monitor -h
   ```
   - Displays usage information
   - Lists all available commands
   - Shows environment variables
   - Provides example commands

4. **version Command**: Show version
   ```bash
   vibetea-monitor version
   vibetea-monitor --version
   vibetea-monitor -V
   ```
   - Prints binary version from Cargo.toml

**CLI Features**:
- Manual argument parsing (no external CLI framework)
- Flag support: `--force`, `-f` for init overwrite
- Short and long option variants for help/version
- User prompts on stdout/stderr
- Structured error messages
- Exit codes: 0 on success, 1 on error

**Environment Variables Used**:

| Variable | Required | Default | Command |
|----------|----------|---------|---------|
| `VIBETEA_SERVER_URL` | Yes | - | run |
| `VIBETEA_SOURCE_ID` | No | hostname | run |
| `VIBETEA_KEY_PATH` | No | ~/.vibetea | init, run |
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | run |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | run |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | run |
| `RUST_LOG` | No | info | all |

**Logging**:
- Structured logging via `tracing` crate
- Environment-based filtering (`RUST_LOG`)
- JSON output support
- Logs configuration, key loading, shutdown events
- Info level by default

**Signal Handling**:
- Listens for SIGINT (Ctrl+C)
- Listens for SIGTERM on Unix
- Cross-platform support via `tokio::signal`
- Graceful shutdown sequence on signal

**Key Registration Workflow**:
1. User runs: `vibetea-monitor init`
2. Binary displays public key
3. User copies to: `export VIBETEA_PUBLIC_KEYS="...:<public_key>"`
4. User adds to server configuration
5. User runs: `vibetea-monitor run`

## Client-Side Integrations (Phase 7)

### Browser WebSocket Connection

**Module Location**: `client/src/hooks/useWebSocket.ts` (321 lines)

**WebSocket Hook Features**:

1. **Connection Management**
   - Establishes WebSocket connection to server
   - Validates token from localStorage before connecting
   - Tracks connection state: connecting, connected, reconnecting, disconnected
   - Provides manual `connect()` and `disconnect()` methods

2. **Auto-Reconnection**
   - Exponential backoff: 1s initial, 60s maximum
   - Jitter: ±25% randomization per attempt
   - Resets attempt counter on successful connection
   - Respects user's disconnect intent (no auto-reconnect after manual disconnect)

3. **Token Management**
   - Reads token from `localStorage` key: `vibetea_token`
   - Token set via TokenForm component
   - Returns error if token missing, prevents connection
   - Token passed as query parameter in WebSocket URL

4. **Event Processing**
   - Receives JSON-encoded VibeteaEvent messages
   - Validates message structure (id, source, timestamp, type, payload)
   - Dispatches valid events to Zustand store via `addEvent()`
   - Silently discards invalid/unparseable messages

5. **Integration with Event Store**
   - `useEventStore` for state management
   - `addEvent(event)` - Add event to store
   - `setStatus(status)` - Update connection status
   - Status field synced with component state

6. **Error Handling**
   - Logs connection errors to console
   - Logs message parsing failures
   - Graceful handling of malformed messages
   - No crashes on connection errors

7. **Cleanup & Lifecycle**
   - Proper cleanup on component unmount
   - Clears pending reconnection timeouts
   - Closes WebSocket connection
   - Prevents memory leaks

**Hook Return Type**:
```typescript
export interface UseWebSocketReturn {
  readonly connect: () => void;          // Manually initiate connection
  readonly disconnect: () => void;        // Manually disconnect
  readonly isConnected: boolean;          // Connection state
}
```

**Constants**:
- `TOKEN_STORAGE_KEY`: `"vibetea_token"` (matches TokenForm)
- `INITIAL_BACKOFF_MS`: 1000ms
- `MAX_BACKOFF_MS`: 60000ms
- `JITTER_FACTOR`: 0.25 (25%)

**Default WebSocket URL**:
- Protocol: `ws://` (HTTP) or `wss://` (HTTPS) based on location protocol
- Host: Current browser location host
- Path: `/ws`
- Query param: `token=<token_from_localStorage>`

### Connection Status Component

**Module Location**: `client/src/components/ConnectionStatus.tsx` (106 lines)

**Features**:

1. **Visual Indicator**
   - Colored dot (2.5x2.5 rem) showing connection state
   - Green (#22c55e) for connected
   - Yellow (#eab308) for connecting/reconnecting
   - Red (#ef4444) for disconnected
   - Uses Tailwind CSS classes

2. **Optional Status Label**
   - Shows text status if `showLabel` prop is true
   - Labels: "Connected", "Connecting", "Reconnecting", "Disconnected"
   - Styled as small gray text
   - Dark mode support

3. **Performance Optimization**
   - Selective Zustand subscription: only re-renders when status changes
   - Uses selector to extract only status field
   - Prevents re-renders on other store updates

4. **Accessibility**
   - `role="status"` for semantic meaning
   - `aria-label` with full status description
   - Visual indicator marked as `aria-hidden="true"`
   - Screen reader friendly

5. **Component Props**:
```typescript
interface ConnectionStatusProps {
  readonly showLabel?: boolean;    // Show status text (default: false)
  readonly className?: string;     // Additional CSS classes
}
```

6. **Styling**
   - Flexbox layout with gap-2
   - Responsive and composable
   - Integrates seamlessly with other UI elements
   - Dark mode aware styling

### Token Form Component

**Module Location**: `client/src/components/TokenForm.tsx` (201 lines)

**Features**:

1. **Token Input & Storage**
   - Password input field for secure token entry
   - Persists token to `localStorage` via `TOKEN_STORAGE_KEY`
   - Matches key used by `useWebSocket` hook
   - Non-empty validation before saving

2. **Button Controls**
   - **Save Token** button
     - Disabled when input is empty
     - Saves trimmed token to localStorage
     - Resets input field after save
     - Invokes optional callback
   - **Clear Token** button
     - Disabled when no token saved
     - Removes token from localStorage
     - Resets input and status
     - Invokes optional callback

3. **Status Indicator**
   - Green dot when token saved
   - Gray dot when no token saved
   - Text shows "Token saved" or "No token saved"
   - Updates in real-time as user changes

4. **Cross-Window Synchronization**
   - Listens to `storage` events
   - Detects token changes from other tabs/windows
   - Updates status accordingly
   - Handles multi-tab scenarios

5. **Component Props**:
```typescript
interface TokenFormProps {
  readonly onTokenChange?: () => void;  // Called when token saved/cleared
  readonly className?: string;          // Additional CSS classes
}
```

6. **Callback Support**
   - `onTokenChange()` invoked on save or clear
   - Allows parent to reconnect WebSocket
   - Enables form submission handlers

7. **Accessibility**
   - Label element linked to input
   - `aria-describedby` for status association
   - Status region with `aria-live="polite"`
   - Semantic form structure
   - Proper button states for disabled

8. **Styling**
   - Tailwind CSS dark mode (bg-gray-800, text-white)
   - Responsive layout
   - Visual feedback on focus (blue ring)
   - Disabled state styling (gray background, cursor not-allowed)
   - Button hover effects

9. **Behavior**
   - Stores token under key `vibetea_token` (matches useWebSocket)
   - Input placeholder changes based on save state
   - Form submission on button click or Enter key
   - Input cleared after successful save
   - Token masked as password field

### Integration Flow (Phase 7)

**Typical Usage Pattern**:
```typescript
function App() {
  const { connect, disconnect, isConnected } = useWebSocket();

  return (
    <div>
      <TokenForm
        onTokenChange={() => {
          // Reconnect when token changes
          connect();
        }}
      />
      <ConnectionStatus showLabel />
      {/* Rest of app */}
    </div>
  );
}
```

**Connection Flow**:
1. User enters token in TokenForm
2. Token saved to `localStorage.vibetea_token`
3. User clicks connect (or app calls `connect()`)
4. `useWebSocket` reads token from localStorage
5. WebSocket connects to server with token in URL
6. Server validates token and upgrades connection
7. ConnectionStatus shows "Connected" with green dot
8. Events flow from server to client via WebSocket
9. Events added to Zustand store via `addEvent()`

## Event Validation & Types

### Shared Event Schema

All components use a unified event schema for message passing:

**Event Structure** (from `server/src/types.rs`):
```
Event {
  id: String,           // evt_<20-char-alphanumeric>
  source: String,       // Source identifier (e.g., hostname)
  timestamp: DateTime,  // RFC 3339 UTC
  type: EventType,      // session, activity, tool, agent, summary, error
  payload: EventPayload // Type-specific data (EventPayload enum)
}
```

**Supported Event Types**:
| Type | Payload Fields | Purpose |
|------|----------------|---------|
| `session` | sessionId, action (started/ended), project | Track session lifecycle |
| `activity` | sessionId, project (optional) | Heartbeat events |
| `tool` | sessionId, tool, status (started/completed), context, project | Tool usage tracking |
| `agent` | sessionId, state | Agent state changes |
| `summary` | sessionId, summary | End-of-session summary |
| `error` | sessionId, category | Error reporting |

**Schema Locations**:
- Rust types: `server/src/types.rs`, `monitor/src/types.rs`
- TypeScript types: `client/src/types/events.ts`
- Event validation: Serde deserialization with untagged union handling

**Phase 4 Parser Integration** (`monitor/src/parser.rs`):
- Maps Claude Code JSONL → ParsedEvent (privacy-first extraction)
- SessionParser converts ParsedEventKind → VibeTea Event types
- Tool invocations tracked with extracted context (file basenames)
- Session lifecycle inferred from JSONL file creation/removal and summary markers

**Phase 5 Privacy Integration** (`monitor/src/privacy.rs`):
- ProcessedEvent payloads through PrivacyPipeline before transmission
- Sensitive contexts stripped according to tool type
- Paths reduced to basenames with extension filtering
- Summary text neutralized to privacy-safe message

**Phase 6 Signing Integration** (`monitor/src/crypto.rs` + `monitor/src/sender.rs`):
- Events signed with Ed25519 private key
- Signature in X-Signature header (base64-encoded)
- Server verifies using registered public key
- Constant-time comparison prevents timing attacks

**Phase 7 Client Reception** (new):
- `useWebSocket` receives and validates events
- TypeScript type guards ensure type safety
- Zustand store aggregates events by session
- Components render session data from store

### Client Event Store Integration

**Location**: `client/src/hooks/useEventStore.ts`

**Zustand Store State**:
```typescript
export interface EventStore {
  status: ConnectionStatus;              // 'connecting' | 'connected' | 'disconnecting' | 'reconnecting'
  events: readonly VibeteaEvent[];       // Last 1000 events, newest first
  sessions: Map<string, Session>;        // Active sessions keyed by sessionId

  addEvent: (event: VibeteaEvent) => void;
  setStatus: (status: ConnectionStatus) => void;
  clearEvents: () => void;
}
```

**Event Processing**:
- FIFO eviction: Keeps last 1000 events, newest first
- Session aggregation: Derives Session objects from events
- Session status transitions: 'active' → 'ended' on summary event
- Event counting: Increments eventCount per session
- Project tracking: Updates project field if present in event payload

**Selector Utilities**:
- `selectEventsBySession(state, sessionId)` - Filter events by session
- `selectActiveSessions(state)` - Get sessions with status !== 'ended'
- `selectSession(state, sessionId)` - Get single session by ID

**Serialization Formats**

| Component | Format | Field Naming | Location |
|-----------|--------|--------------|----------|
| Server/Monitor | JSON (serde) | snake_case in payloads | Rust source |
| Client | TypeScript types | camelCase in UI/API | `client/src/types/events.ts` |
| Wire Protocol | JSON | Both (depends on layer) | Event payloads |
| Claude Code Files | JSONL | Mixed (JSON structure) | `~/.claude/projects/**/*.jsonl` |

## Network Communication

### Monitor → Server (Event Publishing)

**Endpoint**: `https://<server-url>/events`
**Method**: POST
**Authentication**: Ed25519 signature in X-Signature header (Phase 6)
**Content-Type**: application/json

**Flow**:
1. Monitor watches local JSONL files via file watcher
2. Parser extracts metadata from new/modified JSONL lines
3. Events processed through PrivacyPipeline (Phase 5)
4. Monitor signs event payload with Ed25519 private key (Phase 6)
5. Monitor POSTs signed event to server with X-Source-ID and X-Signature headers (Phase 6)
6. Server validates signature against registered public key
7. Server rate limits based on source ID (100 events/sec default)
8. Server broadcasts to all connected clients via WebSocket

**Rate Limiting** (`server/src/rate_limit.rs`):
- Token bucket algorithm per source
- 100.0 tokens/second refill rate (configurable)
- 100 token capacity (configurable)
- Exceeded limit returns 429 Too Many Requests with Retry-After header
- Automatic cleanup of inactive sources after 60 seconds

**Client Library**: `reqwest` crate (HTTP client)
**Configuration**: `monitor/src/config.rs`
- `VIBETEA_SERVER_URL` - Server endpoint (required)
- `VIBETEA_SOURCE_ID` - Source identifier for event attribution (default: hostname)
- Uses gethostname crate to get system hostname if not provided

**Phase 6 Enhancements**:
- Crypto module signs all events before transmission
- Sender module handles buffering, retry, rate limiting
- CLI allows easy key management and monitor startup

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request headers
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket connection with Bearer token
2. Server validates token and establishes connection
3. Server broadcasts events as they arrive from monitors
4. Optional: Server filters events based on query parameters (source, type, project)
5. Client processes and stores events in Zustand state via `addEvent()`
6. Client UI renders session information from state

**Server Broadcasting** (`server/src/broadcast.rs`):
- EventBroadcaster wraps tokio broadcast channel
- 1000-event capacity for burst handling
- Thread-safe, cloneable for sharing across handlers
- SubscriberFilter enables selective delivery by event type, source, project

**WebSocket Upgrade** (`server/src/routes.rs`):
- GET /ws endpoint handles upgrade requests
- Validates bearer token before upgrade
- Returns 101 Switching Protocols on success
- Returns 401 Unauthorized on token validation failure

**Client-Side Handling** (Phase 7):
- WebSocket proxy configured in `client/vite.config.ts` (target: ws://localhost:8080)
- State management via `useEventStore` hook (Zustand)
- Event type guards for safe type access in `client/src/types/events.ts`
- ConnectionStatus transitions: disconnected → connecting → connected → reconnecting
- Token management via `TokenForm` component
- Connection control via `useWebSocket` hook

**Connection Details**:
- Address/port: Configured via `PORT` environment variable (default: 8080)
- Persistent connection model
- Automatic reconnection with exponential backoff (Phase 7)
- No message queuing (direct streaming)
- Events processed with selective subscriptions to prevent unnecessary re-renders

### Monitor → File System (JSONL Watching)

**Target**: `~/.claude/projects/**/*.jsonl`
**Mechanism**: `notify` crate file system events (inotify/FSEvents)
**Update Strategy**: Incremental line reading with position tracking

**Flow**:
1. FileWatcher initialized with watch directory
2. Recursive file system monitoring begins
3. File creation detected → WatchEvent::FileCreated emitted
4. File modification detected → New lines read from position marker
5. Lines sent in WatchEvent::LinesAdded with accumulated lines
6. Position marker updated to avoid re-reading
7. File deletion detected → WatchEvent::FileRemoved emitted, cleanup position state

**Efficiency Features**:
- Position tracking prevents re-reading file content
- Only new lines since last position are extracted
- BufReader with Seek for efficient line iteration
- Arc<RwLock<>> for thread-safe concurrent access

## HTTP API Endpoints

### POST /events

**Purpose**: Ingest events from monitors

**Request Headers**:
| Header | Required | Value |
|--------|----------|-------|
| X-Source-ID | Yes | Monitor identifier (non-empty string) |
| X-Signature | No* | Base64-encoded Ed25519 signature (Phase 6) |
| Content-Type | Yes | application/json |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**Request Body**: Single Event or array of Events (JSON)

**Response Codes**:
- 202 Accepted - Events accepted and broadcasted
- 400 Bad Request - Invalid JSON or malformed events
- 401 Unauthorized - Missing/empty X-Source-ID or signature verification failed
- 429 Too Many Requests - Rate limit exceeded (includes Retry-After header)

**Flow** (`server/src/routes.rs`):
1. Extract X-Source-ID from headers
2. Check rate limit for source
3. If unsafe_no_auth is false, verify X-Signature against public key
4. Deserialize event(s) from body
5. Broadcast each event via EventBroadcaster
6. Return 202 Accepted

### GET /ws

**Purpose**: WebSocket subscription for event streaming

**Query Parameters**:
| Parameter | Required | Example |
|-----------|----------|---------|
| token | No* | my-secret-token |
| source | No | monitor-1 |
| type | No | session |
| project | No | my-project |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**WebSocket Messages**: JSON-encoded Event objects (one per message)

**Response Codes**:
- 101 Switching Protocols - WebSocket upgrade successful
- 401 Unauthorized - Token validation failed

**Filtering** (`server/src/broadcast.rs`):
- SubscriberFilter applied if query parameters provided
- Matches event.event_type against type parameter
- Matches event.source against source parameter
- Matches event.payload.project against project parameter

### GET /health

**Purpose**: Health check and uptime reporting

**Response**:
```json
{
  "status": "ok",
  "uptime_secs": 3600
}
```

**Response Code**: 200 OK (always succeeds, no auth required)

## Development & Local Configuration

### Local Server Setup

**Environment Variables**:
```bash
PORT=8080                                        # Server port
VIBETEA_PUBLIC_KEYS=localhost:cHVia2V5MQ==      # Monitor public key (base64)
VIBETEA_SUBSCRIBER_TOKEN=dev-token-secret        # Client WebSocket token
VIBETEA_UNSAFE_NO_AUTH=false                     # Set true to disable all auth
RUST_LOG=debug                                   # Logging level
```

**Unsafe Development Mode**:
When `VIBETEA_UNSAFE_NO_AUTH=true`:
- All monitor authentication is bypassed (X-Signature ignored)
- All client authentication is bypassed (token parameter ignored)
- Suitable for local development only
- Never use in production
- Warning logged on startup when enabled

**Validation Behavior**:
- With unsafe_no_auth=false: Requires both VIBETEA_PUBLIC_KEYS and VIBETEA_SUBSCRIBER_TOKEN
- With unsafe_no_auth=true: Both auth variables become optional
- PORT defaults to 8080 if not specified
- Invalid PORT formats rejected with ParseIntError

### Local Monitor Setup

**Environment Variables**:
```bash
VIBETEA_SERVER_URL=http://localhost:8080         # Server endpoint
VIBETEA_SOURCE_ID=my-monitor                     # Custom source identifier
VIBETEA_KEY_PATH=~/.vibetea                      # Directory with private/public keys
VIBETEA_CLAUDE_DIR=~/.claude                     # Claude Code directory to watch
VIBETEA_BUFFER_SIZE=1000                         # Event buffer capacity
VIBETEA_BASENAME_ALLOWLIST=.ts,.tsx,.rs          # Optional file extension filter (Phase 5)
RUST_LOG=debug                                   # Logging level
```

**Configuration Loading**: `monitor/src/config.rs`
- Required: VIBETEA_SERVER_URL (no default)
- Optional defaults use directories crate for platform-specific paths
- Home directory determined via BaseDirs::new()
- Hostname fallback when VIBETEA_SOURCE_ID not set
- Buffer size parsed as usize, validated for positive integers
- Allowlist split by comma, whitespace trimmed, empty entries filtered

**Key Management** (Phase 6):
- `vibetea-monitor init` generates Ed25519 keypair
- Keys stored in ~/.vibetea/ or VIBETEA_KEY_PATH
- Private key: key.priv (0600 permissions)
- Public key: key.pub (0644 permissions)
- Public key must be registered with server via VIBETEA_PUBLIC_KEYS

**Privacy Configuration** (Phase 5):
- `VIBETEA_BASENAME_ALLOWLIST` loads into PrivacyConfig via `from_env()`
- Format: `.rs,.ts,.md` or `rs,ts,md` (dots auto-added)
- Whitespace tolerance: ` .rs , .ts ` → `[".rs", ".ts"]`
- Empty entries filtered: `.rs,,.ts,,,` → `[".rs", ".ts"]`
- When not set: All extensions allowed (default behavior)
- Applied during PrivacyPipeline event processing

**File System Monitoring**:
- Watches directory: VIBETEA_CLAUDE_DIR
- Monitors for file creation, modification, deletion, and directory changes
- Uses `notify` crate (version 8.0) for cross-platform inotify/FSEvents
- Optional extension filtering via VIBETEA_BASENAME_ALLOWLIST
- Phase 4: FileWatcher tracks position to efficiently tail JSONL files

**JSONL Parsing**:
- Phase 4: SessionParser extracts metadata from Claude Code JSONL
- Privacy-first: Never processes code content or prompts
- Tool tracking: Extracts tool name and context from assistant tool_use events
- Progress tracking: Detects tool completion from progress PostToolUse events

**Privacy Pipeline** (Phase 5):
- PrivacyPipeline processes all events before transmission
- PrivacyConfig loaded from `VIBETEA_BASENAME_ALLOWLIST`
- Sensitive tools stripped: Bash, Grep, Glob, WebSearch, WebFetch
- Paths reduced to basenames with extension allowlist filtering
- Summary text neutralized to "Session ended"

**Cryptographic Signing** (Phase 6):
- Crypto module signs all events with Ed25519 private key
- Signature sent in X-Signature header (base64-encoded)
- Monitor must be initialized before first run: `vibetea-monitor init`

**HTTP Transmission** (Phase 6):
- Sender module handles event buffering (1000 events default)
- Exponential backoff retry: 1s → 60s with ±25% jitter
- Rate limit handling: Respects 429 with Retry-After header
- Connection pooling: 10 max idle connections per host
- 30-second request timeout

### Local Client Setup

**Development Server**:
- Runs on port 5173 (Vite default)
- WebSocket proxy to localhost:8080

**Environment**: None required for local dev
- Token hardcoded in future phases
- Currently uses Vite proxy configuration

**Build Configuration**: `client/vite.config.ts`
```typescript
server: {
  proxy: {
    '/ws': {
      target: 'ws://localhost:8080',
      ws: true
    }
  }
}
```

**Vite Build Features**:
- React Fast Refresh via @vitejs/plugin-react
- Tailwind CSS integration via @tailwindcss/vite
- Brotli compression for production builds
- Code splitting: react-vendor, state, virtual chunks
- Target: ES2020

**Phase 7 Client Features**:
- Token management via TokenForm component
- Connection status visualization via ConnectionStatus component
- WebSocket connection management via useWebSocket hook
- Event display and session tracking via Zustand store
- localStorage persistence for authentication token

## Error Handling & Validation

### Server-Side Error Handling

**Error Types** (from `server/src/error.rs`):
- `ConfigError` - Configuration loading/validation failures
- `ServerError` - Runtime errors (Auth, Validation, RateLimit, WebSocket, Internal)

**Validation Points**:
1. Configuration validation on startup (`config.rs`)
   - Port number must be valid u16
   - If unsafe_no_auth is false, both public_keys and subscriber_token required
   - Public keys format: `source_id:pubkey` pairs
2. Event signature validation on POST (with constant-time comparison)
3. Event schema validation (serde untagged enum)
4. Bearer token validation on WebSocket connect

**Config Error Types** (comprehensive):
- MissingEnvVar(String) - Required variable not found
- InvalidFormat { var: String, message: String } - Format/parsing error
- InvalidPort(ParseIntError) - Port not valid u16
- ValidationError(String) - Config validation failed

**Auth Error Types** (`server/src/auth.rs`):
- UnknownSource(String) - Source not found in public keys
- InvalidSignature - Signature verification failed
- InvalidBase64(String) - Base64 decoding failed
- InvalidPublicKey - Malformed public key
- InvalidToken - Bearer token mismatch

### Monitor-Side Error Handling

**Error Types** (from `monitor/src/error.rs`):
- Configuration errors (missing env vars, invalid paths)
- File watching errors (permission denied, path not found)
- HTTP request errors (connection refused, timeout)
- Cryptographic errors (invalid private key)
- Phase 4: JSONL parsing errors (invalid JSON, malformed events)
- Phase 5: Privacy processing errors (path parsing failures)
- Phase 6: Key management errors (missing/invalid keys)
- Phase 6: HTTP sender errors (connection, rate limit, signature)

**Config Error Types**:
- MissingEnvVar(String) - VIBETEA_SERVER_URL required
- InvalidValue { key: String, message: String } - Invalid parsed value
- NoHomeDirectory - Cannot determine home directory

**Parser Error Types** (`monitor/src/parser.rs`):
- InvalidJson - Failed to parse JSONL line
- InvalidPath - Malformed file path format
- InvalidSessionId - UUID parsing failure

**Watcher Error Types** (`monitor/src/watcher.rs`):
- WatcherInit - File system watcher initialization failure
- Io - File system I/O errors
- DirectoryNotFound - Watch directory missing or inaccessible

**Crypto Error Types** (`monitor/src/crypto.rs` - Phase 6):
- Io - File system errors during key I/O
- InvalidKey - Key format invalid or wrong size
- Base64 - Public key base64 decoding error
- KeyExists - Key files already present (can overwrite)

**Sender Error Types** (`monitor/src/sender.rs` - Phase 6):
- Http - Network/HTTP client error
- ServerError - Non-202 response from server
- AuthFailed - 401 Unauthorized (signature/source mismatch)
- RateLimited - 429 Too Many Requests
- BufferOverflow - Events evicted due to full buffer
- MaxRetriesExceeded - All retry attempts exhausted
- Json - Event serialization failure

**Client-Side Error Handling** (Phase 7):
- WebSocket connection errors logged to console
- Message parsing failures handled gracefully
- Invalid events silently discarded
- No crashes on connection drops (auto-reconnect)
- Token missing handled with warning log

**Resilience**:
- Continues watching even if individual file operations fail
- Retries HTTP requests with exponential backoff (Phase 6)
- Logs errors via `tracing` crate with structured context
- Validates VIBETEA_BUFFER_SIZE as positive integer
- Graceful degradation on malformed JSONL lines
- Privacy processing failures logged without exposing sensitive data
- Sender buffers events if network unavailable, retries with backoff
- Client maintains event store even if connection drops

## File System Monitoring

### Monitor File Watching

**Library**: `notify` crate (version 8.0)
**Behavior**: Cross-platform file system events (inotify on Linux, FSEvents on macOS)

**Configuration**:
- Directory: `VIBETEA_CLAUDE_DIR` (default: `~/.claude`)
- Buffer capacity: `VIBETEA_BUFFER_SIZE` (default: 1000 events)
- Optional allowlist: `VIBETEA_BASENAME_ALLOWLIST` (comma-separated file patterns)

**Events Captured**:
- File creation, modification, deletion
- Directory changes
- Filtering based on file extension allowlist (if configured)

**Location**: `monitor/src/config.rs` and `monitor/src/main.rs`

**Phase 4 Enhancements** (`monitor/src/watcher.rs`):
- Position tracking for efficient file tailing
- Detects and emits only new lines appended to JSONL files
- Automatic cleanup of removed files from tracking state
- Thread-safe position map for concurrent access

## Logging & Observability

### Structured Logging

**Framework**: `tracing` + `tracing-subscriber`
**Configuration**: Environment variable `RUST_LOG`

**Features**:
- JSON output support (via `tracing-subscriber` with json feature)
- Environment-based filtering
- Structured context in logs

**Components**:
- Server: Logs configuration, connection events, errors, rate limiting
- Monitor: Logs file system events, HTTP requests, signing operations (Phase 6)
- Phase 4: Parser logs invalid JSONL events with context
- Phase 4: Watcher logs file tracking updates and position management
- Phase 5: Privacy pipeline logs filtering decisions and sensitive tool detection
- Phase 6: Crypto logs key generation, loading, and signature operations
- Phase 6: Sender logs buffering, retry, rate limit decisions
- Warning logged when VIBETEA_UNSAFE_NO_AUTH is enabled

**No External Service Integration** (Phase 5):
- Logs to stdout/stderr only
- Future: Integration with logging services (e.g., ELK, Datadog)

## Security Considerations

### Cryptographic Authentication

**Ed25519 Signatures** (Phase 6):
- Library: `ed25519-dalek` crate (version 2.1)
- Key generation: 32-byte seed via OS RNG
- Signature verification: Base64-encoded public keys per source
- Private key storage: User's filesystem (unencrypted)
- File permissions: 0600 (owner read/write only)
- Public key permissions: 0644 (owner read/write, others read)
- Timing attack prevention: `subtle::ConstantTimeEq` for comparison

**Security Implications**:
- Private keys must be protected with file permissions
- Public keys registered on server must match monitor's keys
- Signature validation prevents spoofed events
- Constant-time comparison prevents timing attacks on verification
- Ed25519 prevents signature forgery even if attacker has public key
- Phase 6: Enables cryptographic proof of event origin

### Token-Based Client Authentication

**Bearer Token**:
- Currently a static string per deployment
- No encryption in transit (relies on TLS via HTTPS)
- No expiration or refresh (Phase 5 limitation)

**Security Implications**:
- Token should be treated like a password
- Compromise affects all connected clients
- Future: Implement token rotation, per-user tokens
- localStorage exposure could compromise token

### Rate Limiting Security

**Token Bucket Protection**:
- Per-source rate limiting prevents single monitor from overwhelming server
- Default 100 events/second per source
- Automatic cleanup prevents memory leaks from zombie sources
- Retry-After header guides clients on backoff strategy

### Data in Transit

**TLS Encryption**:
- Production deployments use HTTPS (Monitor → Server)
- Production deployments use WSS (Server ↔ Client)
- Local development may use unencrypted HTTP/WS

### Privacy

**Claude Code JSONL** (Phase 4-5):
- Parser never extracts code content, prompts, or responses
- Only metadata stored: tool names, timestamps, file basenames
- File paths used only for project name extraction
- PrivacyPipeline (Phase 5) ensures sensitive data not transmitted:
  - Full paths reduced to basenames
  - Sensitive tool contexts always stripped
  - Extension allowlist filtering applied
  - Summary text neutralized
- Event contents never logged or stored unencrypted
- All transformations logged without revealing sensitive data

### Client-Side Security

**localStorage Token Storage** (Phase 7):
- Token persisted to browser localStorage
- Accessible to any script running in same origin
- XSS vulnerability could expose token
- Cross-site scripting protection recommended
- Consider HTTPOnly cookies as future enhancement

**WebSocket Token Transmission** (Phase 7):
- Token passed as query parameter in URL
- Visible in browser network tab
- Should use WSS (WebSocket Secure) in production
- Token in header would be preferable (future enhancement)

### Sender Security

**HTTP Client Security** (Phase 6):
- Connection pooling prevents connection-based attacks
- Timeout prevents hanging connections
- Exponential backoff prevents amplification attacks
- No credentials in URLs or request bodies (signature-based only)
- X-Signature header prevents man-in-the-middle spoofing
- Event buffering prevents replay of failed requests (forward secrecy)

## Future Integration Points

### Planned (Not Yet Integrated)

- **Main Event Loop**: Integrate file watcher, parser, privacy pipeline, and HTTP sender (Phase 6 in progress)
- **Database/Persistence**: Store events beyond memory (Phase 5+)
- **Authentication Providers**: OAuth2, API key rotation (Phase 5+)
- **Monitoring Services**: Datadog, New Relic, CloudWatch (Phase 5+)
- **Message Queues**: Redis, RabbitMQ for event buffering (Phase 5+)
- **Webhooks**: External service notifications (Phase 6+)
- **Background Task Spawning**: Async watcher and sender pipeline (Phase 6+)
- **Session Persistence**: Store events in database for replay (Phase 7+)
- **Advanced Authentication**: Per-user tokens, OAuth2 flows (Phase 7+)
- **Event Search/Filtering**: Full-text search and advanced filtering UI (Phase 7+)

## Configuration Quick Reference

### Server Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `PORT` | number | 8080 | No | HTTP server listening port |
| `VIBETEA_PUBLIC_KEYS` | string | - | Yes* | Source public keys (source:key,source:key) |
| `VIBETEA_SUBSCRIBER_TOKEN` | string | - | Yes* | Bearer token for clients |
| `VIBETEA_UNSAFE_NO_AUTH` | boolean | false | No | Disable all authentication (dev only) |
| `RUST_LOG` | string | info | No | Logging level (debug, info, warn, error) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | string | - | Yes | Server endpoint (e.g., https://vibetea.fly.dev) |
| `VIBETEA_SOURCE_ID` | string | hostname | No | Monitor identifier |
| `VIBETEA_KEY_PATH` | string | ~/.vibetea | No | Directory with key.priv/key.pub |
| `VIBETEA_CLAUDE_DIR` | string | ~/.claude | No | Claude Code directory to watch |
| `VIBETEA_BUFFER_SIZE` | number | 1000 | No | Event buffer capacity |
| `VIBETEA_BASENAME_ALLOWLIST` | string | - | No | Comma-separated file extensions to watch (Phase 5) |
| `RUST_LOG` | string | info | No | Logging level (debug, info, warn, error) |

### Client Environment Variables

None required for production (future configuration planned).

**Client localStorage Keys** (Phase 7):
| Key | Purpose | Format |
|-----|---------|--------|
| `vibetea_token` | WebSocket authentication token | String |

## Phase Changes Summary

### Phase 4 Changes

**Monitor Parser Module** (`monitor/src/parser.rs`):
- Claude Code JSONL parsing with privacy-first approach
- Event mapping for tool invocations and completions
- SessionParser for multi-line file processing
- Comprehensive error handling

**Monitor File Watcher Module** (`monitor/src/watcher.rs`):
- File system watching with position tracking
- Efficient tailing without re-reading
- WatchEvent enum for file system notifications
- Thread-safe Arc<RwLock<>> state management

**New Dependencies**:
- `futures` 0.3 - Async utilities for monitor
- `tempfile` 3.15 - Testing support

**Module Organization**:
- `monitor/src/lib.rs` - Enhanced exports for parser and watcher
- `monitor/src/parser.rs` - JSONL parsing logic
- `monitor/src/watcher.rs` - File system watching logic

**Integration Status**: Parser and watcher modules complete; integration with main event loop in progress.

### Phase 5 Changes

**Monitor Privacy Module** (`monitor/src/privacy.rs` - 1039 lines):
- **PrivacyConfig**: Configuration management for privacy filtering
- **PrivacyPipeline**: Event sanitization processor
- **extract_basename()**: Path-to-basename conversion utility
- **Sensitive tool detection**: Hardcoded list (Bash, Grep, Glob, WebSearch, WebFetch)
- **Extension allowlist**: Optional filtering by file extension
- **Summary stripping**: Session summary neutralization
- **Comprehensive documentation**: Privacy guarantees and usage examples

**Privacy Test Suite** (`monitor/tests/privacy_test.rs` - 951 lines):
- 18+ comprehensive privacy compliance tests
- Validates Constitution I (Privacy by Design)
- Path sanitization, sensitive tool stripping, content filtering
- Extension allowlist verification, pattern detection
- Sensitive data exclusion validation

**Configuration** (`VIBETEA_BASENAME_ALLOWLIST`):
- Comma-separated extensions (e.g., `.rs,.ts,.md`)
- Auto-adds dots: `rs,ts,md` → `.rs,.ts,.md`
- Whitespace-tolerant: ` .rs , .ts ` normalized
- Empty entries filtered: `.rs,,.ts,,,` → `.rs,.ts`
- When not set: All extensions allowed

**Integration Status**: Privacy module complete and ready for main event loop integration.

### Phase 6 Changes

**Monitor Crypto Module** (`monitor/src/crypto.rs` - 438 lines):
- Ed25519 keypair generation and management
- Key persistence with proper file permissions (0600/0644)
- Key loading with validation
- Public key base64 encoding for server registration
- Event signing with base64-encoded signatures
- Comprehensive error handling

**Monitor Sender Module** (`monitor/src/sender.rs` - 544 lines):
- HTTP client with connection pooling and timeouts
- Event buffering with FIFO eviction strategy
- Exponential backoff retry (1s → 60s with ±25% jitter)
- Rate limit handling (429 with Retry-After)
- Batch event sending with signature support
- Graceful shutdown with event flushing

**Monitor CLI Module** (`monitor/src/main.rs` - 301 lines):
- `vibetea-monitor init` - Generate Ed25519 keypair
- `vibetea-monitor run` - Start monitor daemon
- `vibetea-monitor help` - Show documentation
- `vibetea-monitor version` - Show version
- Signal handling (SIGINT/SIGTERM)
- Graceful shutdown with timeout

**Module Exports** (`monitor/src/lib.rs`):
- Public: Crypto, CryptoError, Sender, SenderConfig, SenderError
- Documentation updated

**Integration Status**: Crypto, sender, and CLI modules complete. Main event loop integration next (Phase 7).

### Phase 7 Changes

**Client WebSocket Hook** (`client/src/hooks/useWebSocket.ts` - 321 lines):
- Custom React hook for WebSocket connection management
- Auto-reconnection with exponential backoff (1s → 60s, ±25% jitter)
- Token management from localStorage
- Integration with Zustand event store
- Manual `connect()` and `disconnect()` control
- Message validation and event dispatch
- Proper cleanup and lifecycle management

**Connection Status Component** (`client/src/components/ConnectionStatus.tsx` - 106 lines):
- Visual status indicator (colored dot)
- Optional status label text
- Selective Zustand subscription for performance
- Accessibility support (ARIA roles, labels)
- Dark mode support with Tailwind CSS
- Responsive and composable design

**Token Form Component** (`client/src/components/TokenForm.tsx` - 201 lines):
- Password-protected token input field
- localStorage persistence for token
- Save/Clear button controls
- Status indicator (saved/not-saved)
- Cross-window synchronization via storage events
- Optional `onTokenChange` callback
- Full accessibility support
- Dark mode Tailwind styling

**Integration Points**:
- useWebSocket reads token from localStorage (set by TokenForm)
- ConnectionStatus displays real-time status from useEventStore
- TokenForm provides UI for user authentication
- All components use selective subscriptions for performance
- TypeScript strict mode compliance throughout

**Client Type System**:
- Complete VibeteaEvent type definitions
- Discriminated union types for payload safety
- Type guards for runtime validation
- Session interface for UI state
- EventPayloadMap for type-safe access

**Integration Status**: Phase 7 client-side features complete. Ready for main event loop integration and session visualization UI.
