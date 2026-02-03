# External Integrations

**Status**: Phase 4 Implementation - Agent spawning and token usage tracking
**Last Updated**: 2026-02-03

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
**Agent Tracker**: `monitor/src/trackers/agent_tracker.rs` (Phase 4 - Task tool agent spawn tracking)

**Privacy-First Approach**:
- Only metadata extracted: tool names, timestamps, file basenames, agent types, descriptions
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
| `assistant` with `tool_use` (non-Task) | Tool invocation | ToolStarted | tool name, context |
| `assistant` with `tool_use` (Task tool) | Agent spawn | AgentSpawned | subagent_type, description |
| `progress` with `PostToolUse` | Tool completion | ToolCompleted | tool name, success |
| `user` | User activity | Activity | timestamp only |
| `summary` | Session end marker | Summary | session metadata |
| File creation | Session start | SessionStarted | project from path |

**Task Tool Special Handling** (Phase 4):
- Task tool invocations (Claude Code subagents) trigger dual event emission
- ToolStarted event: Tracks the Task tool itself
- AgentSpawned event: Tracks the spawned subagent with type and description
- Uses `monitor/src/trackers/agent_tracker.rs` for parsing
- Privacy-first: Only extracts subagent_type and description, never the prompt field

**Agent Tracker Module** (`monitor/src/trackers/agent_tracker.rs`):
- **TaskToolInput**: Structure holding parsed Task tool metadata
  - `subagent_type`: Type of agent being spawned (e.g., "devs:rust-dev", "task")
  - `description`: Task description for the spawned agent
- **parse_task_tool_use()**: Extracts Task tool metadata from Claude Code events
  - Ignores `prompt` field for privacy
  - Handles missing fields with sensible defaults
  - Returns Option<TaskToolInput>
- **try_extract_agent_spawn()**: Creates AgentSpawnEvent from Task tool invocation
  - Combines parsing and event construction
  - Maps subagent_type to agent_type in output event

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
| Agent | Pass through unchanged (description is high-level task description) |
| AgentSpawn | Pass through unchanged (subagent_type and description are safe) |
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

Input:  AgentSpawn { description: "Implement error handling", agent_type: "devs:rust-dev", ... }
Output: AgentSpawn { description: "Implement error handling", agent_type: "devs:rust-dev", ... }  # Pass through

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

## Agent Spawn Tracking (Phase 4)

### Agent Tracker Integration

**Module Location**: `monitor/src/trackers/agent_tracker.rs` (716 lines)

**Purpose**: Detects and tracks Claude Code Task tool agent spawns for monitoring subagent execution.

**Core Types**:

1. **TaskToolInput** - Represents parsed Task tool parameters
   ```rust
   pub struct TaskToolInput {
       pub subagent_type: String,    // e.g., "devs:rust-dev", "task"
       pub description: String,       // Task description for the spawned agent
   }
   ```

2. **AgentSpawnEvent** - Emitted when Task tool spawns subagent
   ```rust
   pub struct AgentSpawnEvent {
       pub session_id: String,        // Session where agent was spawned
       pub agent_type: String,        // Type of agent spawned
       pub description: String,       // Description of agent's task
       pub timestamp: DateTime<Utc>,  // When it was spawned
   }
   ```

**Parsing Functions**:

1. **parse_task_tool_use(tool_name, input)** → Option<TaskToolInput>
   - Checks if tool_name equals "Task" (case-sensitive)
   - Deserializes JSON input to TaskToolInput
   - Handles missing fields with defaults: subagent_type defaults to "task", description to ""
   - Never deserializes or stores the `prompt` field
   - Returns None for non-Task tools or parse failures

2. **create_agent_spawn_event(session_id, timestamp, task_input)** → AgentSpawnEvent
   - Constructs full event from parsed input
   - Maps task_input.subagent_type → agent_type
   - Preserves session context and timestamp

3. **try_extract_agent_spawn(tool_name, input, session_id, timestamp)** → Option<AgentSpawnEvent>
   - Convenience function combining parsing and creation
   - Returns complete AgentSpawnEvent or None

