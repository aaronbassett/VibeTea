# External Integrations

**Status**: Phase 5 Implementation Complete - Skill invocation tracking from history.jsonl
**Generated**: 2026-02-04
**Last Updated**: 2026-02-04

## Summary

VibeTea is a distributed event system with three components:
- **Monitor**: Captures Claude Code session events from local JSONL files, applies privacy filtering, signs with Ed25519, and transmits to server via HTTP
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
| Summary | Text replaced with "Session ended" |
| Error | Pass through unchanged |

**Extension Allowlist Filtering**:
- Not set: All extensions allowed
- Set to `.rs,.ts`: Only those extensions transmitted
- Mismatch: Context filtered to `None`

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

There are no external third-party API integrations in Phase 5. The system is self-contained:
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

### Monitor → File System (JSONL Watching)

**Target**: `~/.claude/projects/**/*.jsonl` and `~/.claude/history.jsonl`
**Mechanism**: `notify` crate file system events
**Update Strategy**: Incremental line reading with position tracking

**Flow**:
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

**Efficiency Features**:
- Position tracking prevents re-reading
- Only new lines since last position extracted
- BufReader with Seek for efficient iteration
- Arc<RwLock<>> for thread-safe concurrent access
- Atomic offset for lock-free reads in skill tracker

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

**Supported Event Types** (Phase 5):
| Type | Payload | Purpose |
|------|---------|---------|
| `session` | sessionId, action, project | Session lifecycle |
| `activity` | sessionId, project | Heartbeat events |
| `tool` | sessionId, tool, status, context | Tool usage |
| `agent` | sessionId, state | Agent state changes |
| `agent_spawn` | sessionId, agent_type, description | Task tool agents |
| `skill_invocation` | sessionId, skill_name, project | Slash commands |
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

**No External Service Integration** (Phase 5):
- Logs to stdout/stderr only
- Future: Integration with logging services (Datadog, ELK)

## Security Considerations

### Cryptographic Authentication

**Ed25519 Signatures** (Phase 6):
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
