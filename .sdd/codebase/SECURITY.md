# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-02
> **Last Updated**: 2026-02-02

## Authentication

### Authentication Method

| Method | Implementation | Configuration | Status |
|--------|----------------|---------------|--------|
| Ed25519 Signatures | ed25519-dalek library with verify_strict() | `VIBETEA_PUBLIC_KEYS` env var | Implemented (Phase 3) |
| Bearer Token | Constant-time comparison in validate_token() | `VIBETEA_SUBSCRIBER_TOKEN` env var | Implemented (Phase 3) |
| Development bypass | `VIBETEA_UNSAFE_NO_AUTH=true` | Environment variable | Development only |

### Authentication Flow

**Monitor to Server (Event Ingestion)**:
- Monitor signs event payload with Ed25519 private key using `SigningKey::sign()`
- Signature sent as base64-encoded value in `X-Signature` header
- Source ID sent in `X-Source-ID` header for public key lookup
- Server verifies using `verify_signature()` function in `server/src/auth.rs:192-233`
- Verification uses `verify_strict()` for strict Ed25519 verification per RFC 8032
- Public keys stored as base64-encoded Ed25519 keys (32 bytes) in `VIBETEA_PUBLIC_KEYS` mapping
- Format: `source_id:base64_pubkey,...` parsed in `server/src/config.rs:157-203`

**Client to Server (WebSocket)**:
- Client sends bearer token in `token` query parameter (e.g., `?token=xxx`)
- Token value from `VIBETEA_SUBSCRIBER_TOKEN` configuration
- Server validates token presence and value using `validate_token()` in `server/src/auth.rs:269-295`
- Comparison uses constant-time bit comparison via `subtle::ConstantTimeEq` to prevent timing attacks
- No token expiration mechanism implemented (planned for Phase 3+)
- No per-client token scope differentiation

### Development Mode Bypass

| Setting | Impact | Location |
|---------|--------|----------|
| `VIBETEA_UNSAFE_NO_AUTH=true` | Disables all auth (dev only) | `server/src/config.rs:57-58` |
| Behavior | Accepts any client, any source, any token | Validated in `Config::validate()` at `server/src/config.rs:108-126` |
| Logging | Warning logged on startup | `server/src/config.rs:94-98` |
| Route enforcement | Auth skipped in `server/src/routes.rs:272-304` | Conditional check before `verify_signature()` |

**Warning**: This setting logs a warning but is not otherwise restricted. Production deployments must never enable this. Configuration validation is enforced with comprehensive tests in `server/src/config.rs:205-415`.

### Signature Verification Details

From `server/src/auth.rs:192-233`:

1. **Source lookup**: Retrieves public key for source_id from `VIBETEA_PUBLIC_KEYS` map
2. **Key decoding**: Decodes base64 public key (must be exactly 32 bytes for Ed25519)
3. **Key parsing**: Constructs `VerifyingKey` from decoded bytes
4. **Signature decoding**: Decodes base64 signature (must be exactly 64 bytes)
5. **Verification**: Uses `verify_strict()` for RFC 8032 strict verification
6. **Error classification**: Returns specific `AuthError` variants for each failure mode

### Token Validation Details

From `server/src/auth.rs:269-295`:

1. **Trimming**: Leading/trailing whitespace removed from both tokens
2. **Empty check**: Both tokens must be non-empty after trimming
3. **Length check**: Token lengths must match (not constant-time, acceptable)
4. **Bit comparison**: `ct_eq()` constant-time comparison prevents timing attacks
5. **Error handling**: Returns `AuthError::InvalidToken` on any mismatch

## Authorization

### Authorization Model

| Model | Implementation | Scope |
|-------|----------------|-------|
| Token-based | Bearer token presence check via `validate_token()` | Client access to WebSocket |
| Source verification | Public key verification via Ed25519 signature | Monitor identity for events |
| No granular RBAC | Not implemented | - |

### Permission Structure

- **Server accepts from**: Any monitor with registered public key (source_id matching in `VIBETEA_PUBLIC_KEYS`)
- **Client receives from**: Any client with valid bearer token matching `VIBETEA_SUBSCRIBER_TOKEN`
- **Server publishes to**: All connected WebSocket clients equally
- **No resource-level permissions**: All clients see all events
- **No user-level isolation**: No per-user filtering of events
- **WebSocket filtering available**: Clients can filter by source/type/project via query parameters (advisory, not enforced)

