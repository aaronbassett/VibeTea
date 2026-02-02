# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-02
> **Last Updated**: 2026-02-02

## Authentication

### Authentication Method

| Method | Implementation | Configuration | Status |
|--------|----------------|---------------|--------|
| Ed25519 Signatures | ed25519-dalek library | `VIBETEA_PUBLIC_KEYS` env var | Integrated (Phase 2) |
| Bearer Token | Custom string token | `VIBETEA_SUBSCRIBER_TOKEN` env var | Integrated (Phase 2) |
| Development bypass | `VIBETEA_UNSAFE_NO_AUTH=true` | Environment variable | Development only |

### Authentication Flow

**Monitor to Server**:
- Monitor signs events with Ed25519 private key
- Signature sent with each event batch
- Server verifies using public key from `VIBETEA_PUBLIC_KEYS` mapping
- Public keys stored as base64-encoded Ed25519 keys
- Format: `source_id:base64_pubkey,...`

**Client to Server**:
- Client sends bearer token in Authorization header
- Token value from `VIBETEA_SUBSCRIBER_TOKEN` configuration
- Server validates token presence and value
- No token expiration mechanism implemented
- No per-client token scope differentiation

### Development Mode Bypass

| Setting | Impact | Location |
|---------|--------|----------|
| `VIBETEA_UNSAFE_NO_AUTH=true` | Disables all auth (dev only) | `server/src/config.rs:57-58` |
| Behavior | Accepts any client, any source | Validated in `Config::validate()` at `server/src/config.rs:108-126` |
| Logging | Warning logged on startup | `server/src/config.rs:94-98` |

**Warning**: This setting logs a warning but is not otherwise restricted. Production deployments must never enable this. Configuration validation is enforced with comprehensive tests in `server/src/config.rs:205-415`.

## Authorization

### Authorization Model

| Model | Implementation | Scope |
|-------|----------------|-------|
| Token-based | Bearer token presence check | Client access |
| Source verification | Public key verification via Ed25519 | Monitor identity |
| No granular RBAC | Not implemented | - |

### Permission Structure

- **Server accepts from**: Any monitor with registered public key (source_id matching)
- **Client receives from**: Any client with valid bearer token
- **Server publishes to**: All connected WebSocket clients
- **No resource-level permissions**: All clients see all events
- **No user-level isolation**: No per-user filtering of events

### Authorization Gaps

- No per-user or per-resource permissions
- No role-based access control (RBAC)
- No scope limitation on token capabilities
- All authenticated clients access identical data streams
- No event filtering based on source or topic

## Input Validation

### Validation Strategy

| Layer | Method | Implementation |
|-------|--------|-----------------|
| Event parsing | Deserialization validation | `serde` with Rust type system |
| Configuration | Structured parsing | `Config::from_env()` with validation |
| API input | Type safety | Rust compiler enforces |
| Field format validation | Manual checks | In config parsing functions |

### Event Validation (Server Types)

Event structure from `server/src/types.rs:1-163`:

- **EventType**: Enum-based (`session`, `activity`, `tool`, `agent`, `summary`, `error`)
- **EventPayload**: Untagged union with variant ordering for correct deserialization
- **Timestamp**: RFC 3339 UTC (`DateTime<Utc>`)
- **Session ID**: UUID format validated by chrono
- **Event ID**: Prefixed format (`evt_` + 20 alphanumeric chars)

All event fields are type-checked at deserialization via serde. Invalid JSON fails before reaching application logic.

### Configuration Validation (Server)

From `server/src/config.rs:79-202`:

- Port: Parsed as `u16` (1-65535) with error handling at `server/src/config.rs:142-151`
- Public keys: Format validation `source:base64key` at `server/src/config.rs:157-203`
- Empty field checks for source_id and pubkey at `server/src/config.rs:182-197`
- Whitespace trimming in key pairs
- Conditional validation: auth fields required unless `VIBETEA_UNSAFE_NO_AUTH=true` at `server/src/config.rs:108-126`
- Comprehensive test coverage: `server/src/config.rs:205-415`

### Configuration Validation (Monitor)

From `monitor/src/config.rs:79-143`:

- Server URL: Required, must be present (no format validation)
- Buffer size: Parsed as `usize` with error handling at `monitor/src/config.rs:119-125`
- File paths: Converted to PathBuf
- Allowlist: Comma-separated, filtered for empty entries at `monitor/src/config.rs:128-133`
- Source ID: Uses hostname as default fallback via `gethostname` crate

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| Event payloads | None - passed through | `server/src/types.rs` |
| Configuration strings | Trimmed in parsing | `config.rs` functions |
| File paths | None - OS handles | `monitor/src/config.rs` |
| Base64 keys | Assumed valid during parsing | `Config::parse_public_keys()` |

**Note**: Base64 public keys are not validated at parse time - validation happens during signature verification in actual cryptographic operations.

## Data Protection

### Sensitive Data Handling