**Privacy Guarantees**:
- **Prompt field never extracted**: Privacy-first design ignores sensitive task prompts
- **Metadata only**: Only subagent_type and description captured
- **Type mapping safe**: Agent type is categorical, never contains sensitive info
- **Description safe**: Expected to be high-level task description, not code or details

**Event Emission**:
- Task tool invocations emit TWO events:
  1. ToolStarted - For the Task tool itself (maps tool context to basename)
  2. AgentSpawned - For the spawned subagent (includes type and description)
- Non-Task tools emit only ToolStarted event
- Both pass through PrivacyPipeline before transmission

**Supported Agent Types** (examples):
- `task` - Generic task agent
- `devs:rust-dev` - Rust developer agent
- `devs:python-dev` - Python developer agent
- `background:file-watcher` - Background file watcher agent
- Custom: Any value in subagent_type field

**Test Coverage**: 40+ test cases validating:
- Task tool parsing with all fields
- Subagent type extraction and defaults
- Description extraction and handling
- Non-Task tool rejection
- Missing field handling
- Extra field handling
- Edge cases (empty input, null, malformed JSON)
- Realistic JSONL parsing from Claude Code format

### Stats Tracker Module (Phase 4)

**Module Location**: `monitor/src/trackers/stats_tracker.rs` (265 lines)

**Purpose**: Accumulates token usage and session statistics from events for monitoring system health.

**Core Type**:

1. **StatsTracker** - Maintains aggregated statistics
   - Tracks per-session and per-model token consumption
   - Accumulates activity patterns across 24-hour windows
   - Provides snapshot of global and session-level metrics

**Tracking Capabilities**:
- **Token usage by model**: Input/output tokens, cache read/write tokens
- **Per-session metrics**: Total sessions, messages, tool invocations
- **Activity patterns**: Hourly distribution of events
- **Model distribution**: Token usage breakdown by model with cache metrics
- **Cache metrics**: Hit rates and efficiency by model

**Integration Points**:
- Receives events from main event loop
- Accumulates TokenUsageEvent data
- Maintains in-memory statistics (not persisted in Phase 4)
- Provides metrics for emission to server

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

## Client-Side Integrations (Phase 7-10)

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

### Event Stream Component (Phase 8)

**Module Location**: `client/src/components/EventStream.tsx` (425 lines)

**Features**:

1. **Virtual Scrolling Performance**
   - Uses `@tanstack/react-virtual` for efficient large-list rendering
   - Estimated row height: 64 pixels
   - Overscan: 5 items (renders items beyond viewport)
   - Supports 1000+ events without performance degradation
   - Memory-efficient: Only visible items rendered

2. **Auto-Scroll Behavior**
   - Automatically scrolls to latest event when new events arrive
   - Auto-scroll disabled when user scrolls up 50+ pixels from bottom
   - "Jump to Latest" button appears when auto-scroll is paused
   - Button shows count of new events available
   - Clicking button re-enables auto-scroll and scrolls to bottom

3. **Event Display**
   - **EventRow sub-component**: Renders single event
   - Event type icon (emoji): Unique symbol for each event type
   - Color-coded badge: Type-specific Tailwind CSS colors
   - Description: Concise event summary from payload
   - Source/Session ID: Source and truncated session ID
   - Timestamp: RFC 3339 converted to HH:MM:SS format

4. **Event Type Styling**
   - session: Purple badge with rocket emoji
   - activity: Green badge with comment emoji
   - tool: Blue badge with wrench emoji
   - agent: Amber badge with robot emoji
   - agent_spawn: Indigo badge with sparkle emoji
   - summary: Cyan badge with clipboard emoji
   - error: Red badge with warning emoji

5. **Event Description Extraction**
   - Session: "Session started: project-name" or "Session ended: project-name"
   - Activity: "Activity in project-name" or "Activity heartbeat"
   - Tool: "tool-name status" with optional context
   - Agent: "Agent state: state-name"
   - AgentSpawn: "Agent spawned: agent_type for description"
   - Summary: First 80 chars of summary text + ellipsis
   - Error: "Error: error-category"

