# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Authentication

### Authentication Method

| Method | Implementation | Configuration |
|--------|----------------|---------------|
| Ed25519 Signature (Rust) | ed25519_dalek with RFC 8032 strict verification | `server/src/auth.rs` |
| Ed25519 Signature (TypeScript) | @noble/ed25519 with RFC 8032 verification | `supabase/functions/_shared/auth.ts` |
| Bearer Token (Rust) | Constant-time comparison for WebSocket clients | `server/src/auth.rs` |
| Bearer Token (TypeScript) | Simple string comparison for edge functions | `supabase/functions/_shared/auth.ts` |

### Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Token type | Bearer token for WebSocket subscriptions and edge functions | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Signature algorithm | Ed25519 (RFC 8032 compliant) | `ed25519_dalek` (Rust), `@noble/ed25519` (TypeScript) |
| Signature encoding | Base64 standard encoding | `X-Signature` header |
| Public key encoding | Base64 standard (32-byte keys) | `VIBETEA_PUBLIC_KEYS` env var |
| Constant-time comparison (Rust) | `subtle::ConstantTimeEq` | `validate_token()` in `server/src/auth.rs` |
| Bearer token format | "Bearer <token>" | `Authorization` header (edge functions) |

### Monitor Authentication Flow

The authentication flow for event submission:

1. **Real-time (Server)** - POST /events:
   - Monitor signs request body with Ed25519 private key
   - Sends `X-Source-ID` header with monitor identifier
   - Sends `X-Signature` header with base64-encoded signature
   - Server verifies signature against registered public key via `verify_signature()` in `server/src/auth.rs`
   - Validates event source matches authenticated source ID

2. **Persistence (Edge Function)** - Supabase ingest endpoint:
   - Monitor signs JSON event batch with Ed25519 private key
   - Sends same `X-Source-ID` and `X-Signature` headers
   - Edge function verifies signature via `verifySignature()` in `supabase/functions/_shared/auth.ts`
   - Calls PostgreSQL function `bulk_insert_events()` with `SECURITY DEFINER` to insert events

### Edge Function Authentication Flow

1. **Query (Client)** - Supabase query endpoint:
   - Client sends `Authorization: Bearer <token>` header
   - Edge function validates token via `validateBearerToken()` in `supabase/functions/_shared/auth.ts`
   - Calls PostgreSQL function `get_hourly_aggregates()` with `SECURITY DEFINER` to fetch aggregates

### Session Management

| Setting | Value |
|---------|-------|
| WebSocket authentication | Query parameter token validation |
| Token validation | Case-sensitive, constant-time comparison (Rust); simple string comparison (TypeScript edge functions) |
| Token format | Any string (configurable via environment) |
| Session duration | Determined by WebSocket connection lifetime |

## Authorization

### Authorization Model

| Model | Description | Implementation |
|-------|-------------|-----------------|
| Source-based | Events attributed to authenticated source ID | Event source field must match X-Source-ID |
| Token-based | WebSocket clients require valid subscriber token | Query parameter token validation |
| Database-level | Row Level Security with service_role bypass | RLS on `public.events` table (deny all without policies) |
| Function-level | PostgreSQL functions use SECURITY DEFINER for service_role access | `bulk_insert_events()`, `get_hourly_aggregates()` |
| No RBAC | No role-based or attribute-based access control | All authenticated sources have equal permissions |

### Permission Enforcement Points

| Location | Pattern | Example |
|----------|---------|---------|
| Event ingestion (real-time) | Source ID validation | `post_events()` - `routes.rs:348-365` |
| Event submission (real-time) | Signature verification | `post_events()` - `routes.rs:293-307` |
| WebSocket connection | Token validation | `get_ws()` - `routes.rs:458-491` |
| Database access | Row Level Security | `ALTER TABLE public.events ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` |
| Batch insertion | Ed25519 signature verification | `verifyIngestAuth()` - `supabase/functions/_shared/auth.ts:128-156` |
| Historic query | Bearer token validation | `verifyQueryAuth()` - `supabase/functions/_shared/auth.ts:164-172` |
| Rate limiting | Per-source limits | `RateLimiter` - `rate_limit.rs` |

## Input Validation

### Validation Strategy

| Layer | Method | Library |
|-------|--------|---------|
| API request (Rust) | JSON schema validation | `serde` with custom types |
| Headers (Rust) | String length and content checks | Manual validation in routes |
| Signatures (Rust) | Base64 format and cryptographic verification | `ed25519_dalek` |
| Signatures (TypeScript) | Base64 format and cryptographic verification | `@noble/ed25519` |
| Event payload (Rust) | Serde deserialization with typed fields | `Event` struct in `types.rs` |
| Public keys | Length validation (32 bytes) | Manual validation in `getPublicKeyForSource()` |

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| JSON payloads (Rust) | Serde deserialization (type-safe) | `routes.rs:329-342` |
| Request body (Rust) | Size limit (1 MB max) | `routes.rs:72` and `DefaultBodyLimit` |
| Headers (Rust) | Non-empty validation | `routes.rs:263-273` (X-Source-ID), `277-290` (X-Signature) |
| Base64 data (Rust) | Decoding validation | `auth.rs:204-206`, `218-220` |
| Public keys (TypeScript) | Length validation (32 bytes) | `supabase/functions/_shared/auth.ts:38-45` |
| Signatures (TypeScript) | Length validation (64 bytes) | `supabase/functions/_shared/auth.ts:42-45` |
| Bearer tokens (TypeScript) | Prefix validation and simple comparison | `supabase/functions/_shared/auth.ts:99-110` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Storage |
|-----------|-------------------|---------|
| Ed25519 private keys (Monitor) | Raw 32-byte seed in file (mode 0600) | `~/.vibetea/key.priv` on monitor |
| Ed25519 public keys | Base64-encoded, registered in config | `VIBETEA_PUBLIC_KEYS` env var |
| Subscriber token | Stored in environment variable | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Event payloads (real-time) | Not encrypted at rest | In-memory broadcasting only |
| Event payloads (persisted) | Stored unencrypted in PostgreSQL | Accessible only via authenticated edge functions |
| Hourly aggregates | Returned as aggregated counts, never raw events | Query edge function returns count-only data |
| Signatures | Base64-encoded, verified against message | Not stored |

