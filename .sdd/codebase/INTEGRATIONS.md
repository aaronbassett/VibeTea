# External Integrations

**Status**: Phase 8 Implementation Complete - Enhanced token and session metrics tracking
**Generated**: 2026-02-04
**Last Updated**: 2026-02-04

## Summary

VibeTea is a distributed event system with three components:
- **Monitor**: Captures Claude Code session events from local JSONL files and todo files, applies privacy filtering, signs with Ed25519, and transmits to server via HTTP
- **Server**: Receives, validates, verifies signatures, and broadcasts events via WebSocket
- **Client**: Subscribes to server events via WebSocket for real-time visualization with token-based authentication

All integrations use standard protocols (HTTPS, WebSocket) with cryptographic message authentication and privacy-by-design data handling.

## File System Integration

### Claude Code Session Files (JSONL)

**Source**: `~/.claude/projects/**/*.jsonl`
**Format**: JSON Lines (one JSON object per line, append-only)
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Parser Location**: `monitor/src/parser.rs` (SessionParser, ParsedEvent, ParsedEventKind)
**Watcher Location**: `monitor/src/watcher.rs` (FileWatcher, WatchEvent)
**Privacy Pipeline**: `monitor/src/privacy.rs` (PrivacyConfig, PrivacyPipeline)
**Agent Tracker**: `monitor/src/trackers/agent_tracker.rs` (Task tool agent spawn tracking)

**Privacy-First Approach**:
- Only metadata extracted: tool names, timestamps, file basenames, agent types
- Never processes code content, prompts, or responses
- File path parsing for project name extraction
- All event payloads sanitized through PrivacyPipeline

**Session File Structure**:
```
~/.claude/projects/<project-slug>/<session-uuid>.jsonl
```

**Supported Event Types** (from Claude Code JSONL):
| Claude Code Type | Parsed As | VibeTea Event | Fields |
|------------------|-----------|---------------|--------|
| `assistant` with `tool_use` (non-Task) | Tool invocation | ToolStarted | tool name, context |
| `assistant` with `tool_use` (Task tool) | Agent spawn | AgentSpawned | agent_type, description |
| `progress` with `PostToolUse` | Tool completion | ToolCompleted | tool name, success |
| `user` | User activity | Activity | timestamp only |
| `summary` | Session end marker | Summary | session metadata |
| File creation | Session start | SessionStarted | project from path |

**Watcher Behavior**:
- Monitors `~/.claude/projects/` directory recursively
- Detects file creation, modification, deletion events
- Maintains position map for efficient tailing (no re-reading)
- Emits WatchEvent::FileCreated, WatchEvent::LinesAdded, WatchEvent::FileRemoved

**Configuration** (`monitor/src/config.rs`):
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | Claude directory to monitor |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated file extensions to watch |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |

### Claude Code History File (Phase 5)

**Source**: `~/.claude/history.jsonl`
**Format**: JSON Lines (one JSON object per line, append-only)
**Update Mechanism**: File system watcher via `notify` crate

**Skill Tracker Location**: `monitor/src/trackers/skill_tracker.rs` (1837 lines)
**Tokenizer Location**: `monitor/src/utils/tokenize.rs`

**Purpose**: Real-time tracking of user skill/slash command invocations

**History.jsonl Structure**:
```json
{
  "display": "/commit -m \"fix: update docs\"",
  "timestamp": 1738567268363,
  "project": "/home/ubuntu/Projects/VibeTea",
  "sessionId": "6e45a55c-3124-4cc8-ad85-040a5c316009"
}
```

**Fields**:
- `display`: Command string with arguments (e.g., "/commit -m \"message\"")
- `timestamp`: Unix milliseconds
- `project`: Absolute path to project root
- `sessionId`: UUID of Claude Code session

**Privacy-First Approach**:
- Only skill name extracted (e.g., "commit" from "/commit -m \"fix\"")
- Command arguments never transmitted
- Project path included for context (identifies project, not code)

**Skill Tracker Module** (`monitor/src/trackers/skill_tracker.rs`):

1. **Core Types**:
   - `SkillInvocationEvent` - Emitted when user invokes a skill
   - `HistoryEntry` - Parsed entry from history.jsonl
   - `SkillTracker` - File watcher and parser
   - `SkillTrackerConfig` - Startup behavior configuration

