# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Authentication

### Authentication Method

| Method | Implementation | Configuration |
|--------|----------------|---------------|
| Monitor API | Ed25519 signature verification (RFC 8032 strict) | `server/src/auth.rs` |
| WebSocket Client | Bearer token with constant-time comparison | `server/src/auth.rs` |

### Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Signature algorithm | Ed25519 (RFC 8032 compliant via `verify_strict()`) | `server/src/auth.rs:231` |
| Signing method | `ed25519_dalek::VerifyingKey::verify_strict()` | `server/src/auth.rs:45,230-232` |
| Token comparison | Constant-time via `subtle::ConstantTimeEq` | `server/src/auth.rs:290` |
| Signature encoding | Base64 standard | `server/src/routes.rs:874` |
| Public key encoding | Base64 standard (32 bytes) | `server/src/config.rs:48-49` |

### Monitor Authentication Flow

1. Monitor signs request body with Ed25519 private key
2. Sends `X-Source-ID` header with monitor identifier
3. Sends `X-Signature` header with base64-encoded signature
4. Server verifies signature using registered public key
5. Server validates event.source matches X-Source-ID header

### Session Management

| Setting | Value |
|---------|-------|
| WebSocket authentication | Query parameter token validation (`?token=xxx`) |
| Token validation | Case-sensitive, constant-time comparison |
| Session duration | Determined by WebSocket connection lifetime |
| Idle timeout | TCP keepalive (OS-level) |

## Authorization

### Authorization Model

| Model | Description |
|-------|-------------|
| Source-based access control | Each monitor has unique `source_id` with registered Ed25519 public key |
| Token-based access control | WebSocket clients authenticate with shared subscriber token |
| Event source validation | Event payload source must match authenticated X-Source-ID header |

### Permission Enforcement Points

| Location | Pattern | Implementation |
|----------|---------|-----------------|
| Event ingestion | Signature verification | `server/src/routes.rs:293-307` |
| Event source validation | Cross-check header vs payload | `server/src/routes.rs:348-365` |
| WebSocket connection | Token validation | `server/src/routes.rs:458-491` |
| Rate limiting | Per-source token bucket | `server/src/rate_limit.rs` |

## Input Validation

### Validation Strategy

| Layer | Method | Implementation |
|-------|--------|-----------------|
| JSON deserialization | Type-safe serde deserialization | `server/src/routes.rs:329-342` |
| Header validation | Non-empty string checks | `server/src/routes.rs:263-290` |
| Signature verification | Base64 decode + Ed25519 verification | `server/src/auth.rs:192-233` |
| Request body | Size limit (1 MB max) | `server/src/routes.rs:72,182` |
| Public key format | Base64 decode + 32-byte length validation | `server/src/auth.rs:204-211` |

### Sanitization

| Data Type | Method | Location |
|-----------|--------|----------|
| File paths in tools | Basename extraction (no full paths) | `monitor/src/privacy.rs:445-454` |
| Sensitive tool context | Bash/Grep/Glob patterns stripped | `monitor/src/privacy.rs:380-382` |
| Source code content | Tool context redacted | `monitor/src/privacy.rs:378-401` |
| Prompts/responses | Completely stripped from payloads | `monitor/src/privacy.rs:352-355` |
| Shell commands | Never transmitted | `monitor/src/privacy.rs:63` (Bash in SENSITIVE_TOOLS) |
| Search patterns | Never transmitted | `monitor/src/privacy.rs:63` (Grep in SENSITIVE_TOOLS) |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Location |
|-----------|-------------------|----------|
| Ed25519 private keys | File storage (file mode 0600 by user) | `monitor/src/config.rs:76` |
| Ed25519 public keys | Base64-encoded in environment variable | `server/src/config.rs:48-49` |
| Bearer token | Environment variable, constant-time comparison | `server/src/auth.rs:269-294` |
| Source code | Privacy pipeline strips before transmission | `monitor/src/privacy.rs:222-372` |
| Event payloads | In-memory broadcasting only (no persistence) | `server/src/broadcast.rs` |

### Cryptography

| Type | Algorithm | Implementation | Key Management |
|------|-----------|-----------------|-----------------|
| Signature verification | Ed25519 | `ed25519_dalek::VerifyingKey::verify_strict()` | Public keys from `VIBETEA_PUBLIC_KEYS` |
| Token comparison | Constant-time | `subtle::ConstantTimeEq` | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Transport security | HTTPS/TLS 1.3 | Configured at reverse proxy | Deployment responsibility |

