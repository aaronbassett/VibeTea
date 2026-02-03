# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Authentication

### Authentication Method

| Method | Implementation | Configuration |
|--------|----------------|---------------|
| Ed25519 Signatures | ed25519_dalek | `server/src/auth.rs` |
| Bearer Token (WebSocket) | Token validation | `server/src/auth.rs` |
| Environment Variables | Public key registration | `VIBETEA_PUBLIC_KEYS`, `VIBETEA_SUBSCRIBER_TOKEN` |

### Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Monitor authentication | Ed25519 digital signatures | X-Source-ID, X-Signature headers |
| WebSocket authentication | Bearer token (static string) | ?token query parameter |
| Signing algorithm | Ed25519 (RFC 8032 strict) | Uses `verify_strict()` |
| Key storage (monitor) | Ed25519 raw seed (32 bytes) | `~/.vibetea/key.priv` (mode 0600) |
| Public key encoding | Base64 standard | Registered via `VIBETEA_PUBLIC_KEYS` |

### Signature Verification

| Component | Detail |
|-----------|--------|
| Public key bytes | 32 bytes Ed25519 |
| Signature bytes | 64 bytes Ed25519 |
| Constant-time comparison | Used with `subtle::ConstantTimeEq` |
| Message content | Full HTTP request body (prevents tampering) |

### Session Management

| Setting | Value |
|---------|-------|
| Session type | Stateless per-request authentication |
| Token type | Non-expiring static bearer token (WebSocket only) |
| Storage | Environment variables + in-memory configuration |
| Timeout | No timeout (continuous WebSocket connection) |

## Authorization

### Authorization Model

| Model | Description |
|-------|-------------|
| Source-based | Event sources are identified by source_id and must match registered public keys |
| Token-based | WebSocket clients authenticate with a single shared token |
| No granular roles | All sources have same permissions; all WebSocket clients have same permissions |

### Permissions

| Actor | Permissions | Scope |
|-------|------------|-------|
| Registered monitor (source_id) | Submit events to POST /events | Events matching authenticated source_id |
| WebSocket client (valid token) | Subscribe to event stream | All events (with optional filtering by source/type/project) |
| Unknown monitor | Rejected at authentication stage | N/A |
| Invalid token | Rejected at authentication stage | N/A |

### Permission Checks

| Location | Pattern | Example |
|----------|---------|---------|
| API events endpoint | Signature verification | `server/src/routes.rs:293` |
| WebSocket endpoint | Token validation | `server/src/routes.rs:483` |
| Event validation | Source ID matching | `server/src/routes.rs:348` - Events must match authenticated source |

## Input Validation

### Validation Strategy

| Layer | Method | Implementation |
|-------|--------|-----------------|
| Event payload (API) | JSON deserialization | serde with custom Event types |
| Headers | String length and format checks | Empty string rejection |
| Request body | Size limit | 1 MB maximum (DefaultBodyLimit) |
| Event fields | Type validation | Timestamp, UUID, enum types enforced by serde |

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| Event JSON | Deserialization validates structure | `server/src/routes.rs:329` |
| Headers | Whitespace trimming + empty check | `server/src/auth.rs:270-276` |
| Source ID | Must not be empty | `server/src/config.rs:182-187` |
| Public key | Must not be empty, must be valid base64 | `server/src/config.rs:189-193` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Storage |
|-----------|-------------------|---------|
| Private keys (monitor) | Raw 32-byte seed file | `~/.vibetea/key.priv` (mode 0600, owner-only) |
| Public keys (server) | Base64-encoded format | Environment variable `VIBETEA_PUBLIC_KEYS` |
| Subscriber token | Plain string comparison | Environment variable `VIBETEA_SUBSCRIBER_TOKEN` |
| Event payload | No encryption at rest | In-memory broadcast channel |

### Encryption