2. **Parsing Functions**:
   - `parse_history_entry(line)` - Parses JSON with validation
   - `parse_history_entries(content)` - Parses multiple lines, lenient
   - `create_skill_invocation_event(entry)` - Constructs event

3. **File Watching**:
   - Watches parent directory of history.jsonl
   - Detects file creation, modification
   - Maintains atomic byte offset
   - Handles truncation gracefully
   - Emits SkillInvocationEvent via mpsc channel

4. **Skill Name Extraction** (`monitor/src/utils/tokenize.rs`):
   - `extract_skill_name(display)` - Parses from display string
   - Handles `/commit` → `commit`
   - Handles `/sdd:plan` → `sdd:plan`
   - Handles `/review-pr` → `review-pr`
   - Handles arguments: `/commit -m \"fix\"` → `commit`

**Configuration**:
- No specific environment variables (uses default ~/.claude)
- Optional: Extend to support custom history.jsonl paths

**Test Coverage**: 60+ comprehensive tests covering:
- Parsing (12 tests)
- Multiple entries (6 tests)
- Methods (5 tests)
- Skill extraction (10 tests)
- Event creation (5 tests)
- File operations (12+ async tests)

### Claude Code Todo Files (Phase 6)

**Source**: `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json`
**Format**: JSON Array of todo objects
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Todo Tracker Location**: `monitor/src/trackers/todo_tracker.rs` (2345 lines)
**Utility Location**: `monitor/src/utils/debounce.rs`, `monitor/src/utils/session_filename.rs`

**Purpose**: Track todo list progress and detect abandoned tasks per session

**Todo File Structure**:
```json
[
  {
    "content": "Task description text",
    "status": "completed",
    "activeForm": "Completing task..."
  },
  {
    "content": "Another task",
    "status": "in_progress",
    "activeForm": "Working on task..."
  },
  {
    "content": "Pending task",
    "status": "pending",
    "activeForm": null
  }
]
```

**Fields**:
- `content`: Task description (never transmitted for privacy)
- `status`: One of `completed`, `in_progress`, `pending`
- `activeForm`: Optional active form text shown during task execution

**Privacy-First Approach**:
- Only status counts extracted: completed, in_progress, pending
- Task content (`content` field) never read or transmitted
- Abandonment detection for analysis (did tasks go incomplete?)
- Session context preserved for correlation

**Todo Tracker Module** (`monitor/src/trackers/todo_tracker.rs`):

1. **Core Types**:
   - `TodoProgressEvent` - Emitted when todo list changes
   - `TodoEntry` - Individual todo item
   - `TodoStatus` - Enum: Completed, InProgress, Pending
   - `TodoStatusCounts` - Aggregated counts by status
   - `TodoTracker` - File watcher for todos directory
   - `TodoTrackerConfig` - Configuration (debounce duration)
   - `TodoParseError` / `TodoTrackerError` - Comprehensive error types

2. **Parsing Functions**:
   - `parse_todo_file(content)` - Strict JSON array parsing
   - `parse_todo_file_lenient(content)` - Lenient parsing, skips invalid entries
   - `parse_todo_entry(value)` - Single entry validation
   - `count_todo_statuses(entries)` - Aggregate counts
   - `extract_session_id_from_filename(path)` - UUID extraction

3. **Abandonment Detection**:
   - `is_abandoned(counts, session_ended)` - True if session ended with incomplete tasks
   - `create_todo_progress_event(session_id, counts, abandoned)` - Event construction
   - Requires explicit session ended tracking via `mark_session_ended()`

4. **File Watching**:
   - Monitors `~/.claude/todos/` directory (non-recursive)
   - Detects .json file creation and modification
   - Validates filename format: `<uuid>-agent-<uuid>.json`
   - Debounces rapid changes (100ms default)
   - Uses notify crate for cross-platform compatibility
   - Maintains RwLock<HashSet> of ended sessions
   - Lenient parsing handles partially-written files

5. **Session Lifecycle Integration**:
   - `mark_session_ended(session_id)` - Call when summary event received
   - `is_session_ended(session_id)` - Query ended status
   - `clear_session_ended(session_id)` - Reset ended status
   - Abandonment flag set only if session ended AND incomplete tasks exist