## Privacy Controls

### Privacy Pipeline (Monitor)

The monitor implements a comprehensive privacy pipeline to ensure no sensitive data is transmitted:

| Aspect | Implementation | Location |
|--------|----------------|----------|
| Path sanitization | Full paths reduced to basenames | `monitor/src/privacy.rs:445-454` |
| Sensitive tool stripping | Bash, Grep, Glob, WebSearch, WebFetch context set to None | `monitor/src/privacy.rs:55-63,380-382` |
| Extension allowlist | Optional filter via `VIBETEA_BASENAME_ALLOWLIST` | `monitor/src/privacy.rs:136-158` |
| Summary text redaction | All summary text replaced with "Session ended" | `monitor/src/privacy.rs:352-355` |
| Enhanced events pass-through | Activity, Agent, SessionMetrics, TokenUsage, etc. transmit as-is | `monitor/src/privacy.rs:326-371` |

### Privacy Guarantees for Enhanced Events (Phase 9-10)

All new events from Phase 9 and 10 transmit metadata only with no sensitive content:

| Event Type | Privacy Status | Data Transmitted | Location |
|-----------|----------------|-----------------|----------|
| ActivityPatternEvent | Metadata-only | Hourly activity counts (0-23 as string keys) | `monitor/src/trackers/stats_tracker.rs:492-507` |
| ModelDistributionEvent | Metadata-only | Model names + aggregated token counts | `monitor/src/trackers/stats_tracker.rs:509-538` |
| SessionMetricsEvent | Metadata-only | Global session counts, message counts, tool usage counts | `monitor/src/trackers/stats_tracker.rs:471-490` |
| TokenUsageEvent | Metadata-only | Model name + token counts by type | `monitor/src/trackers/stats_tracker.rs:540-561` |

All events follow established privacy patterns: **no code, prompts, commands, or user content transmitted**.

### ActivityPatternEvent Privacy Details

- **hour_counts**: Map of hour (0-23 as string) to activity count
- **No PII**: Only aggregated hourly metrics
- **No content**: Zero chance of code or prompt extraction
- **Type-safe**: ActivityPatternEvent struct contains only hour_counts field

### ModelDistributionEvent Privacy Details

- **model_usage**: Map of model name to TokenUsageSummary
- **TokenUsageSummary fields**: input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens
- **No PII**: Only token counts aggregated by model
- **No content**: Zero chance of prompt or output extraction
- **Type-safe**: ModelDistributionEvent struct contains only model_usage field

## Rate Limiting

| Endpoint | Limit | Window | Tracked By |
|----------|-------|--------|-----------|
| POST /events | 100 requests/second | Per-source rolling | `X-Source-ID` header |
| GET /ws | No rate limit | N/A | N/A |
| GET /health | No rate limit | N/A | N/A |

### Rate Limiter Details

- **Algorithm**: Token bucket with per-source tracking
- **Rate**: 100 tokens/second (configurable via `RateLimiter::new()`)
- **Burst**: 100 tokens initial capacity (configurable)
- **Cleanup**: Stale entries removed after 60 seconds of inactivity
- **Implementation**: Thread-safe `RwLock<HashMap>` with background cleanup task
- **Retry-After**: Returns `Retry-After` header on 429 response

## Secrets Management

### Environment Variables

| Variable | Required | Format | Usage |
|----------|----------|--------|-------|
| `VIBETEA_PUBLIC_KEYS` | Yes (if auth enabled) | `source1:pubkey1,source2:pubkey2` | Monitor public keys |
| `VIBETEA_SUBSCRIBER_TOKEN` | Yes (if auth enabled) | Any string | WebSocket client token |
| `PORT` | No | Numeric (default: 8080) | HTTP server port |
| `VIBETEA_UNSAFE_NO_AUTH` | No | "true" to disable | Dev-only auth bypass |
| `RUST_LOG` | No | Log filter (default: info) | Logging level |
| `VIBETEA_BASENAME_ALLOWLIST` | No | `.rs,.ts,.md` comma-separated | Privacy extension filter |

### Secrets Storage

