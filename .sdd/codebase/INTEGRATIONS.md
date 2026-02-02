# External Integrations

**Status**: Phase 2 - Foundational architecture with cryptographic authentication
**Last Updated**: 2026-02-02

## Summary

VibeTea is designed as a distributed event system with three components:
- **Monitor**: Captures Claude Code session events from local file system
- **Server**: Receives, validates, and broadcasts events via WebSocket
- **Client**: Subscribes to server events via WebSocket for visualization

All integrations use standard protocols (HTTPS, WebSocket) with cryptographic message authentication.

## Authentication & Authorization

### Monitor Authentication (Server → Monitor)

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Ed25519 digital signatures | Rust `ed25519-dalek` crate |
| **Protocol** | HTTPS POST with signed payload | Event signatures in request |
| **Key Management** | Source-specific public key registration | `VIBETEA_PUBLIC_KEYS` env var |
| **Key Format** | Base64-encoded Ed25519 public keys | `source1:pubkey1,source2:pubkey2` |
| **Flow** | Monitor signs event → Server validates signature | `server/src/config.rs`, `server/src/types.rs` |
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

### Client Authentication (Server → Client)

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Bearer token in WebSocket headers | Static token per deployment |
| **Protocol** | WebSocket upgrade with `Authorization: Bearer <token>` | Client sends on connect |
| **Token Type** | Opaque string (no expiration in Phase 2) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| **Scope** | All clients use the same token | No per-user differentiation |
| **Storage** | Server-side validation only | In-memory, no persistence |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_SUBSCRIBER_TOKEN` (required unless unsafe mode enabled)
- Token required for all WebSocket connections
- No token refresh mechanism in Phase 2
- Stored as `Option<String>` in Config struct

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

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

**Phase 2 TypeScript Enhancements** (`client/src/types/events.ts`):
- Discriminated union type for type-safe payload access
- `EventPayload` union with type discriminators
- `EventPayloadMap` interface mapping event types to payloads
- Generic `VibeteaEvent<T>` with type parameter for type narrowing
- Type guards: `isSessionEvent()`, `isActivityEvent()`, `isToolEvent()`, `isAgentEvent()`, `isSummaryEvent()`, `isErrorEvent()`
- Runtime validation: `isValidEventType()` checks against VALID_EVENT_TYPES array
- All payload fields marked as readonly for immutability

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

## Network Communication

### Monitor → Server (Event Publishing)

**Endpoint**: `https://<server-url>/events`
**Method**: POST
**Authentication**: Ed25519 signature in request header or body
**Content-Type**: application/json

**Flow**:
1. Monitor captures event from file system
2. Monitor signs event payload with Ed25519 private key
3. Monitor POSTs signed event to server
4. Server validates signature against registered public key
5. Server broadcasts to all connected clients via WebSocket

**Client Library**: `reqwest` crate (HTTP client)
**Configuration**: `monitor/src/config.rs`
- `VIBETEA_SERVER_URL` - Server endpoint (required)
- `VIBETEA_SOURCE_ID` - Source identifier for event attribution (default: hostname)
- Uses gethostname crate to get system hostname if not provided

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request headers
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket connection with Bearer token
2. Server validates token and establishes connection
3. Server sends events as they arrive from monitors
4. Client processes and stores events in Zustand state via `addEvent()`
5. Client UI renders session information from state