**Configuration**:
- Default location: `~/.claude/todos/`
- Debounce interval: 100ms (coalesce rapid writes)
- No environment variables required (uses `directories` crate)

**Test Coverage**: 100+ comprehensive tests:
- Filename parsing (8 tests)
- Status counting (6 tests)
- Abandonment detection (6 tests)
- Entry parsing (8 tests)
- File parsing (8 tests)
- Lenient parsing (4 tests)
- Trait implementations (3 tests)
- Error messages (2 tests)
- Configuration (2 tests)
- File operations and async (12+ tests)

**Debouncing Implementation** (`monitor/src/utils/debounce.rs`):
- Generic `Debouncer<K, V>` for generic key-value coalescing
- Configurable duration (100ms for todos)
- mpsc channel based event emission
- Prevents duplicate processing of rapid file changes

**Filename Parsing** (`monitor/src/utils/session_filename.rs`):
- `parse_todo_filename(path)` - Extracts session UUID from filename
- Pattern: `<session-uuid>-agent-<session-uuid>.json`
- Returns Option<String> with first UUID

### Claude Code Stats Cache (Phase 8)

**Source**: `~/.claude/stats-cache.json`
**Format**: JSON object with model usage and session metrics
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Stats Tracker Location**: `monitor/src/trackers/stats_tracker.rs` (1279 lines)

**Purpose**: Track token usage per model and global session statistics

**Stats Cache File Structure**:
```json
{
  "totalSessions": 150,
  "totalMessages": 2500,
  "totalToolUsage": 8000,
  "longestSession": "00:45:30",
  "hourCounts": { "0": 10, "1": 5, ..., "23": 50 },
  "modelUsage": {
    "claude-sonnet-4-20250514": {
      "inputTokens": 1500000,
      "outputTokens": 300000,
      "cacheReadInputTokens": 800000,
      "cacheCreationInputTokens": 100000
    },
    "claude-opus-4-20250514": {
      "inputTokens": 500000,
      "outputTokens": 150000,
      "cacheReadInputTokens": 200000,
      "cacheCreationInputTokens": 50000
    }
  }
}
```

**Fields**:
- `totalSessions`: Total number of Claude Code sessions
- `totalMessages`: Total messages across all sessions
- `totalToolUsage`: Total tool invocations
- `longestSession`: Duration string (HH:MM:SS format)
- `hourCounts`: Activity distribution by hour of day (0-23)
- `modelUsage`: Per-model token consumption with cache metrics

**Privacy-First Approach**:
- Only aggregate statistics extracted (never session/prompt content)
- No model-specific information beyond model name
- Cache metrics only (no raw data)
- Session context derived from file name parsing only

**Stats Tracker Module** (`monitor/src/trackers/stats_tracker.rs`):

1. **Core Types**:
   - `StatsEvent` - Enum with variants: `SessionMetrics`, `TokenUsage`
   - `SessionMetricsEvent` - Global session statistics
   - `TokenUsageEvent` - Per-model token consumption
   - `StatsCache` - Deserialized stats-cache.json
   - `ModelTokens` - Per-model token counts
   - `StatsTracker` - File watcher for stats-cache.json
   - `StatsTrackerError` - Comprehensive error types

2. **Parsing Functions**:
   - `read_stats_with_retry()` - Reads with retry logic (up to 3 attempts)
   - `read_stats()` - Synchronous file read and parse
   - `parse_stats_cache()` - Public helper for testing
   - `emit_stats_events()` - Creates SessionMetricsEvent + TokenUsageEvents

3. **Event Emission**:
   - Emits `SessionMetricsEvent` once per stats-cache.json read
   - Emits `TokenUsageEvent` for each model in modelUsage
   - Per-model events include: input tokens, output tokens, cache read tokens, cache creation tokens

4. **File Watching**:
   - Monitors `~/.claude/stats-cache.json` for changes
   - 200ms debounce interval to coalesce rapid writes
   - Handles initial read if file exists on startup
   - Retries JSON parse with 100ms delays (up to 3 attempts)
   - Uses notify crate for cross-platform FSEvents/inotify
   - Graceful degradation if file unavailable