### Authorization Gaps

- No per-user or per-resource permissions
- No role-based access control (RBAC)
- No scope limitation on token capabilities
- All authenticated clients access identical data streams
- No server-side enforcement of client-specified filters

## Input Validation

### Validation Strategy

| Layer | Method | Implementation |
|-------|--------|-----------------|
| Event parsing | Deserialization validation | `serde` with Rust type system |
| Configuration | Structured parsing | `Config::from_env()` with validation |
| API input | Type safety | Rust compiler enforces types |
| Signature/Token validation | Cryptographic checks | `verify_signature()` and `validate_token()` |
| JSONL parsing | JSON deserialization + error handling | `monitor/src/parser.rs:354-359` |

### Event Validation (Server Types)

Event structure from `server/src/types.rs:1-163`:

- **EventType**: Enum-based (`session`, `activity`, `tool`, `agent`, `summary`, `error`)
- **EventPayload**: Untagged union with variant ordering for correct deserialization
- **Timestamp**: RFC 3339 UTC (`DateTime<Utc>`)
- **Session ID**: UUID format validated by chrono
- **Event ID**: Prefixed format (`evt_` + 20 alphanumeric chars)

All event fields are type-checked at deserialization via serde. Invalid JSON fails before reaching application logic. Validation occurs at `server/src/routes.rs:325-338`.

### Claude Code JSONL Parsing

From `monitor/src/parser.rs`:

- **Privacy-first design**: Only metadata extracted (tool names, timestamps, file basenames)
- **Code content excluded**: Prompts, response text, tool results never extracted
- **Structure validation**: Raw JSON deserialization validates format
- **Event type mapping**: Untagged enum ensures only recognized events processed
- **Path safety**: File basenames extracted without full paths via `extract_basename()`
- **URL decoding**: Project names decoded safely with validation
- **Error handling**: Malformed JSON lines skipped with warning logs (line 357)

### Configuration Validation (Server)

From `server/src/config.rs:79-202`:

- Port: Parsed as `u16` (1-65535) with error handling at `server/src/config.rs:142-151`
- Public keys: Format validation `source:base64key` at `server/src/config.rs:157-203`
- Empty field checks for source_id and pubkey at `server/src/config.rs:182-197`
- Whitespace trimming in key pairs
- Conditional validation: auth fields required unless `VIBETEA_UNSAFE_NO_AUTH=true` at `server/src/config.rs:108-126`
- Comprehensive test coverage: `server/src/config.rs:205-415`

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| Event payloads | None - passed through | `server/src/routes.rs:344-352` |
| Configuration strings | Trimmed in parsing | `config.rs` functions |
| File paths | None - OS handles | `monitor/src/config.rs` |
| Base64 keys | Validated during signature verification | `auth.rs:204-215` |
| Signatures | Base64 decoding with error handling | `auth.rs:218-225` |
| Tokens | Trimmed and length-checked | `auth.rs:270-287` |
| JSONL lines | Whitespace trimmed, empty lines filtered | `monitor/src/parser.rs:348-350`, `monitor/src/watcher.rs:562-565` |
| File paths from tool input | Basename extraction only | `monitor/src/parser.rs:465-488` |
| Project names | URL decoding with validation | `monitor/src/parser.rs:491-529` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection | Storage | Notes |
|-----------|-----------|---------|-------|
| Private key | File permissions | `~/.vibetea/key.priv` | Monitor loads from disk |
| Public key | Base64-encoded | Environment variable | On server |
| Bearer token | Environment variable | `VIBETEA_SUBSCRIBER_TOKEN` | In-memory, passed by clients in query params |
| Event payloads | No encryption | Memory/transit | Sent over HTTPS/WSS only |
| JSONL data | Read from disk | `~/.claude/projects/` | Watched by monitor, only metadata extracted |

### Encryption in Transit

| Channel | Protocol | Implementation |
|---------|----------|-----------------|
| Monitor → Server | HTTPS | TLS 1.2+ required (enforced by reqwest) |
| Server → Client | WSS (WebSocket Secure) | TLS 1.2+ (depends on deployment) |