| Type | Algorithm | Implementation |
|------|-----------|-----------------|
| In transit | TLS 1.3+ | Requires HTTPS/WSS in production |
| At rest | None | Events are in-memory only, not persisted |
| Signing | Ed25519 deterministic | Uses standard RFC 8032 implementation |

### Private Key Security

| Aspect | Implementation |
|--------|-----------------|
| Generation | Uses OS cryptographically secure RNG (`rand::rng()`) |
| File permissions | Unix mode 0600 (owner read/write only) |
| Format | Raw 32-byte seed, not PKCS#8 or other wrapper |
| Loading | Fails if file size != 32 bytes |
| Exposure | Monitored via `VIBETEA_PRIVATE_KEY` env var as alternative |

## Rate Limiting

| Endpoint | Default Limit | Configuration |
|----------|---------------|---------------|
| POST /events | 100 requests/sec | 100 token capacity per source |
| GET /ws | No per-connection limit | WebSocket is persistent subscription |
| Rate limiter cleanup | Every 30 seconds | Removes inactive sources after 60 seconds |

### Rate Limit Algorithm

| Aspect | Detail |
|--------|--------|
| Type | Token bucket algorithm |
| Granularity | Per source_id |
| Rate | 100 tokens/second (default) |
| Capacity | 100 tokens (allows bursts) |
| Response | 429 Too Many Requests with Retry-After header |

## Secrets Management

### Environment Variables

| Category | Naming | Example | Required |
|----------|--------|---------|----------|
| Monitor auth | `VIBETEA_SOURCE_ID` | monitor-1 | No (defaults to hostname) |
| Monitor keys | `VIBETEA_KEY_PATH` | ~/.vibetea | No (defaults) |
| Server public keys | `VIBETEA_PUBLIC_KEYS` | source1:base64key,source2:base64key | Yes (unless unsafe_no_auth) |
| WebSocket token | `VIBETEA_SUBSCRIBER_TOKEN` | secret-token-string | Yes (unless unsafe_no_auth) |
| Server URL (monitor) | `VIBETEA_SERVER_URL` | https://vibetea.fly.dev | Yes (required) |

### Secrets Storage

| Environment | Method |
|-------------|--------|
| Development | `.env` files (gitignored) or export statements |
| CI/CD | GitHub Secrets or equivalent |
| Production | Environment variables via container orchestration |
| Private keys | File-based (~/.vibetea/key.priv with mode 0600) |

### Key Provisioning

| Process | Location |
|---------|----------|
| Monitor key generation | `monitor/src/crypto.rs:100` - Crypto::generate() |
| Server registration | Manual environment variable setup |
| Public key fingerprint | `monitor/src/crypto.rs:263` - public_key_fingerprint() (8 chars) |

## Security Headers

| Header | Status | Note |
|--------|--------|------|
| X-Source-ID | Required | Must be non-empty string |
| X-Signature | Required (when auth enabled) | Base64-encoded Ed25519 signature |
| Content-Type | Assumed application/json | Not validated, accepted from client |
| Retry-After | Conditional | Only set on 429 responses |

## CORS Configuration

| Setting | Value |
|---------|-------|
| Allowed origins | All (no CORS policy enforced in codebase) |
| Allowed methods | POST (events), GET (ws, health) |
| Credentials | WebSocket token via query param |

## Audit Logging

| Event | Logged Data | Level |
|-------|-------------|-------|
| Authentication failures | source_id, error type | warn/debug |
| Signature verification failures | source_id, error details | warn |
| Rate limit exceeded | source_id, retry_after | info |
| WebSocket connect | filter params | info |
| WebSocket disconnect | - | info |
| Configuration errors | Missing variables | error |

---

## Development Mode Security

### VIBETEA_UNSAFE_NO_AUTH Mode

When `VIBETEA_UNSAFE_NO_AUTH=true`:
- All signature verification is skipped
- WebSocket token validation is skipped
- WARNING logged at startup
- Intended for local development only
- Located in `server/src/config.rs:94-99`

---

*This document defines active security controls. Review when adding authentication methods or cryptographic operations.*
