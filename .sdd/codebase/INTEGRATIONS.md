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
| **Flow** | Monitor signs event → Server validates signature | `src/config.rs`, `src/types.rs` |
| **Fallback** | Unsafe no-auth mode (dev only) | `VIBETEA_UNSAFE_NO_AUTH=true` |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_PUBLIC_KEYS` (required unless unsafe mode enabled)
- Parses `VIBETEA_UNSAFE_NO_AUTH` (dev-only authentication bypass)
- Validates on every server startup

**Example Key Format**:
```
VIBETEA_PUBLIC_KEYS=monitor-prod:dGVzdHB1YmtleTEx,monitor-dev:dGVzdHB1YmtleTIy
```

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

### Serialization Formats

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

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request headers
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket connection with Bearer token
2. Server validates token and establishes connection
3. Server sends events as they arrive from monitors
4. Client processes and stores events in Zustand state
5. Client UI renders session information from state

**Client-Side Handling**:
- WebSocket proxy configured in `client/vite.config.ts`
- State management via `useEventStore` hook (Zustand)
- Event type guards for safe type access in `client/src/types/events.ts`

**Connection Details**:
- Address/port: Configured via `PORT` environment variable (default: 8080)
- Persistent connection model
- No automatic reconnection (Phase 2)
- No message queuing (direct streaming)

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
- Supports home directory expansion (`~/.vibetea`)
- Uses `directories` crate for platform-specific paths
- Validates required variables on startup

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

## Error Handling & Validation

### Server-Side Error Handling

**Error Types** (from `server/src/error.rs`):
- `ConfigError` - Configuration loading/validation failures
- `ServerError` - Runtime errors (Auth, Validation, RateLimit, WebSocket, Internal)

**Validation Points**:
1. Configuration validation on startup (`config.rs`)
2. Event signature validation on POST
3. Event schema validation (serde untagged enum)
4. Bearer token validation on WebSocket connect

### Monitor-Side Error Handling

**Error Types** (from `monitor/src/error.rs`):
- Configuration errors (missing env vars, invalid paths)
- File watching errors (permission denied, path not found)
- HTTP request errors (connection refused, timeout)
- Cryptographic errors (invalid private key)

**Resilience**:
- Continues watching even if individual file operations fail
- Retries HTTP requests with exponential backoff (future enhancement)
- Logs errors via `tracing` crate

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

**No External Service Integration** (Phase 2):
- Logs to stdout/stderr only
- Future: Integration with logging services (e.g., ELK, Datadog)

## Security Considerations

### Cryptographic Authentication

**Ed25519 Signatures**:
- Library: `ed25519-dalek` crate
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