**Deployment note**: VibeTea server endpoints must be served over HTTPS/WSS. Currently no explicit header configuration for security headers (HSTS, CSP, etc.).

### Encryption at Rest

| Data | Encryption | Key Management |
|------|-----------|-----------------|
| Event payloads | None | Not applicable |
| Private keys | None (file permissions) | OS filesystem security |
| Configuration | None | Environment variables |

**Note**: VibeTea does not implement application-level encryption. Sensitive credentials are protected by:
1. Environment variable isolation
2. File system permissions (private key in `~/.vibetea/key.priv`)
3. HTTPS/WSS transport security

## Cryptography

### Signature Scheme

| Parameter | Value | Implementation |
|-----------|-------|-----------------|
| Algorithm | Ed25519 | `ed25519-dalek` 2.1 with `verify_strict()` |
| Key format | Base64-encoded public key | In `VIBETEA_PUBLIC_KEYS` |
| Signature verification | Per-event during POST /events | `server/src/routes.rs:289` |
| Constant-time token comparison | Via `subtle::ConstantTimeEq` | `server/src/auth.rs:290` |
| Dependencies | ed25519-dalek, base64, subtle | Production-ready (Phase 3) |

**Status**: Ed25519 signature verification fully implemented and tested. Token comparison uses constant-time comparison to prevent timing attacks.

### Verification Implementation

From `server/src/auth.rs`:

- `verify_signature()`: Handles full verification flow (lines 192-233)
  - Returns specific `AuthError` types for debugging
  - Validates key length and format
  - Uses strict verification per RFC 8032
  - Tested with 13 comprehensive test cases (lines 334-586)

- `validate_token()`: Handles constant-time token comparison (lines 269-295)
  - Trims whitespace from both sides
  - Performs length check first (acceptable, not timing-sensitive)
  - Uses `ct_eq()` for byte-level constant-time comparison
  - Tested with 15 test cases covering edge cases (lines 656-757)

### Key Generation (Monitor)

- Private key generated separately (external tool or one-time setup)
- Stored as binary in `~/.vibetea/key.priv`
- Public key registered on server as base64 in `VIBETEA_PUBLIC_KEYS`
- Source ID must match monitor hostname or `VIBETEA_SOURCE_ID` override

## Rate Limiting

### Implementation

| Aspect | Details | Location |
|--------|---------|----------|
| Algorithm | Token bucket | `server/src/rate_limit.rs:94-196` |
| Rate | Configurable (default 100 tokens/sec) | `server/src/rate_limit.rs:42-46` |
| Capacity | Configurable (default 100 tokens) | `server/src/rate_limit.rs:42-46` |
| Per-source tracking | `HashMap<String, TokenBucket>` | `server/src/rate_limit.rs:233-243` |
| Granularity | Per X-Source-ID header | `server/src/routes.rs:307` |
| Status | Fully implemented and integrated | `server/src/main.rs:84-91` |

### Rate Limiter Details

From `server/src/rate_limit.rs`:

**Token Bucket Algorithm**:
- Each source gets independent bucket with `capacity` tokens
- Tokens refill at `rate` tokens per second
- Each request consumes 1 token
- No tokens = request rejected with 429 Too Many Requests
- Retry-After header indicates seconds until next token available

**RateLimiter Structure**:
- Thread-safe via `RwLock` for concurrent access
- Automatic bucket creation on first request per source
- Stale entry cleanup every 60 seconds (configurable via `cleanup_stale_entries_with_timeout()`)
- Background cleanup task spawned at server startup (line 85-91 in main.rs)

**Integration**:
- Middleware check in `server/src/routes.rs:306-322`
- Returns 429 with `Retry-After` header when limited
- Applied to all event ingestion regardless of auth mode
- Independent per source ID to prevent cross-source throttling

### Response Handling

From `server/src/routes.rs:307-322`:

```
If allowed: Continue to authentication/processing
If limited: Return 429 Too Many Requests
  - Header: Retry-After: {seconds}
  - Body: {"error": "rate limit exceeded", "code": "rate_limited"}
  - Log: info!(source, retry_after, "Rate limit exceeded")
```

## CORS Configuration

| Setting | Value | Purpose |
|---------|-------|---------|
| Allowed origins | Via tower-http (not configured) | Not yet configured |
| Methods | GET, POST | Likely via tower-http |
| Headers | Authorization, Content-Type | Likely via tower-http |
| Credentials | true | If client auth needed |