5. **Main Event Loop Integration**:
   - StatsTracker initialization during startup (optional, warns on failure)
   - Dedicated channel: `mpsc::channel::<StatsEvent>`
   - Stats event processing in main select! loop
   - `process_stats_event()` handler in main.rs converts to Event

**Configuration**:
- Default location: `~/.claude/stats-cache.json`
- Debounce interval: 200ms
- Parse retry delay: 100ms
- Max retries: 3 attempts
- No environment variables required (uses `directories` crate)

**Test Coverage**: 60+ comprehensive tests covering:
- JSON parsing (7 tests)
- Model token parsing (6 tests)
- Empty/partial stats (3 tests)
- Malformed JSON handling (3 tests)
- Stats event emission (3 tests)
- Debounce timing (2 tests)
- Parse retry logic (2 tests)
- Missing/malformed files (3 tests)
- Tracker creation (2 tests)
- Initial read behavior (1 test)
- Refresh method (1 test)
- Enum/equality tests (10 tests)

## Privacy & Data Sanitization

### Privacy Pipeline Architecture

**Location**: `monitor/src/privacy.rs` (1039 lines)

**Core Components**:

1. **PrivacyConfig** - Configuration management
   - Optional extension allowlist (e.g., `.rs`, `.ts`)
   - Loaded from `VIBETEA_BASENAME_ALLOWLIST`
   - Supports comma-separated format

2. **PrivacyPipeline** - Event sanitization processor
   - Processes EventPayload before transmission
   - Strips sensitive contexts
   - Extracts basenames from paths
   - Applies extension filtering
   - Neutralizes summary text

3. **extract_basename()** - Path safety function
   - `/home/user/src/auth.ts` → `auth.ts`
   - Handles Unix, Windows, relative paths
   - Returns `None` for invalid paths

**Sensitive Tools** (context always stripped):
- `Bash` - Commands may contain secrets
- `Grep` - Patterns reveal search intent
- `Glob` - Patterns reveal project structure
- `WebSearch` - Queries reveal intent
- `WebFetch` - URLs may contain secrets

**Privacy Processing Rules**:
| Payload Type | Processing |
|--------------|-----------|
| Session | Pass through |
| Activity | Pass through |
| Tool (sensitive) | Context set to None |
| Tool (other) | Basename + allowlist filtering |
| Agent | Pass through unchanged |
| AgentSpawn | Pass through unchanged |
| SkillInvocation | Pass through unchanged |
| TodoProgress | Pass through unchanged (only counts) |
| SessionMetrics | Pass through unchanged (aggregate data) |
| TokenUsage | Pass through unchanged (aggregate data) |
| Summary | Text replaced with "Session ended" |
| Error | Pass through unchanged |

**Extension Allowlist Filtering**:
- Not set: All extensions allowed
- Set to `.rs,.ts`: Only those extensions transmitted
- Mismatch: Context filtered to `None`

**Todo Privacy**:
- TodoProgressEvent contains only counts and abandonment flag
- No task content or descriptions transmitted
- Counts are aggregate, non-sensitive metadata

**Stats Privacy**:
- SessionMetricsEvent contains only aggregate counts
- TokenUsageEvent contains per-model consumption metrics
- No per-session data or user information
- Cache metrics are transparent usage data

### Privacy Test Suite

**Location**: `monitor/tests/privacy_test.rs` (951 lines)

**Coverage**: 18+ comprehensive privacy compliance tests
**Validates**: Constitution I (Privacy by Design)

**Test Categories**:
- Path sanitization (no full paths)
- Sensitive tool stripping
- Content stripping
- Prompt/response stripping
- Command argument removal
- Summary neutralization
- Extension filtering
- Sensitive pattern detection

## Authentication & Authorization

### Monitor → Server (Event Publishing)

**Method**: Ed25519 digital signatures
**Protocol**: HTTPS POST with signed payload
**Key Management**: Source-specific public key registration
**Verification**: Constant-time comparison using `subtle` crate

**Configuration Location**: `server/src/config.rs`
- `VIBETEA_PUBLIC_KEYS` - Format: `source1:pubkey1,source2:pubkey2`
- `VIBETEA_UNSAFE_NO_AUTH` - Dev-only authentication bypass

