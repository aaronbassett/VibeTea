# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Authentication

### Authentication Method

| Method | Implementation | Configuration |
|--------|----------------|---------------|
| Ed25519 Signature | ed25519_dalek with RFC 8032 strict verification | `server/src/auth.rs` |
| Bearer Token | Constant-time comparison for WebSocket clients | `server/src/auth.rs` |

### Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Token type | Bearer token for WebSocket subscriptions | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Signature algorithm | Ed25519 (RFC 8032 compliant) | `ed25519_dalek` crate |
| Signature encoding | Base64 standard encoding | `X-Signature` header |
| Public key encoding | Base64 standard (32-byte keys) | `VIBETEA_PUBLIC_KEYS` env var |
| Constant-time comparison | `subtle::ConstantTimeEq` | `validate_token()` function |

### Monitor Authentication Flow

The authentication flow for event submission (POST /events):

1. Monitor signs request body with Ed25519 private key
2. Sends `X-Source-ID` header with monitor identifier
3. Sends `X-Signature` header with base64-encoded signature
4. Server verifies signature against registered public key
5. Validates event source matches authenticated source ID

### Session Management

| Setting | Value |
|---------|-------|
| WebSocket authentication | Query parameter token validation |
| Token validation | Case-sensitive, constant-time comparison |
| Token format | Any string (configurable via environment) |
| Session duration | Determined by WebSocket connection lifetime |

## Authorization

### Authorization Model

| Model | Description | Implementation |
|-------|-------------|-----------------|
| Source-based | Events attributed to authenticated source ID | Event source field must match X-Source-ID |
| Token-based | WebSocket clients require valid subscriber token | Query parameter token validation |
| No RBAC | No role-based or attribute-based access control | All authenticated sources have equal permissions |

### Permission Enforcement Points

| Location | Pattern | Example |
|----------|---------|---------|
| Event ingestion | Source ID validation | `post_events()` - `routes.rs:348-365` |
| Event submission | Signature verification | `post_events()` - `routes.rs:293-307` |
| WebSocket connection | Token validation | `get_ws()` - `routes.rs:458-491` |
| Rate limiting | Per-source limits | `RateLimiter` - `rate_limit.rs` |

## Input Validation

### Validation Strategy

| Layer | Method | Library |
|-------|--------|---------|
| API request | JSON schema validation | `serde` with custom types |
| Headers | String length and content checks | Manual validation in routes |
| Signatures | Base64 format and cryptographic verification | `ed25519_dalek` |
| Event payload | Serde deserialization with typed fields | `Event` struct in `types.rs` |
| Todo files | JSON array parsing with entry validation | `serde` with lenient fallback parsing in `todo_tracker.rs` |
| Stats cache | JSON parsing with retry on failure | `serde` with graceful fallback in `stats_tracker.rs` |

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| JSON payloads | Serde deserialization (type-safe) | `routes.rs:329-342` |
| Request body | Size limit (1 MB max) | `routes.rs:72` and `DefaultBodyLimit` |
| Headers | Non-empty validation | `routes.rs:263-273` (X-Source-ID), `277-290` (X-Signature) |
| Base64 data | Decoding validation | `auth.rs:204-206`, `218-220` |
| Todo content | Never transmitted (metadata only) | `monitor/src/trackers/todo_tracker.rs:45-48` |
| Stats cache file | Parsed with retries; invalid entries silently skipped | `monitor/src/trackers/stats_tracker.rs:175-207` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Storage |
|-----------|-------------------|---------|
| Ed25519 private keys (Monitor) | Raw 32-byte seed in file (mode 0600) | `~/.vibetea/key.priv` on monitor |
| Ed25519 public keys | Base64-encoded, registered in config | `VIBETEA_PUBLIC_KEYS` env var |
| Subscriber token | Stored in environment variable | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Event payloads | Not encrypted at rest | In-memory broadcasting only |
| Signatures | Base64-encoded, verified against message | Not stored |
| Todo task content | Never extracted or transmitted | Files read locally, only counts emitted |
| Todo file paths | Validated with UUID pattern matching | `monitor/src/trackers/todo_tracker.rs:504-506` |
| Stats cache file | Read-only access; metrics aggregated | `monitor/src/trackers/stats_tracker.rs:33-86` |