**Client-Side Handling**:
- WebSocket proxy configured in `client/vite.config.ts` (target: ws://localhost:8080)
- State management via `useEventStore` hook (Zustand)
- Event type guards for safe type access in `client/src/types/events.ts`
- ConnectionStatus transitions: disconnected → connecting → connected → reconnecting

**Connection Details**:
- Address/port: Configured via `PORT` environment variable (default: 8080)
- Persistent connection model
- No automatic reconnection (Phase 2)
- No message queuing (direct streaming)
- Events processed with selective subscriptions to prevent unnecessary re-renders

## Development & Local Configuration

### Local Server Setup

**Environment Variables**:
```bash
PORT=8080                                        # Server port
VIBETEA_PUBLIC_KEYS=localhost:cHVia2V5MQ==      # Monitor public key (base64)
VIBETEA_SUBSCRIBER_TOKEN=dev-token-secret        # Client WebSocket token
VIBETEA_UNSAFE_NO_AUTH=false                     # Set true to disable all auth
```

**Unsafe Development Mode**:
When `VIBETEA_UNSAFE_NO_AUTH=true`:
- All monitor authentication is bypassed
- All client authentication is bypassed
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
VIBETEA_BASENAME_ALLOWLIST=ts,tsx,rs             # Optional file extension filter
```

**Configuration Loading**: `monitor/src/config.rs`
- Required: VIBETEA_SERVER_URL (no default)
- Optional defaults use directories crate for platform-specific paths
- Home directory determined via BaseDirs::new()
- Hostname fallback when VIBETEA_SOURCE_ID not set
- Buffer size parsed as usize, validated for positive integers
- Allowlist split by comma, whitespace trimmed, empty entries filtered

**File System Monitoring**:
- Watches directory: VIBETEA_CLAUDE_DIR
- Captures file creation, modification, deletion, and directory changes
- Uses `notify` crate (version 8.0) for cross-platform inotify/FSEvents
- Optional extension filtering via VIBETEA_BASENAME_ALLOWLIST

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
2. Event signature validation on POST
3. Event schema validation (serde untagged enum)
4. Bearer token validation on WebSocket connect

**Config Error Types** (comprehensive):
- MissingEnvVar(String) - Required variable not found
- InvalidFormat { var: String, message: String } - Format/parsing error
- InvalidPort(ParseIntError) - Port not valid u16
- ValidationError(String) - Config validation failed

### Monitor-Side Error Handling

**Error Types** (from `monitor/src/error.rs`):
- Configuration errors (missing env vars, invalid paths)
- File watching errors (permission denied, path not found)
- HTTP request errors (connection refused, timeout)
- Cryptographic errors (invalid private key)

**Config Error Types**:
- MissingEnvVar(String) - VIBETEA_SERVER_URL required
- InvalidValue { key: String, message: String } - Invalid parsed value
- NoHomeDirectory - Cannot determine home directory

**Resilience**:
- Continues watching even if individual file operations fail
- Retries HTTP requests with exponential backoff (future enhancement)
- Logs errors via `tracing` crate with structured context
- Validates VIBETEA_BUFFER_SIZE as positive integer

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

## Logging & Observability

### Structured Logging

**Framework**: `tracing` + `tracing-subscriber`
**Configuration**: Environment variable `RUST_LOG`

**Features**:
- JSON output support (via `tracing-subscriber` with json feature)
- Environment-based filtering
- Structured context in logs

**Components**:
- Server: Logs configuration, connection events, errors
- Monitor: Logs file system events, HTTP requests, signing operations
- Warning logged when VIBETEA_UNSAFE_NO_AUTH is enabled

**No External Service Integration** (Phase 2):
- Logs to stdout/stderr only
- Future: Integration with logging services (e.g., ELK, Datadog)

## Security Considerations

### Cryptographic Authentication

**Ed25519 Signatures**:
- Library: `ed25519-dalek` crate (version 2.1)
- Key generation: 32-byte seed
- Signature verification: Base64-encoded public keys per source
- Private key storage: User's filesystem (unencrypted)

**Security Implications**:
- Private keys must be protected with file permissions
- Public keys registered on server must match monitor's keys
- Signature validation prevents spoofed events

### Token-Based Client Authentication

**Bearer Token**:
- Currently a static string per deployment
- No encryption in transit (relies on TLS via HTTPS)
- No expiration or refresh (Phase 2 limitation)

**Security Implications**:
- Token should be treated like a password
- Compromise affects all connected clients
- Future: Implement token rotation, per-user tokens

### Data in Transit

**TLS Encryption**:
- Production deployments use HTTPS (Monitor → Server)
- Production deployments use WSS (Server ↔ Client)
- Local development may use unencrypted HTTP/WS

## Future Integration Points

### Planned (Not Yet Integrated)

- **Database/Persistence**: Store events beyond memory (Phase 3+)
- **Authentication Providers**: OAuth2, API key rotation (Phase 3+)
- **Monitoring Services**: Datadog, New Relic, CloudWatch (Phase 3+)
- **Message Queues**: Redis, RabbitMQ for event buffering (Phase 4+)
- **Webhooks**: External service notifications (Phase 4+)

## Configuration Quick Reference

### Server Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `PORT` | number | 8080 | No | HTTP server listening port |
| `VIBETEA_PUBLIC_KEYS` | string | - | Yes* | Source public keys (source:key,source:key) |
| `VIBETEA_SUBSCRIBER_TOKEN` | string | - | Yes* | Bearer token for clients |
| `VIBETEA_UNSAFE_NO_AUTH` | boolean | false | No | Disable all authentication (dev only) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | string | - | Yes | Server endpoint (e.g., https://vibetea.fly.dev) |
| `VIBETEA_SOURCE_ID` | string | hostname | No | Monitor identifier |
| `VIBETEA_KEY_PATH` | string | ~/.vibetea | No | Directory with key.priv/key.pub |
| `VIBETEA_CLAUDE_DIR` | string | ~/.claude | No | Claude Code directory to watch |
| `VIBETEA_BUFFER_SIZE` | number | 1000 | No | Event buffer capacity |
| `VIBETEA_BASENAME_ALLOWLIST` | string | - | No | Comma-separated file extensions to watch |

### Client Environment Variables

None required for production (future configuration planned).

## Phase 2 Changes

**Server Config Enhancement**:
- Added comprehensive validation with detailed error messages
- Support for unsafe_no_auth mode for development
- Proper HashMap-based key storage for multiple sources

**Monitor Config Enhancement**:
- Home directory detection via directories crate
- Hostname fallback using gethostname crate
- Path expansion support (~/.vibetea, ~/.claude)
- Buffer size validation
- Allowlist parsing with whitespace handling

**Client Event Integration**:
- Zustand store with selective subscriptions
- Discriminated union types for events
- Type-safe event payload access
- Session aggregation from events
- ConnectionStatus tracking
- Event filtering utilities
