# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Authentication

### Authentication Method

| Method | Implementation | Configuration |
|--------|----------------|---------------|
| Ed25519 Signatures | ed25519-dalek with verify_strict() for RFC 8032 compliance | `server/src/auth.rs` |
| Bearer Token (WebSocket) | Constant-time string comparison using `subtle::ConstantTimeEq` | `server/src/auth.rs` (lines 269-295) |

### Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Monitor auth | Ed25519 signatures on request body | `X-Source-ID` and `X-Signature` headers |
| Signature algorithm | Ed25519 with strict RFC 8032 verification | `server/src/auth.rs` (line 231) |
| Client auth token type | Bearer token string (any format) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Client token transmission | Query parameter in WebSocket URL | `client/src/hooks/useWebSocket.ts` (line 77) |
| Client token storage | Browser localStorage with password input masking | `client/src/hooks/useWebSocket.ts` (line 18) |

### Session Management

| Setting | Value |
|---------|-------|
| Session storage | In-memory event broadcaster; no persistent user sessions |
| WebSocket persistence | Automatic exponential backoff reconnection (1s-60s with jitter) |
| Idle timeout | Browser manages timeout; server-side connection can be long-lived |
| Token TTL | No expiration implemented; static bearer token |

## Authorization

### Authorization Model

| Model | Description |
|-------|-------------|
| Source-based signature verification | Each monitor has unique source_id and Ed25519 public key pair |
| Bearer token validation | WebSocket clients share single subscriber token for read-only access |

### Roles & Permissions

| Entity | Permissions | Scope |
|--------|-------------|-------|
| Monitor (Authenticated via signature) | Submit events via POST /events | Event publication only |
| WebSocket Client (Authenticated via token) | Subscribe to events via /ws with optional filtering | Event consumption only |

### Permission Checks

| Location | Pattern | Example |
|----------|---------|---------|
| Event submission | Header validation + Ed25519 signature verification | `server/src/routes.rs:261-290` |
| WebSocket upgrade | Bearer token validation before upgrade | `server/src/routes.rs:452-501` |
| Rate limiting | Per-source-id token bucket algorithm | `server/src/rate_limit.rs` |

## Input Validation

### Validation Strategy

| Layer | Method | Library |
|-------|--------|---------|
| API headers | Required header presence and non-empty validation | Custom validation in `server/src/routes.rs` |
| Event payload | JSON deserialization with typed Event struct | `serde_json` with `types::Event` schema |
| Config parsing | Format validation for environment variables | Custom parser in `server/src/config.rs` (lines 154-200) |
| Token validation | Whitespace trim + empty string rejection + constant-time comparison | `server/src/auth.rs` (lines 269-295) |

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| Base64 encoded signatures | Decode and validate 64-byte length | `server/src/auth.rs:218-225` |
| Base64 encoded public keys | Decode and validate 32-byte Ed25519 format | `server/src/auth.rs:204-215` |
| Source identifiers | Non-empty string validation during config parse | `server/src/config.rs:179-187` |
| WebSocket messages (client) | Structural JSON validation of required fields | `client/src/hooks/useWebSocket.ts:110-118` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Storage |
|-----------|-------------------|---------|
| Ed25519 public keys | Base64 encoded in trusted configuration | Environment variable `VIBETEA_PUBLIC_KEYS` |
| Bearer token (server) | Environment variable, no logging | Environment variable `VIBETEA_SUBSCRIBER_TOKEN` |
| Bearer token (client) | Password input type; localStorage storage | Browser localStorage (accessed via `TOKEN_STORAGE_KEY`) |
| Event payloads | No persistence; in-memory broadcast only | Ephemeral in-memory distribution |
| Signatures | Base64 encoded, verified but not stored | Transient; used only for verification |

### Encryption