**Signature Verification Process** (`server/src/auth.rs`):
1. Decode base64 signature from X-Signature header
2. Decode base64 public key from configuration
3. Extract Ed25519 VerifyingKey from public key bytes
4. Verify signature with RFC 8032 compliance
5. Apply constant-time comparison to prevent timing attacks

**Cryptographic Details**:
- Algorithm: Ed25519 (ECDSA variant)
- Library: `ed25519-dalek` crate (version 2.1)
- Key generation: 32-byte seed via OS RNG
- File permissions: 0600 (private key only)
- Public key format: Base64-encoded

### Server → Client (Event Streaming)

**Method**: Bearer token in WebSocket headers
**Protocol**: WebSocket upgrade with `Authorization: Bearer <token>`
**Token Type**: Opaque string (no expiration in Phase 5)
**Scope**: All clients use the same token (global scope)

**Configuration Location**: `server/src/config.rs`
- `VIBETEA_SUBSCRIBER_TOKEN` - Required unless unsafe mode
- Validated on WebSocket upgrade
- No token refresh mechanism in Phase 5

**Validation**: Server-side validation only (in-memory)

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

## External APIs

There are no external third-party API integrations in Phase 8. The system is self-contained:
- All data sources are local files
- All services are internal (Monitor, Server, Client)
- No SaaS dependencies or external service calls

**Future Integration Points** (Not Yet Implemented):
- Cloud storage (S3, GCS) for event archive
- Monitoring services (Datadog, New Relic)
- Message queues (Redis, RabbitMQ)
- Webhooks for external notifications
- Database persistence (PostgreSQL, etc.)

## HTTP API Endpoints

### POST /events

**Purpose**: Ingest events from monitors

**Request Headers**:
| Header | Required | Value |
|--------|----------|-------|
| X-Source-ID | Yes | Monitor identifier (non-empty) |
| X-Signature | No* | Base64-encoded Ed25519 signature |
| Content-Type | Yes | application/json |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**Request Body**: Single Event or array of Events (JSON)

**Response Codes**:
- 202 Accepted - Events accepted and broadcasted
- 400 Bad Request - Invalid JSON or malformed events
- 401 Unauthorized - Invalid X-Source-ID or signature mismatch
- 429 Too Many Requests - Rate limit exceeded (includes Retry-After header)

**Rate Limiting** (`server/src/rate_limit.rs`):
- Token bucket algorithm per source
- Default: 100 events/second per source
- Capacity: 100 tokens
- Exceeds limit: Returns 429 with Retry-After header
- Cleanup: Stale sources removed after 60 seconds idle

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
- 101 Switching Protocols - Upgrade successful
- 401 Unauthorized - Token validation failed

**Filtering** (`server/src/broadcast.rs`):
- Optional SubscriberFilter based on query parameters
- Matches event type, source, project
- Enables selective delivery

### GET /health

**Purpose**: Health check and uptime reporting

**Response**:
```json
{
  "status": "ok",
  "uptime_secs": 3600
}
```

**Response Code**: 200 OK (always succeeds, no auth)

## Network Communication

### Monitor → Server (Event Publishing)

**Endpoint**: `https://<server-url>/events`
**Method**: POST
**Content-Type**: application/json

**Flow**:
1. Monitor watches local JSONL files via file watcher
2. Parser extracts metadata from new/modified lines
3. Events processed through PrivacyPipeline
4. Monitor signs event payload with Ed25519 private key
5. Monitor POSTs signed event with X-Source-ID and X-Signature headers
6. Server validates signature against registered public key
7. Server rate limits based on source ID
8. Server broadcasts to all connected clients via WebSocket

**Client Library**: `reqwest` (HTTP client)

**Configuration** (`monitor/src/config.rs`):
- `VIBETEA_SERVER_URL` - Server endpoint (required)
- `VIBETEA_SOURCE_ID` - Monitor identifier (default: hostname)

**Sender Module** (`monitor/src/sender.rs`):
- HTTP client with connection pooling (10 max idle)
- Event buffering with FIFO eviction (1000 events default)
- Exponential backoff: 1s → 60s with ±25% jitter
- Rate limit handling: Respects 429 with Retry-After
- Timeout: 30 seconds per request

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket with Bearer token
2. Server validates token and establishes connection
3. Server broadcasts events as they arrive
4. Optional filtering based on query parameters
5. Client processes events via Zustand store
6. Client UI renders session information