### Cryptography

| Type | Algorithm | Key Management |
|------|-----------|----------------|
| Authentication (Monitor → Server) | Ed25519 (RFC 8032 strict via ed25519_dalek) | Public keys from `VIBETEA_PUBLIC_KEYS` |
| Authentication (Monitor → Edge Function) | Ed25519 (RFC 8032 via @noble/ed25519) | Public keys from `VIBETEA_PUBLIC_KEYS` |
| Authentication (Client → Edge Function) | Constant-time string comparison (bearer token) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| Transport security | HTTPS/TLS (application-agnostic) | Configured at load balancer/reverse proxy |

## Database Security

### Row Level Security (RLS)

| Feature | Status | Configuration |
|---------|--------|---------------|
| RLS enabled | Yes | `ALTER TABLE public.events ENABLE ROW LEVEL SECURITY` |
| RLS enforced | Yes | `ALTER TABLE public.events FORCE ROW LEVEL SECURITY` |
| Policies defined | None (implicit deny-all) | Service role bypass only |
| Direct table access | Denied | All access via SECURITY DEFINER functions |

### SECURITY DEFINER Functions

| Function | Purpose | Access | Role |
|----------|---------|--------|------|
| `bulk_insert_events(JSONB)` | Insert batch events atomically | GRANT EXECUTE to service_role only | service_role |
| `get_hourly_aggregates(INTEGER, TEXT)` | Retrieve hourly aggregates for heatmap | GRANT EXECUTE to service_role only | service_role |

Both functions operate with `SECURITY DEFINER` to bypass RLS when called from edge functions, which authenticate via bearer token and Ed25519 signatures before invoking.

## Rate Limiting

| Endpoint | Limit | Window | Per |
|----------|-------|--------|-----|
| POST /events | 100 requests/second | Rolling window | Source ID |
| GET /ws | No limit | N/A | No rate limiting on WebSocket connections |
| GET /health | No limit | N/A | No rate limiting on health checks |
| Supabase ingest (edge function) | No limit (rate limited at edge function platform level) | N/A | Per request |
| Supabase query (edge function) | No limit (rate limited at edge function platform level) | N/A | Per request |

### Rate Limiter Implementation (Rust)

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
| Public keys (Monitor) | `VIBETEA_PUBLIC_KEYS` | Yes (if auth enabled) | `source1:pubkey1,source2:pubkey2` (comma-separated) |
| Token (all components) | `VIBETEA_SUBSCRIBER_TOKEN` | Yes (if auth enabled) | Any string value |
| Port (Server) | `PORT` | No | Numeric, default 8080 |
| Auth bypass (Server) | `VIBETEA_UNSAFE_NO_AUTH` | No | "true" to disable auth (dev only) |
| Logging (Server) | `RUST_LOG` | No | Log level filter (default: info) |
| Supabase URL (Monitor) | `VIBETEA_SUPABASE_URL` | No | Edge function base URL |
| Supabase batch interval (Monitor) | `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS` | No | Numeric, default 60 |
| Supabase retry limit (Monitor) | `VIBETEA_SUPABASE_RETRY_LIMIT` | No | Numeric, default 3 |
| Supabase URL (Client) | `VITE_SUPABASE_URL` | No | Edge function base URL |

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
| Allowed headers | Content-Type, X-Source-ID, X-Signature, Authorization |
| Credentials | Not applicable (token in query param or env var) |

## Audit Logging

| Event | Logged Data | Location |
|-------|-------------|----------|
| Signature verification failure (Rust) | Source ID, error type | `routes.rs:294` (warn level) |
| Signature verification failure (TypeScript) | Source ID, error message | `supabase/functions/_shared/auth.ts:49` (console.error) |
| Rate limit exceeded | Source ID, retry_after | `routes.rs:314-318` (info level) |
| Invalid event format | Source ID, parse error | `routes.rs:332` (debug level) |
| Source mismatch | Authenticated source, event source | `routes.rs:350-355` (warn level) |
| WebSocket connection | Filter configuration | `routes.rs:494-497` (info level) |
| WebSocket disconnection | N/A | `routes.rs:578` (info level) |
| Configuration errors | Error message | `main.rs:53` (error level) |
| Server startup | Port, auth mode, public key count | `main.rs:74-79` (info level) |
| Bearer token validation failure | Error type | `supabase/functions/_shared/auth.ts:95` (console.error) |
| Public key lookup failure | Source ID | `supabase/functions/_shared/auth.ts:78` (console.error) |

All logging is structured JSON output via `tracing` crate (Rust) or console methods (TypeScript).

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