### Cryptography

| Type | Algorithm | Key Management |
|------|-----------|----------------|
| Authentication (Monitor) | Ed25519 (RFC 8032 strict) | Public keys from `VIBETEA_PUBLIC_KEYS` |
| Authentication (WebSocket) | Constant-time string comparison | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Transport security | HTTPS/TLS (application-agnostic) | Configured at load balancer/reverse proxy |

## Privacy Controls

### Agent Tracking Privacy

| Aspect | Implementation | Location |
|--------|----------------|----------|
| Task tool extraction | Metadata only (no prompts) | `monitor/src/trackers/agent_tracker.rs` |
| Extracted fields | `subagent_type`, `description` | `TaskToolInput` struct |
| Privacy-first principle | `prompt` field intentionally omitted | Line 75, struct definition |
| Events transmitted | Contain only agent_type and description | `monitor/src/types.rs:64` (AgentSpawnEvent) |

### Skill Tracking Privacy (Phase 5)

| Aspect | Implementation | Location |
|--------|----------------|----------|
| History file monitoring | Watches `~/.claude/history.jsonl` for skill invocations | `monitor/src/trackers/skill_tracker.rs` |
| Data extraction | Skill name, project path, timestamp, session ID | `SkillInvocationEvent` struct |
| Command arguments excluded | Slash command args not transmitted | `skill_tracker.rs:56-68` (extract_skill_name function) |
| Privacy-first design | Only basename metadata captured, no full paths | `skill_tracker.rs:22-25` |
| Append-only processing | Tail-like behavior tracks file position | `skill_tracker.rs:81, 480` (offset tracking) |
| Byte offset tracking | AtomicU64 with SeqCst ordering prevents re-reading | `skill_tracker.rs:405-413` |

### Todo Tracking Privacy (Phase 6)

| Aspect | Implementation | Location |
|--------|----------------|----------|
| Todo file monitoring | Watches `~/.claude/todos/` for file changes | `monitor/src/trackers/todo_tracker.rs:587-609` |
| Data extraction | Task status counts only (completed, in_progress, pending) | `TodoProgressEvent` struct |
| Task content excluded | Task descriptions never extracted or transmitted | `todo_tracker.rs:45-48` (privacy documentation) |
| Filename validation | UUID pattern matching prevents arbitrary files | `todo_tracker.rs:504-506` |
| Lenient parsing | Handles partially written files gracefully | `todo_tracker.rs:422-436` (parse_todo_file_lenient) |
| Session tracking | Correlates with summary events for abandonment detection | `todo_tracker.rs:549-584` (abandonment detection) |
| Metadata only principle | Only status counts and abandonment flag transmitted | `todo_tracker.rs:508-547` (event creation) |

### Stats Tracking Privacy (Phase 8)

| Aspect | Implementation | Location |
|--------|----------------|----------|
| Stats cache monitoring | Watches `~/.claude/stats-cache.json` for changes | `monitor/src/trackers/stats_tracker.rs:274-295` |
| Data extraction | Aggregated metrics only (session counts, token usage by model) | `SessionMetricsEvent`, `TokenUsageEvent` structs |
| No file content exposed | Only parsed metrics transmitted, not raw file contents | `stats_tracker.rs:175-207` (parse and emit) |
| Metrics aggregation | Global stats (total sessions, total messages, tool usage) | `SessionMetricsEvent` - `types.rs:108-117` |
| Per-model token tracking | Token usage breakdown by model name | `TokenUsageEvent` - `types.rs:92-103` |
| Retry logic | Graceful handling when file is being written | `stats_tracker.rs:222-254` (with_retry function) |
| Metadata only principle | Only aggregated counts transmitted, never user code or content | `types.rs:92-117` (StatsEvent variants) |

### Data Handling Philosophy

- **Privacy-first design**: All trackers extract only non-sensitive metadata
- **Metadata extraction**: Only status counts, skill names, agent types, token aggregates - never content, prompts, or arguments
- **No content logging**: Task content, prompts, command arguments, and file contents are never extracted or transmitted
- **Type-safe privacy**: Privacy enforcement is built into struct definitions, not runtime validation
- **File validation**: Strict filename pattern matching prevents reading unintended files
- **Graceful degradation**: Lenient parsing continues on invalid entries rather than failing completely
- **Aggregation approach**: Stats are aggregated into summary metrics before transmission