| Data Type | Protection | Storage | Notes |
|-----------|-----------|---------|-------|
| Private key | File permissions | `~/.vibetea/key.priv` | Monitor loads from disk |
| Public key | Base64-encoded | Environment variable | On server |
| Bearer token | Environment variable | `VIBETEA_SUBSCRIBER_TOKEN` | In-memory, passed by clients |
| Event payloads | No encryption | Memory/transit | Sent over HTTPS/WSS only |

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
| Algorithm | Ed25519 | `ed25519-dalek` 2.1 |
| Key format | Base64-encoded public key | In `VIBETEA_PUBLIC_KEYS` |
| Signature verification | Per-event batch | Location TBD (implementation pending) |
| Dependencies | ed25519-dalek, base64, rand | Integrated in Phase 2 |

**Status**: Ed25519 dependencies are integrated and configured. Actual signature verification logic is pending implementation.

### Key Generation (Monitor)

- Private key generated separately (external tool or one-time setup)
- Stored as PEM or binary in `~/.vibetea/key.priv`
- Public key registered on server as base64 in `VIBETEA_PUBLIC_KEYS`
- Source ID must match monitor hostname or `VIBETEA_SOURCE_ID` override

## CORS Configuration

| Setting | Value | Purpose |
|---------|-------|---------|
| Allowed origins | Via tower-http | Not yet configured |
| Methods | GET, POST | Likely via tower-http |
| Headers | Authorization, Content-Type | Likely via tower-http |
| Credentials | true | If client auth needed |

**Status**: CORS is available via `tower-http` dependency but configuration is pending implementation.

## Rate Limiting

| Endpoint | Limit | Implementation |
|----------|-------|-----------------|
| Event ingestion | Not implemented | - |
| WebSocket connections | Not implemented | - |
| Global | Not implemented | - |

**Status**: Error type supports rate limiting (`ServerError::RateLimit` at `server/src/error.rs:84-94`) but middleware is not yet implemented.

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

## Error Handling Security

### Information Disclosure

| Error Type | Message Content | Risk |
|------------|-----------------|------|
| Auth failures | "authentication failed: {msg}" | Moderate - reveals auth failure |
| Validation errors | "validation error: {msg}" | Low - field-level errors acceptable |
| Rate limit | "rate limit exceeded for {source}" | Low - expected for clients |
| Config errors | "configuration validation failed" | Low - visible only to operator |
| Internal errors | "internal server error" | Low - no sensitive details exposed |

### Error Response Handling

Errors from `server/src/error.rs:67-236`:

- `ServerError::Auth` - Indicates auth failure without exposing mechanism details
- `ServerError::Validation` - Safe for clients to see
- `ServerError::RateLimit` - Includes source identifier for debugging
- `ServerError::Internal` - Generic message, details logged server-side
- `ServerError::WebSocket` - Connection issues without exposing internals

No SQL errors, path traversal details, or stack traces exposed to clients.

### Monitor Error Handling

Errors from `monitor/src/error.rs:26-60`:

- `MonitorError::Crypto` - Covers key loading and signature errors
- `MonitorError::Http` - HTTP communication failures
- `MonitorError::Io` - File system errors
- `MonitorError::Watch` - File watcher errors
- Clear error messages for debugging without exposing sensitive paths

## Audit Logging

| Event | Logged Data | Implementation |
|-------|-------------|-----------------|
| Auth failures | Via error messages | In error variants |
| Rate limiting | source identifier | In `RateLimit` variant |
| Configuration errors | At startup | Via `tracing::warn!` |
| Development mode enabled | Warning message | In `server/src/config.rs:94-98` |

**Status**: Basic error logging present. Comprehensive audit logging (user IP, timestamp, detailed action logs) not yet implemented.

## Security Headers

Not yet configured. Recommended headers for production:

| Header | Recommended Value | Purpose |
|--------|-------------------|---------|
| Strict-Transport-Security | `max-age=31536000; includeSubDomains` | HTTPS enforcement |
| X-Content-Type-Options | `nosniff` | MIME sniffing prevention |
| X-Frame-Options | `DENY` | Clickjacking protection |
| Content-Security-Policy | `default-src 'self'` | XSS protection |

**Action needed**: Configure via tower-http middleware before production.

## Dependency Security

### Core Security Dependencies

| Package | Version | Purpose | Status |
|---------|---------|---------|--------|
| ed25519-dalek | 2.1 | Cryptographic signing | Current |
| base64 | 0.22 | Key encoding | Current |
| reqwest | 0.12 | HTTPS client | Current |
| serde | 1.0 | Safe deserialization | Current |
| rand | 0.9 | Random number gen | Current |
| thiserror | Latest | Error handling | Current |
| uuid | Latest | Session IDs | Current |
| chrono | Latest | Timestamps | Current |

### Dependency Audit

**Status**: No known CVEs in current dependencies (as of 2026-02-02).

**Process**: Use `cargo audit` to check for vulnerabilities:
```bash
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

## Known Vulnerabilities & Gaps

- No rate limiting middleware implemented (pending)
- No granular authorization/RBAC (design phase)
- No encryption at rest for configuration/events (acceptable for MVP)
- No audit logging beyond error messages (pending)
- No CORS header configuration (pending)
- Base64 public key validation happens during use, not parsing
- No client-side token management (pending)
- No per-client isolation or scoping

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