**Broadcasting** (`server/src/broadcast.rs`):
- EventBroadcaster wraps tokio broadcast channel
- 1000-event capacity for burst handling
- Thread-safe, cloneable across handlers
- SubscriberFilter enables selective delivery

**Client-Side Handling**:
- WebSocket proxy configured in `client/vite.config.ts`
- State management via `useEventStore` hook (Zustand)
- Event type guards in `client/src/types/events.ts`
- ConnectionStatus component for visual feedback
- useWebSocket hook with auto-reconnect

### Monitor → File System (JSONL, Todo, & Stats Watching)

**Targets**:
- `~/.claude/projects/**/*.jsonl` - Session events
- `~/.claude/history.jsonl` - Skill invocations
- `~/.claude/todos/*.json` - Todo lists
- `~/.claude/stats-cache.json` - Token/session statistics

**Mechanism**: `notify` crate file system events
**Update Strategy**: Incremental line reading with position tracking (sessions/history), file debouncing (todos/stats)

**Session File Flow**:
1. FileWatcher initialized with watch directory
2. Recursive file system monitoring begins
3. File creation detected → WatchEvent::FileCreated
4. File modification detected → Read new lines from position
5. Lines accumulated → WatchEvent::LinesAdded
6. Position marker updated
7. File deletion detected → WatchEvent::FileRemoved

**Skill File Monitoring** (Phase 5):
1. SkillTracker initialized with history.jsonl path
2. Watcher monitors parent directory
3. Modification detected (data changes only)
4. New entries read from byte offset
5. Entries parsed → SkillInvocationEvent created
6. Event emitted via mpsc channel
7. Byte offset updated

**Todo File Monitoring** (Phase 6):
1. TodoTracker initialized with todos directory
2. Watcher monitors `~/.claude/todos/` non-recursively
3. File creation/modification detected
4. Filename validated: `<uuid>-agent-<uuid>.json`
5. File content read as JSON array
6. Entries parsed and counted (lenient)
7. Abandonment flag set based on session ended status
8. TodoProgressEvent emitted via mpsc channel
9. Changes debounced at 100ms to coalesce rapid writes

**Stats File Monitoring** (Phase 8):
1. StatsTracker initialized with stats-cache.json path
2. Watcher monitors `~/.claude/` directory
3. File creation/modification detected
4. File content read as JSON object
5. SessionMetricsEvent created and emitted
6. TokenUsageEvent created for each model
7. Events emitted via mpsc channel
8. Changes debounced at 200ms to coalesce rapid writes

**Efficiency Features**:
- Position tracking prevents re-reading (sessions/history)
- Only new lines since last position extracted
- BufReader with Seek for efficient iteration
- Arc<RwLock<>> for thread-safe concurrent access
- Atomic offset for lock-free reads in skill tracker
- Debouncing prevents duplicate processing (todos/stats)

## Development & Local Configuration

### Local Server Setup

**Environment Variables**:
```bash
PORT=8080                                       # Server port
VIBETEA_PUBLIC_KEYS=localhost:cHVia2V5MQ==     # Monitor public key
VIBETEA_SUBSCRIBER_TOKEN=dev-token-secret      # Client token
VIBETEA_UNSAFE_NO_AUTH=false                   # Auth mode
RUST_LOG=debug                                 # Logging level
```

**Unsafe Development Mode**:
When `VIBETEA_UNSAFE_NO_AUTH=true`:
- All monitor authentication bypassed
- All client authentication bypassed
- Suitable for local development only
- Never use in production
- Warning logged on startup

### Local Monitor Setup

**Environment Variables**:
```bash
VIBETEA_SERVER_URL=http://localhost:8080       # Server endpoint
VIBETEA_SOURCE_ID=my-monitor                   # Custom source ID
VIBETEA_KEY_PATH=~/.vibetea                    # Key directory
VIBETEA_CLAUDE_DIR=~/.claude                   # Claude directory
VIBETEA_BUFFER_SIZE=1000                       # Event buffer capacity
VIBETEA_BASENAME_ALLOWLIST=.ts,.tsx,.rs        # File extension filter
RUST_LOG=debug                                 # Logging level
```