**Status**: CORS is available via `tower-http` dependency but configuration is pending implementation.

## Security Headers

Not yet configured. Recommended headers for production:

| Header | Recommended Value | Purpose |
|--------|-------------------|---------|
| Strict-Transport-Security | `max-age=31536000; includeSubDomains` | HTTPS enforcement |
| X-Content-Type-Options | `nosniff` | MIME sniffing prevention |
| X-Frame-Options | `DENY` | Clickjacking protection |
| Content-Security-Policy | `default-src 'self'` | XSS protection |

**Action needed**: Configure via tower-http middleware before production.

## Secrets Management

### Environment Variables

| Category | Variable | Required | Default | Notes |
|----------|----------|----------|---------|-------|
| Server - Auth | `VIBETEA_PUBLIC_KEYS` | Yes* | - | Format: `id1:b64key1,id2:b64key2` |
| Server - Auth | `VIBETEA_SUBSCRIBER_TOKEN` | Yes* | - | Bearer token for clients |
| Server - Config | `PORT` | No | 8080 | HTTP port |
| Server - Dev | `VIBETEA_UNSAFE_NO_AUTH` | No | false | Development bypass |
| Monitor - URL | `VIBETEA_SERVER_URL` | Yes | - | Server endpoint URL |
| Monitor - Identity | `VIBETEA_SOURCE_ID` | No | hostname | Monitor name |
| Monitor - Keys | `VIBETEA_KEY_PATH` | No | `~/.vibetea` | Directory with keys |
| Monitor - Watch | `VIBETEA_CLAUDE_DIR` | No | `~/.claude` | Claude Code directory |
| Monitor - Tuning | `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |
| Monitor - Filter | `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated extensions |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true` (dev only)

### Secrets Storage by Environment

| Environment | Method | Tool |
|-------------|--------|------|
| Development | `.env.local` | Gitignored local file |
| CI/CD | GitHub Secrets | Repository settings |
| Production | Environment variables | Deployment platform (Fly.io) |

**Note**: No `.env` example file included in repo. Secrets are documented here and set during deployment.

## Audit Logging

| Event | Logged Data | Implementation |
|-------|-------------|-----------------|
| Auth failures | Via error variants and warn logs | `server/src/routes.rs:290, 459` |
| Rate limiting | source identifier and retry_after | `server/src/routes.rs:310-314` |
| Configuration errors | At startup | Via `tracing::warn!` in config validation |
| Development mode enabled | Warning message | In `server/src/config.rs:94-98` |
| WebSocket connections | Connection/disconnection events | `server/src/routes.rs:493, 554` |
| Event broadcasts | Per-event trace logs | `server/src/routes.rs:345-350` |
| File watcher events | File creation/modification/removal | `monitor/src/watcher.rs:329-362, 366-400, 426-456` |
| Parser events | JSONL parsing failures logged as warnings | `monitor/src/parser.rs:357` |

**Status**: Basic error logging present. Structured auth decision logging and comprehensive audit trails pending.

## Error Handling Security

### Information Disclosure

| Error Type | Message Content | Risk |
|------------|-----------------|------|
| Auth failures | Specific error codes | Low - helpful for debugging |
| Unknown source | "unknown source: {source_id}" | Low - expected behavior |
| Invalid signature | "invalid signature" | Low - doesn't expose details |
| Base64 errors | "invalid base64 encoding for {field}" | Low - field name only |
| Rate limit | "rate limit exceeded" | Low - expected for clients |
| Config errors | "configuration validation failed" | Low - visible only to operator |
| Internal errors | "server configuration error" | Low - no sensitive details exposed |
| Parser errors | Logged as warnings without details | Low - JSON parsing failures don't expose content |

### Error Response Handling

Errors from `server/src/routes.rs:188-208` and `server/src/auth.rs:49-92`:

- `AuthError::UnknownSource` - Returns 401 "unknown source"
- `AuthError::InvalidSignature` - Returns 401 "invalid signature"
- `AuthError::InvalidBase64` - Returns 401 "invalid signature encoding"
- `AuthError::InvalidPublicKey` - Returns 500 "server configuration error"
- `AuthError::InvalidToken` - Returns 401 "invalid token"
- Rate limit errors - Returns 429 with Retry-After
- Parser errors - Logged as warnings, non-fatal to file monitoring

No SQL errors, path traversal details, or stack traces exposed to clients.

## Dependency Security

### Core Security Dependencies

| Package | Version | Purpose | Status |
|---------|---------|---------|--------|
| ed25519-dalek | 2.1 | Cryptographic signing/verification | Current, production |
| base64 | 0.22 | Key/signature encoding | Current |
| subtle | Latest | Constant-time comparison | Critical for security |
| reqwest | 0.12 | HTTPS client | Current |
| serde | 1.0 | Safe deserialization | Current |
| serde_json | Latest | JSON parsing | Current |
| rand | 0.9 | Random number gen | Current |
| tokio | Latest | Async runtime | Current |
| axum | Latest | HTTP framework | Current |
| thiserror | Latest | Error handling | Current |
| uuid | Latest | Session IDs | Current |
| chrono | Latest | Timestamps | Current |
| notify | Latest | File watching | Current (Phase 4) |
| directories | Latest | Home directory resolution | Current (Phase 4) |

### Dependency Audit

**Status**: No known CVEs in current dependencies (as of 2026-02-02).

**Process**: To check for vulnerabilities, install and run cargo-audit:
```bash
cargo install cargo-audit
cd /home/ubuntu/Projects/VibeTea && cargo audit
```

## Client-Side Security (TypeScript)

### Event Types and Validation

From `client/src/types/events.ts:1-248`:

- **EventType**: String discriminator union validated via `isValidEventType()`
- **VibeteaEvent**: Generic interface with type-safe payload mapping
- **Type guards**: Runtime validators for all event types
- **readonly fields**: Immutability for all payload interfaces

### State Management

From `client/src/hooks/useEventStore.ts:1-172`:

- **Zustand store**: Centralized state with selective subscriptions
- **Event buffer**: Last 1000 events with FIFO eviction (no size limit security risk)
- **Session tracking**: Derived from event stream aggregation
- **No authentication state**: Bearer token not stored in client-side store
- **No sensitive data**: Events are passed through without validation

### Client Authorization Gaps

- No token management implementation
- No authorization checks in event filtering
- No rate limiting on client side
- No CORS origin validation (server responsibility)

## Phase 4 Security Changes

New components added for Claude Code monitoring:

### File Watcher (`monitor/src/watcher.rs`)

- **Privacy-safe file monitoring**: Watches `~/.claude/projects/**/*.jsonl` for changes only
- **Position tracking**: Efficient tailing without re-reading via byte-offset map
- **Error handling**: Graceful handling of permission denied, I/O errors, missing files
- **Async implementation**: Uses tokio for non-blocking file operations
- **Thread safety**: `Arc<RwLock>` protects position map for concurrent access

Security considerations:
- Only JSONL files processed (extension filtering)
- Position map prevents event replay
- File removal cleanup automatic
- Errors logged but don't crash watcher

### JSONL Parser (`monitor/src/parser.rs`)

- **Privacy-first extraction**: Only metadata (tool names, file basenames, timestamps) extracted
- **Safe path handling**: `extract_basename()` prevents full path transmission
- **URL decoding**: Safe project name decoding with validation
- **Error resilience**: Malformed JSON skipped with warnings
- **Type safety**: Rust enums prevent invalid event kinds

Security guarantees:
- Code content never extracted (text blocks skipped)
- Prompts never extracted (thinking blocks skipped)
- Tool results never extracted
- Full file paths never transmitted (basenames only)
- Session start/end tracking for lifecycle management

## Known Vulnerabilities & Gaps

**Fixed in Phase 3:**
- Ed25519 signature verification fully implemented with strict verification
- Token comparison using constant-time comparison to prevent timing attacks
- Per-source rate limiting with token bucket algorithm
- Comprehensive error handling with specific AuthError variants

**Remaining gaps:**
- No rate limiting middleware for other endpoints (only event ingestion protected)
- No granular authorization/RBAC (design phase)
- No encryption at rest for configuration/events (acceptable for MVP)
- No comprehensive audit logging beyond error messages
- No CORS header configuration (pending)
- No client-side token management (pending)
- No per-client isolation or scoping (all clients see all events)
- No TLS certificate validation in monitor HTTP client (reqwest default)

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