6. **Empty State**
   - Friendly message when no events available
   - Icon and descriptive text
   - Guides user to wait for events

7. **Accessibility**
   - `role="log"` for semantic event stream
   - `aria-live="polite"` for live region updates
   - `role="list"` and `role="listitem"` for event items
   - Proper `aria-label` attributes for elements
   - Event count in aria-label
   - Timestamp as `<time>` element with `dateTime` attribute

8. **Integration with Zustand Store**
   - Selective subscription: only re-renders when events change
   - Uses `useEventStore` hook with selector
   - Gets `events` array (newest-first ordering)
   - Reverses array for display (oldest at top, newest at bottom)

### Session Overview Component (Phase 10)

**Module Location**: `client/src/components/SessionOverview.tsx` (484 lines)

**Features**:

1. **Session Cards Display**
   - Real-time activity indicators with pulsing dots
   - Project name as title
   - Source identifier
   - Session duration (formatted)
   - Status badges (Active, Idle, Ended)
   - Event count for active sessions
   - "Last active" timestamp for inactive sessions

2. **Activity Indicators**
   - Pulsing dot showing activity level (variable speed)
   - 1-5 events in 60s: 1Hz pulse (slow)
   - 6-15 events in 60s: 2Hz pulse (medium)
   - 16+ events in 60s: 3Hz pulse (fast)
   - Inactive sessions: Gray dot, no pulse

3. **Status Badges**
   - Active: Green badge with "Active" label
   - Inactive: Yellow badge with "Idle" label
   - Ended: Gray badge with "Ended" label

4. **Session Sorting**
   - Active sessions first
   - Then by last event time descending
   - Maintains consistent ordering across renders

5. **Recent Event Counting**
   - 60-second window for activity calculation
   - Uses most recent event timestamp as reference
   - Pure render behavior with memoization

6. **Click Handlers & Filtering**
   - Optional `onSessionClick` callback
   - Future feature: filter events by session
   - Keyboard support (Enter/Space)

7. **Accessibility**
   - `role="region"` for container
   - `role="list"` and `role="listitem"` for cards
   - `aria-label` describing session info
   - Keyboard focus support
   - Full keyboard navigation

8. **Styling**
   - Dark mode Tailwind CSS
   - Opacity changes for inactive sessions
   - Hover effects for active cards
   - Color-coded status badges

9. **Sub-components**:
   - `ActivityIndicator`: Pulsing dot with animation
   - `StatusBadge`: Color-coded status label
   - `SessionCard`: Individual session display
   - `EmptyState`: Message when no sessions

### Session Timeout Management (Phase 10)

**Module Location**: `client/src/hooks/useSessionTimeouts.ts` (48 lines)

**Hook Features**:

1. **Session State Transitions**
   - Active → Inactive: After 5 minutes without events
   - Inactive/Ended → Removed: After 30 minutes without events
   - Managed by `useEventStore` action `updateSessionStates()`

2. **Periodic Checking**
   - Interval: 30 seconds (SESSION_CHECK_INTERVAL_MS)
   - Called once per interval
   - Non-blocking check with minimal overhead

3. **Integration**
   - Calls `updateSessionStates()` from Zustand store
   - No state management in hook itself
   - Uses only store action

4. **Lifecycle Management**
   - Cleanup on unmount
   - Clears interval when component unmounts
   - Prevents memory leaks
   - No dependencies on props

5. **App-level Usage**
   - Should be called once at root level (App.tsx)
   - Sets up monitoring for all sessions
   - No parameters required

**Store Integration**:
```typescript
const updateSessionStates = useEventStore(
  (state) => state.updateSessionStates
);

useEffect(() => {
  const intervalId = setInterval(() => {
    updateSessionStates();
  }, SESSION_CHECK_INTERVAL_MS);

  return () => {
    clearInterval(intervalId);
  };
}, [updateSessionStates]);
```