**Key Management** (Phase 6):
- `vibetea-monitor init` generates Ed25519 keypair
- Keys stored in ~/.vibetea/ or VIBETEA_KEY_PATH
- Private key: key.priv (0600 permissions)
- Public key: key.pub (0644 permissions)

### Local Client Setup

**Development Server**:
- Runs on port 5173 (Vite default)
- WebSocket proxy to localhost:8080

**Build Configuration** (`client/vite.config.ts`):
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
- Brotli compression for production
- Code splitting: react-vendor, state, virtual chunks

## Event Validation & Schema

### Shared Event Schema

All components use unified event schema for message passing:

**Event Structure** (from `server/src/types.rs`):
```
Event {
  id: String,           // evt_<20-char-alphanumeric>
  source: String,       // Source identifier
  timestamp: DateTime,  // RFC 3339 UTC
  type: EventType,      // Event classification
  payload: EventPayload // Type-specific data
}
```

**Supported Event Types** (Phase 8):
| Type | Payload | Purpose |
|------|---------|---------|
| `session` | sessionId, action, project | Session lifecycle |
| `activity` | sessionId, project | Heartbeat events |
| `tool` | sessionId, tool, status, context | Tool usage |
| `agent` | sessionId, state | Agent state changes |
| `agent_spawn` | sessionId, agent_type, description | Task tool agents |
| `skill_invocation` | sessionId, skill_name, project | Slash commands |
| `todo_progress` | sessionId, completed, in_progress, pending, abandoned | Todo tracking |
| `session_metrics` | total_sessions, total_messages, total_tool_usage, longest_session | Global stats |
| `token_usage` | model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens | Per-model consumption |
| `summary` | sessionId, summary | Session end |
| `error` | sessionId, category | Error reporting |

**Schema Locations**:
- Rust types: `server/src/types.rs`, `monitor/src/types.rs`
- TypeScript types: `client/src/types/events.ts`
- Validation: Serde deserialization with untagged union

## Logging & Observability

### Structured Logging

**Framework**: `tracing` + `tracing-subscriber`
**Configuration**: Environment variable `RUST_LOG`

**Features**:
- JSON output support
- Environment-based filtering
- Structured context in logs

**No External Service Integration** (Phase 8):
- Logs to stdout/stderr only
- Future: Integration with logging services (Datadog, ELK)

## Security Considerations

### Cryptographic Authentication

**Ed25519 Signatures** (Phase 8):
- Library: `ed25519-dalek` crate (version 2.1)
- Key generation: 32-byte seed via OS RNG
- Signature verification: Base64-encoded public keys per source
- Timing attack prevention: `subtle::ConstantTimeEq`

### Privacy

**Claude Code JSONL** (Phase 4-5):
- Parser never extracts code, prompts, responses
- Only metadata: tool names, timestamps, basenames, agent types
- File paths used only for project name extraction
- PrivacyPipeline (Phase 5) sanitizes all transmissions:
  - Full paths reduced to basenames
  - Sensitive tool contexts always stripped
  - Extension filtering applied
  - Summary text neutralized

**History.jsonl** (Phase 5):
- Only skill names extracted
- Command arguments intentionally omitted
- Project paths for context only
- Privacy-first design throughout

**Todo Files** (Phase 6):
- Only status counts transmitted (completed, in_progress, pending)
- Task content never read or transmitted
- Abandonment flag is only metadata
- Session correlation via session_id only

**Stats Cache** (Phase 8):
- Only aggregate statistics transmitted
- No per-session or per-user data
- Model names and consumption metrics only
- Cache metrics for transparency

### Data in Transit

**TLS Encryption**:
- Production: HTTPS (Monitor → Server)
- Production: WSS (Server ↔ Client)
- Local development: HTTP/WS allowed

### Client-Side Security

**Token Storage**:
- localStorage key: `vibetea_token`
- Accessible to scripts in same origin
- XSS vulnerability exposure possible
- WSS recommended for production

---

*This document maps external service dependencies and file system integrations. Update when adding new integrations.*