## Rate Limiting

| Endpoint | Limit | Window | Per |
|----------|-------|--------|-----|
| POST /events | 100 requests/second | Rolling window | Source ID |
| GET /ws | No limit | N/A | No rate limiting on WebSocket connections |
| GET /health | No limit | N/A | No rate limiting on health checks |

### Rate Limiter Implementation

- **Algorithm**: Token bucket with per-source tracking
- **Rate**: 100 tokens/second (configurable)
- **Burst capacity**: 100 tokens (configurable)
- **Cleanup**: Stale entries removed after 60 seconds of inactivity
- **Memory**: In-memory HashMap with RwLock for thread safety
- **Configuration**: `RateLimiter::new()` in `rate_limit.rs`

## Secrets Management

### Environment Variables

| Category | Variable | Required | Format |
|----------|----------|----------|--------|
| Public keys | `VIBETEA_PUBLIC_KEYS` | Yes (if auth enabled) | `source1:pubkey1,source2:pubkey2` (comma-separated) |
| Token | `VIBETEA_SUBSCRIBER_TOKEN` | Yes (if auth enabled) | Any string value |
| Port | `PORT` | No | Numeric, default 8080 |
| Auth bypass | `VIBETEA_UNSAFE_NO_AUTH` | No | "true" to disable auth (dev only) |
| Logging | `RUST_LOG` | No | Log level filter (default: info) |
| Privacy allowlist | `VIBETEA_BASENAME_ALLOWLIST` | No | Comma-separated file extensions (e.g., `.rs,.ts,.md`) |

### Secrets Storage

| Environment | Method |
|-------------|--------|
| Development | Environment variables (set directly or via shell) |
| CI/CD | GitHub Actions secrets or equivalent |
| Production | Environment variable injection at deployment time |
| Monitor keys | Stored locally in `~/.vibetea/` with 0600 permissions |

## Security Headers

VibeTea server does not directly manage security headers. These must be configured at the reverse proxy/load balancer level:

| Header | Recommended Value | Purpose |
|--------|-------------------|---------|
| Content-Security-Policy | `default-src 'self'` | XSS protection |
| X-Frame-Options | `DENY` | Clickjacking protection |
| X-Content-Type-Options | `nosniff` | MIME sniffing protection |
| Strict-Transport-Security | `max-age=31536000` | HTTPS enforcement |

## CORS Configuration

VibeTea is a WebSocket/HTTP API server designed for backend-to-backend communication. CORS configuration should be set at the reverse proxy level based on:

| Setting | Recommendation |
|---------|-----------------|
| Allowed origins | Restrict to known monitor/client sources |
| Allowed methods | POST (events), GET (WebSocket, health) |
| Allowed headers | Content-Type, X-Source-ID, X-Signature |
| Credentials | Not applicable (token in query param or env var) |

## Audit Logging

| Event | Logged Data | Location |
|-------|-------------|----------|
| Signature verification failure | Source ID, error type | `routes.rs:294` (warn level) |
| Rate limit exceeded | Source ID, retry_after | `routes.rs:314-318` (info level) |
| Invalid event format | Source ID, parse error | `routes.rs:332` (debug level) |
| Source mismatch | Authenticated source, event source | `routes.rs:350-355` (warn level) |
| WebSocket connection | Filter configuration | `routes.rs:494-497` (info level) |
| WebSocket disconnection | N/A | `routes.rs:578` (info level) |
| Configuration errors | Error message | `main.rs:53` (error level) |
| Server startup | Port, auth mode, public key count | `main.rs:74-79` (info level) |
| File watcher initialization | History file path | `skill_tracker.rs:474-476` (info level) |
| Todo watcher initialization | Todos directory path | `todo_tracker.rs:712-716` (info level) |
| Stats watcher initialization | Stats cache file path | `stats_tracker.rs:306-310` (info level) |

All logging is structured JSON output via `tracing` crate.

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