### Zustand Store Enhancement (Phase 10)

**Location**: `client/src/hooks/useEventStore.ts`

**Session State Machine**:
- New events → Active (fresh session)
- Active + no events for 5min → Inactive
- Inactive + event → Active
- Any state + summary → Ended
- Ended/Inactive + no events for 30min → Removed

**Session Interface**:
```typescript
interface Session {
  readonly sessionId: string;      // Unique identifier
  readonly source: string;         // Monitor source ID
  readonly project: string;        // Project name
  readonly startedAt: Date;        // Session start
  readonly lastEventAt: Date;      // Last event time
  readonly eventCount: number;     // Total events
  readonly status: SessionStatus;  // 'active' | 'inactive' | 'ended'
}
```

**New Action - updateSessionStates()**:
- Transitions sessions based on time thresholds
- Called every 30 seconds by useSessionTimeouts
- Updates lastEventAt for new events in addEvent()
- Removes sessions after 30 minutes inactivity
- Maintains state machine invariants

**Constants**:
- `INACTIVE_THRESHOLD_MS = 300,000` (5 minutes)
- `REMOVAL_THRESHOLD_MS = 1,800,000` (30 minutes)
- `SESSION_CHECK_INTERVAL_MS = 30,000` (30 seconds)

### Formatting Utilities (Phase 8)

**Module Location**: `client/src/utils/formatting.ts` (331 lines)

**Formatting Functions**:

1. **Timestamp Formatting**
   - `formatTimestamp(timestamp)`: RFC 3339 → "HH:MM:SS"
   - `formatTimestampFull(timestamp)`: RFC 3339 → "YYYY-MM-DD HH:MM:SS"
   - Both use local timezone for display
   - Fallback strings for invalid input

2. **Relative Time Formatting**
   - `formatRelativeTime(timestamp, now?)`: Relative time display
   - Returns: "just now", "5m ago", "2h ago", "yesterday", "3d ago", "2w ago"
   - Optional `now` parameter for testing with fixed reference time
   - Handles future timestamps as "just now"

3. **Duration Formatting**
   - `formatDuration(milliseconds)`: Duration → "1h 30m", "5m 30s", "30s"
   - Shows up to two most significant units
   - Omits seconds when hours present
   - Fallback "0s" for invalid/zero/negative input

4. **Compact Duration Formatting**
   - `formatDurationShort(milliseconds)`: Duration → "1:30:00", "5:30", "0:30"
   - H:MM:SS format for durations >= 1 hour
   - M:SS format for durations < 1 hour
   - Fallback "0:00" for invalid/zero/negative input

5. **Helper Functions**
   - `parseTimestamp()`: Safely parse RFC 3339 to Date
   - `padZero()`: Pad numbers with leading zeros
   - `isSameDay()`: Check if dates are same calendar day
   - `isYesterday()`: Check if date1 is yesterday relative to date2

6. **Error Handling**
   - All functions handle invalid input gracefully
   - Return sensible fallback strings
   - No exceptions thrown
   - No side effects (pure functions)

7. **Usage in Components**
   - SessionOverview uses formatDuration() for session duration
   - SessionOverview uses formatRelativeTime() for last active time
   - EventStream uses formatTimestamp() for event timestamps
   - Heatmap uses formatCellDateTime() for cell labels

## Event Validation & Types

### Shared Event Schema

All components use a unified event schema for message passing:

**Event Structure** (from `server/src/types.rs`):
```
Event {
  id: String,           // evt_<20-char-alphanumeric>
  source: String,       // Source identifier (e.g., hostname)
  timestamp: DateTime,  // RFC 3339 UTC
  type: EventType,      // session, activity, tool, agent, agent_spawn, summary, error
  payload: EventPayload // Type-specific data (EventPayload enum)
}
```