| Type | Algorithm | Key Management |
|------|-----------|----------------|
| Signature verification | Ed25519 with RFC 8032 strict mode | Public keys from configuration |
| Token comparison | Constant-time comparison (no encryption) | Timing attack resistance |
| Transport security | TLS 1.3 (enforced by client via wss://) | Managed by deployment reverse proxy |
| At-rest encryption | None (events not persisted) | N/A |

## Security Headers

These headers are NOT configured in the application itself. They must be configured at reverse proxy/deployment level:

| Header | Recommended Value | Purpose |
|--------|-------------------|---------|
| Content-Security-Policy | `default-src 'self'; script-src 'self' 'wasm-unsafe-eval'` | XSS protection for React client |
| X-Frame-Options | `DENY` | Clickjacking protection |
| X-Content-Type-Options | `nosniff` | MIME sniffing protection |
| Strict-Transport-Security | `max-age=31536000; includeSubDomains` | HTTPS enforcement |

**Implementation note**: Axum has `tower-http` CORS feature available but not actively configured in `server/src/routes.rs`. Configuration should be added or done at reverse proxy.

## CORS Configuration

| Setting | Current Value |
|---------|---------------|
| CORS headers | Not configured in application |
| Allowed origins | Browser enforces same-origin for WebSocket |
| Methods | GET (health, /ws), POST (/events) |
| Credentials | Query parameter token (not HTTP credentials) |

**Note**: CORS should be explicitly configured at reverse proxy level.

## Rate Limiting

Rate limiting implemented per source-id using token bucket algorithm:

| Endpoint | Limit | Window | Tracked By |
|----------|-------|--------|-----------|
| POST /events | 100 requests | Per second | Source-ID header |
| GET /ws | No limit | N/A | No per-connection limiting |
| GET /health | No limit | N/A | No limiting |

### Configuration

| Parameter | Value | Location |
|-----------|-------|----------|
| Rate | 100.0 tokens/second | `server/src/rate_limit.rs:43` |
| Burst capacity | 100 tokens | `server/src/rate_limit.rs:46` |
| Stale cleanup | 60 seconds | `server/src/rate_limit.rs:49` |
| Storage | In-memory RwLock HashMap | `server/src/rate_limit.rs` |

## Secrets Management

### Environment Variables

| Category | Variable | Required | Format | Example |
|----------|----------|----------|--------|---------|
| Auth bypass | `VIBETEA_UNSAFE_NO_AUTH` | No | "true" to disable auth | (dev only) |
| Public keys | `VIBETEA_PUBLIC_KEYS` | Yes if auth enabled | `source:base64key,source2:base64key2` | See config.rs |
| Subscriber token | `VIBETEA_SUBSCRIBER_TOKEN` | Yes if auth enabled | Any string | Any 32+ char string recommended |
| Server port | `PORT` | No | Numeric | 8080 (default) |
| Logging level | `RUST_LOG` | No | Log filter | info (default) |

### Secrets Storage by Environment

| Environment | Method | Notes |
|-------------|--------|-------|
| Development | Environment variable or shell export | `VIBETEA_UNSAFE_NO_AUTH=true` bypasses auth entirely |
| CI/CD | GitHub Actions secrets | Never commit secrets to repository |
| Production | Fly.io environment secrets | Runtime variable injection |

## Audit Logging

| Event | Logged Data | Location | Level |
|-------|-------------|----------|-------|
| Server startup | Port, auth mode, public_key_count | `server/src/main.rs:74-79` | info |
| Auth failures | Source-ID, error type | `server/src/routes.rs` | debug/warn |
| Rate limit exceeded | Source-ID, retry_after_secs | `server/src/routes.rs` | info |
| WebSocket connection | Filter configuration | `server/src/routes.rs:494-497` | info |
| WebSocket disconnection | (minimal) | `server/src/routes.rs:578` | info |

**Logging framework**: Structured JSON output via `tracing` crate; controllable via `RUST_LOG` env var.

---

## Critical Security Patterns

### Ed25519 Signature Verification (RFC 8032 Compliant)

The implementation uses cryptographic best practices:

- **Strict RFC 8032 verification**: Uses `VerifyingKey::verify_strict()` instead of lenient mode
- **Comprehensive error handling**: Distinct errors for unknown source, invalid signature, malformed keys
- **Test coverage**: 20+ test cases covering invalid inputs, edge cases, multiple sources

**Code location**: `server/src/auth.rs:192-233`

**Key test cases**:
- Valid signature verification
- Tampered message detection
- Wrong key rejection
- Invalid base64 handling
- Identity point rejection

### Constant-Time Token Comparison

Bearer token validation uses timing-attack-resistant comparison:

```rust
// server/src/auth.rs:290
if provided_bytes.ct_eq(expected_bytes).into() {
    Ok(())
} else {
    Err(AuthError::InvalidToken)
}
```

This prevents attackers from guessing tokens byte-by-byte based on response timing.

### Test Parallelism Safety

Environment variable tests use RAII pattern to prevent interference:

- Tests marked with `#[serial]` attribute
- `EnvGuard` struct saves/restores environment state
- CI runs with `--test-threads=1` to prevent race conditions

**Location**: `server/src/config.rs:209-240`

---

## Known Security Controls

### Implemented Defenses

1. **Input validation**: Non-empty headers, base64 validation, JSON type checking
2. **Cryptographic verification**: RFC 8032 compliant Ed25519 with strict verification
3. **Rate limiting**: Per-source token bucket (100 req/sec capacity)
4. **Constant-time comparison**: Token validation resists timing attacks
5. **Request size limits**: 1 MB maximum body size
6. **Configuration validation**: Public key format and presence checks

### Missing/Incomplete Controls

See CONCERNS.md for:
- Token expiration mechanism
- HTTPS enforcement (must be at reverse proxy)
- Security headers configuration
- Per-connection WebSocket rate limiting
- Distributed rate limiting across multiple instances

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