| Environment | Method |
|-------------|--------|
| Development | Environment variables (shell/Docker) |
| CI/CD | GitHub Actions secrets or vault |
| Production | Container orchestrator (Kubernetes, AWS Secrets Manager) |
| Monitor local | File-based key storage in `~/.vibetea/` |

### Configuration Validation

- Public key format: Base64 decode + 32-byte Ed25519 key validation
- Empty token rejection: `validate_token()` rejects empty strings
- Startup validation: Server fails fast on missing required secrets
- Warning logging: `VIBETEA_UNSAFE_NO_AUTH=true` logged as warning

## Security Headers

VibeTea server does not directly set security headers. Configure at reverse proxy/load balancer:

| Header | Recommended | Purpose |
|--------|-------------|---------|
| Content-Security-Policy | `default-src 'self'` | XSS protection |
| X-Frame-Options | `DENY` | Clickjacking protection |
| X-Content-Type-Options | `nosniff` | MIME sniffing protection |
| Strict-Transport-Security | `max-age=31536000` | HTTPS enforcement |

## CORS Configuration

VibeTea is designed for backend-to-backend communication. Configure CORS at reverse proxy level:

| Setting | Recommendation |
|---------|-----------------|
| Allowed origins | Restrict to known client IPs |
| Allowed methods | POST (events), GET (WebSocket, health) |
| Allowed headers | Content-Type, X-Source-ID, X-Signature |
| Credentials | Not applicable (auth via headers) |

## Audit Logging

| Event | Logged Data | Level | Location |
|-------|-------------|-------|----------|
| Signature verification failure | source, error type | warn | `routes.rs:294` |
| Rate limit exceeded | source, retry_after_secs | info | `routes.rs:314-318` |
| Invalid event format | source, parse error | debug | `routes.rs:332` |
| Event source mismatch | authenticated source, event source | warn | `routes.rs:350-355` |
| WebSocket connection | filter parameters | info | `routes.rs:494-497` |
| Unknown source | source_id attempted | warn | `routes.rs:294` |
| Configuration error | error details | error | `main.rs:53` |
| Server startup | port, auth mode, key count | info | `main.rs:74-79` |

All logging is structured JSON via `tracing` crate, configurable via `RUST_LOG`.

## Security Control Summary

### Strengths

1. **Ed25519 RFC 8032 Strict Verification**: `verify_strict()` ensures RFC 8032 compliance, preventing signature malleability attacks
2. **Constant-Time Token Comparison**: `subtle::ConstantTimeEq` prevents timing attacks on token validation
3. **Privacy-First Architecture**: Comprehensive privacy pipeline ensures no code, prompts, or commands transmitted
4. **Metadata-Only Enhanced Events**: Phase 9-10 events transmit only aggregated statistics (hourly counts, model names, token counts)
5. **Per-Source Rate Limiting**: Token bucket algorithm with independent limits per source
6. **Configuration Validation**: Required secrets validated on startup with fast failure
7. **Type-Safe JSON Parsing**: Serde deserialization enforces event schema
8. **Source Validation**: Event.source field cross-checked against X-Source-ID header
9. **Request Size Limits**: 1 MB maximum body size prevents resource exhaustion
10. **Structured Logging**: Production-ready JSON logging with configurable levels
11. **File Watcher Security**: UUID pattern matching prevents reading unintended files in todo tracker
12. **Type-Safe Privacy Enforcement**: Privacy struct fields prevent sensitive data extraction at compile-time

### Attack Vectors & Mitigations

| Vector | Risk | Mitigation |
|--------|------|-----------|
| Signature forgery | High | Ed25519 is cryptographically secure; `verify_strict()` prevents malleability |
| Timing attacks on token | Medium | Constant-time comparison via `subtle::ConstantTimeEq` |
| Token guessing | Low | No rate limiting on WebSocket auth attempts (deployment concern) |
| Malformed signatures | Low | Base64 validation + length checks before crypto operations |
| Source spoofing | Low | Cross-validation of X-Source-ID header and event.source field |
| Resource exhaustion | Low | Rate limiting (100 req/sec per source) + 1 MB body size limit |
| Privacy breach via events | Low | Privacy pipeline + type-safe event structs prevent sensitive data transmission |
| Configuration misconfiguration | Medium | Startup validation fails fast; warnings for unsafe mode |

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