**Supported Event Types** (Phase 4+):
| Type | Payload Fields | Purpose |
|------|----------------|---------|
| `session` | sessionId, action (started/ended), project | Track session lifecycle |
| `activity` | sessionId, project (optional) | Heartbeat events |
| `tool` | sessionId, tool, status (started/completed), context, project | Tool usage tracking |
| `agent` | sessionId, state | Agent state changes |
| `agent_spawn` | sessionId, agent_type, description, timestamp | Task tool agent spawns (Phase 4) |
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
- Task tool invocations emit AgentSpawned events
- Session lifecycle inferred from JSONL file creation/removal and summary markers
- Agent tracking integrated via `monitor/src/trackers/agent_tracker.rs`

**Phase 5 Privacy Integration** (`monitor/src/privacy.rs`):
- ProcessedEvent payloads through PrivacyPipeline before transmission
- Sensitive contexts stripped according to tool type
- Paths reduced to basenames with extension filtering
- Summary text neutralized to privacy-safe message
- AgentSpawn payloads pass through unchanged (descriptions are safe)

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

**Phase 8 Display** (new):
- EventStream renders events with virtual scrolling
- Formatting utilities provide consistent timestamp/duration display
- Color-coded badges and icons for event types
- Event descriptions extracted from payloads

**Phase 10 Session Management** (new):
- Sessions created from first event with sessionId
- Session state transitions based on event timing
- Activity indicators updated from event frequency
- Session timeout management with periodic checking

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
  updateSessionStates: () => void;       // Phase 10 addition
}
```

**Event Processing**:
- FIFO eviction: Keeps last 1000 events, newest first
- Session aggregation: Derives Session objects from events
- Session status transitions: 'active' → 'ended' on summary event
- Event counting: Increments eventCount per session
- Project tracking: Updates project field if present in event payload
- Timeout management: Session state transitions via updateSessionStates()

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

**Client-Side Handling** (Phase 7-10):
- WebSocket proxy configured in `client/vite.config.ts` (target: ws://localhost:8080)
- State management via `useEventStore` hook (Zustand)
- Event type guards for safe type access in `client/src/types/events.ts`
- ConnectionStatus transitions: disconnected → connecting → connected → reconnecting
- Token management via `TokenForm` component
- Connection control via `useWebSocket` hook
- Virtual scrolling display via EventStream component (Phase 8)
- Session management via SessionOverview component (Phase 10)

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
- Agent tracking: Detects Task tool invocations with agent_tracker module
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

**Phase 7-10 Client Features**:
- Token management via TokenForm component
- Connection status visualization via ConnectionStatus component
- WebSocket connection management via useWebSocket hook
- Event display and session tracking via Zustand store
- Virtual scrolling with EventStream component (Phase 8)
- Timestamp and duration formatting with utilities (Phase 8)
- Activity heatmap with Heatmap component (Phase 9)
- Session overview with SessionOverview component (Phase 10)
- Session timeout management via useSessionTimeouts hook (Phase 10)
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
- Phase 4: Agent tracking errors (malformed Task tool input)
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

**Client-Side Error Handling** (Phase 7-10):
- WebSocket connection errors logged to console
- Message parsing failures handled gracefully
- Invalid events silently discarded
- No crashes on connection drops (auto-reconnect)
- Token missing handled with warning log
- Formatting functions handle invalid timestamps/durations with fallback strings
- Session timeout checking handles missing sessions gracefully
- No runtime errors from formatting utility functions

**Resilience**:
- Continues watching even if individual file operations fail
- Retries HTTP requests with exponential backoff (Phase 6)
- Logs errors via `tracing` crate with structured context
- Validates VIBETEA_BUFFER_SIZE as positive integer
- Graceful degradation on malformed JSONL lines
- Privacy processing failures logged without exposing sensitive data
- Sender buffers events if network unavailable, retries with backoff
- Client maintains event store even if connection drops
- Virtual scrolling gracefully handles empty event lists and large datasets
- Session management handles edge cases (removed sessions, missing data)

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
- Phase 4: Agent tracker logs Task tool detections and parsing
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
- Only metadata stored: tool names, timestamps, file basenames, agent types
- File paths used only for project name extraction
- PrivacyPipeline (Phase 5) ensures sensitive data not transmitted:
  - Full paths reduced to basenames
  - Sensitive tool contexts always stripped
  - Extension allowlist filtering applied
  - Summary text neutralized
- Agent tracker (Phase 4) only extracts safe metadata:
  - Subagent type and description
  - Never extracts or transmits prompts
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
- **Performance Monitoring**: Client-side performance metrics (Phase 8+)

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

### Phase 4 Changes (Current)

**Agent Tracker Module** (`monitor/src/trackers/agent_tracker.rs`):
- Detects and parses Task tool agent spawns from Claude Code JSONL
- Extracts subagent_type and description (never the prompt)
- Emits AgentSpawnEvent for monitoring subagent execution
- Provides parse_task_tool_use(), create_agent_spawn_event(), try_extract_agent_spawn()
- 40+ test cases covering parsing, defaults, edge cases, realistic JSONL

**Stats Tracker Module** (`monitor/src/trackers/stats_tracker.rs`):
- Accumulates token usage and session statistics
- Per-model token consumption tracking
- Activity pattern and hourly distribution
- Model distribution and cache metrics

**Enhanced Parser** (`monitor/src/parser.rs`):
- New ParsedEventKind::AgentSpawned variant
- Task tool detection and dual event emission
- ToolStarted (for Task tool itself) + AgentSpawned (for spawned agent)

**Enhanced Event Types** (`monitor/src/types.rs`):
- AgentSpawnEvent with session_id, agent_type, description, timestamp
- EventType::AgentSpawn enum variant
- EventPayload::AgentSpawn payload variant
- Multiple new event types for token tracking and metrics

**Integration Points**:
- Agent tracker called from parser for Task tool invocations
- AgentSpawn events processed through PrivacyPipeline (pass-through)
- Both ToolStarted and AgentSpawned emitted for Task tools
- StatsTracker integrated into main event loop for token accumulation
- Client EventStream displays agent_spawn events with special badge
- Session tracking includes agent spawn count in metrics

### Phase 10 Changes (Previous)

**Client Session Timeout Hook** (`client/src/hooks/useSessionTimeouts.ts` - 48 lines):
- Sets up periodic interval (30 seconds) to check and update session states
- Transitions sessions: Active → Inactive (5min), Inactive/Ended → Removed (30min)
- Integrates with Zustand store's `updateSessionStates()` action
- Proper cleanup on unmount

**Session Overview Component** (`client/src/components/SessionOverview.tsx` - 484 lines):
- Displays session cards with project, source, duration, activity indicator, status badge
- Real-time activity indicators with variable pulse rates (1Hz, 2Hz, 3Hz based on event volume)
- Status badges: Active (green), Idle (yellow), Ended (gray)
- Session sorting: Active first, then by last event time
- Recent event counting over 60-second window
- Click handlers for session filtering (future feature)
- Full accessibility support with ARIA labels and keyboard navigation

**Zustand Store Enhancement** (`client/src/hooks/useEventStore.ts`):
- New constants: INACTIVE_THRESHOLD_MS, REMOVAL_THRESHOLD_MS, SESSION_CHECK_INTERVAL_MS
- Session interface with sessionId, source, project, startedAt, lastEventAt, eventCount, status
- New action: `updateSessionStates()` for periodic state transitions
- Enhanced `addEvent()` to update lastEventAt and handle session creation
- Map-based session storage keyed by sessionId

**CSS Animations** (`client/src/index.css`):
- Pulse animations already defined in Phase 9:
  - pulse-slow: 1Hz (1 second cycle)
  - pulse-medium: 2Hz (0.5 second cycle)
  - pulse-fast: 3Hz (0.33 second cycle)
- Used by ActivityIndicator component for activity level visualization

**Integration Points**:
- SessionOverview subscribes to sessions and events from store
- useSessionTimeouts manages periodic state transitions
- formatDuration() and formatRelativeTime() used for display formatting
- Session status machine: Active → Inactive → Removed, with transitions on summary event
