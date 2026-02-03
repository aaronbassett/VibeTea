# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Authentication

### Authentication Methods

| Method | Implementation | Configuration | Scope |
|--------|----------------|---------------|-------|
| Ed25519 Signatures (Batch) | `ed25519_dalek` crate with RFC 8032 strict verification | `server/src/auth.rs` | Monitor → Server API (batch events) |
| Bearer Tokens (Simple) | Constant-time comparison (`subtle::ConstantTimeEq`) | `server/src/auth.rs` | WebSocket clients |
| Edge Function Signatures | `@noble/ed25519` (RFC 8032 compliant) | `supabase/functions/_shared/auth.ts` | Monitor → Ingest function (batch events) |
| Service Role Access | Supabase service role key | `SUPABASE_SERVICE_ROLE_KEY` | RPC functions, database access |

### Ed25519 Signature Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Algorithm | Ed25519 (RFC 8032) | `server/src/auth.rs` (lines 1-47) |
| Verification Method | `VerifyingKey::verify_strict()` for strict RFC 8032 compliance | `server/src/auth.rs` (line 231) |
| Key Format | Base64-encoded 32-byte public keys | `VIBETEA_PUBLIC_KEYS` env var |
| Signature Format | Base64-encoded 64-byte signatures | `X-Signature` header |
| Batch Signing | Entire batch payload signed as single message | `server/src/auth.rs` verify_signature() (line 195) |
| Key Storage | Configuration file (for monitors), environment vars (for server) | `~/.vibetea/key.pub`, `VIBETEA_PUBLIC_KEYS` |
| Private Key Permissions | 0600 (owner read/write only) | `monitor/src/crypto.rs` (line 179) |
| Public Key Permissions | 0644 (owner read/write, others read) | `monitor/src/crypto.rs` (line 194) |

### Request Headers for Authentication

| Header | Purpose | Format | Required |
|--------|---------|--------|----------|
| `X-Source-ID` | Monitor identifier for signature verification | String, alphanumeric | Yes (unless unsafe mode) |
| `X-Signature` | Base64-encoded Ed25519 signature of request body | Base64 (64 bytes decoded) | Yes (unless unsafe mode) |

### Bearer Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Token Type | Simple bearer token (no JWT) | `server/src/auth.rs` (lines 269-295) |
| Comparison Method | Constant-time (`ct_eq`) to prevent timing attacks | `server/src/auth.rs` (line 290) |
| Storage | Environment variable `VIBETEA_SUBSCRIBER_TOKEN` | Server config |
| Usage | WebSocket client authentication, edge function query endpoint | `supabase/functions/_shared/auth.ts` (lines 88-109) |
| Timeout | No server-side timeout (stateless bearer token) | N/A |

## Authorization

### Authorization Model

| Model | Implementation | Scope |
|-------|----------------|-------|
| Signature-Based | Only authenticated sources (via Ed25519 keys) can ingest batch events | Monitor API (`server/src/auth.rs`, `supabase/functions/ingest/index.ts`) |
| Token-Based | Bearer token grants read access to query endpoint | Client API (`supabase/functions/query/index.ts`) |
| RLS (Row Level Security) | No policies on events table = implicit deny-all (except service_role) | `supabase/migrations/20260203000000_create_events_table.sql` |
| Service Role Only | RPC functions `bulk_insert_events` and `get_hourly_aggregates` executable by service_role only | `supabase/migrations/20260203000001_create_functions.sql` |

### Access Control Matrix

| Access Pattern | Method | Who Can Do This | Evidence |
|---|---|---|---|
| Ingest batch events via POST /events (server) | Ed25519 signature verification | Authenticated monitors with registered public keys | `server/src/routes.rs` (lines 261-308), `server/src/auth.rs` |
| Ingest batch events via edge function | Ed25519 signature verification | Authenticated monitors with registered public keys | `supabase/functions/ingest/index.ts` (lines 274-294) |
| Query aggregates via edge function | Bearer token validation | Clients with valid `VIBETEA_SUBSCRIBER_TOKEN` | `supabase/functions/query/index.ts` (lines 186-195) |
| WebSocket subscription (server) | Bearer token validation (via query string) | Clients with valid token | `server/src/routes.rs` |
| Direct database access to events table | RLS policy enforcement | service_role key only (no policies for other roles) | `supabase/migrations/20260203000000_create_events_table.sql` (lines 39-42) |
| Call `bulk_insert_events` RPC | Explicit GRANT EXECUTE | service_role only | `supabase/migrations/20260203000001_create_functions.sql` (line 34) |
| Call `get_hourly_aggregates` RPC | Explicit GRANT EXECUTE | service_role only | `supabase/migrations/20260203000001_create_functions.sql` (line 71) |

## Input Validation

### Validation Strategy

| Layer | Method | Library | Location |
|-------|--------|---------|----------|
| Event ID format | Regex pattern matching | Built-in | `supabase/functions/ingest/index.ts` (line 49) |
| Event type enumeration | Whitelist matching | Built-in | `supabase/functions/ingest/index.ts` (lines 23-30, 182) |
| Timestamp format | RFC 3339 regex validation | Built-in | `supabase/functions/ingest/index.ts` (lines 55-56, 166) |
| Batch size limits | Runtime length check (max 1000 events) | Built-in | `supabase/functions/ingest/index.ts` (lines 297-307) |
| JSON schema validation | Manual field validation | Built-in | `supabase/functions/ingest/index.ts` (lines 115-209) |
| Source matching | Authenticated source vs. event source comparison | Built-in | `supabase/functions/ingest/index.ts` (lines 214-225) |
| Query parameters (days) | Enum validation (7 or 30 only) | Built-in | `supabase/functions/query/index.ts` (lines 99-122) |
| Base64 decoding | Try-catch with error handling | Built-in | `supabase/functions/ingest/index.ts` (lines 204-206, 218-220) |

### Sanitization

| Data Type | Sanitization Method | Location |
|-----------|-------------------|----------|
| Event payloads | Schema validation + JSONB type enforcement | Database constraints |
| SQL queries | Parameterized queries via Supabase SDK RPC | `supabase/functions/ingest/index.ts` (line 333) |
| Event source ID | Type validation (string), matched against authenticated source | `supabase/functions/ingest/index.ts` (line 320) |
| HTTP headers | Whitelist validation (X-Source-ID, X-Signature) | `supabase/functions/ingest/index.ts` (lines 132-141) |

## Data Protection

### Event Data Handling

| Aspect | Protection | Details |
|--------|-----------|---------|
| Privacy filtering | Pre-filtered by monitor before transmission | Event payload must contain only non-sensitive data |
| Storage | PostgreSQL JSONB type with RLS enforcement | `supabase/migrations/20260203000000_create_events_table.sql` |
| Access control | RLS implicit deny + RPC function execution grants | Only service_role can access via bulk_insert_events/get_hourly_aggregates |
| Encryption in transit | HTTPS/TLS (Supabase Edge Functions, browser WebSocket) | Standard web security |
| Encryption at rest | Supabase managed (no explicit config needed) | Supabase security model |

### Secrets Management

| Secret Type | Storage | Rotation | Usage |
|-------------|---------|----------|-------|
| Monitor Ed25519 private key | Local filesystem `~/.vibetea/key.priv` (0600) | Manual (regenerate key.priv) | Event signing |
| Monitor Ed25519 public key | Local filesystem `~/.vibetea/key.pub`, registered with server | On key rotation | Server-side verification registration |
| Server public keys | Environment variable `VIBETEA_PUBLIC_KEYS` (format: `source_id:base64_key,source_id2:base64_key2`) | Via deployment config update | Ed25519 verification |
| Subscriber token | Environment variable `VIBETEA_SUBSCRIBER_TOKEN` | Via deployment config update | Bearer token validation |
| Supabase service role key | Environment variable `SUPABASE_SERVICE_ROLE_KEY` | Supabase managed rotation | Database/RPC access |
| Supabase anon key | Environment variable `SUPABASE_ANON_KEY` (RLS enforced) | Supabase managed rotation | Client-side (unused for events table due to RLS) |

## Security Headers & CORS

### CORS Configuration (Ingest Endpoint)

| Setting | Value | Purpose |
|---------|-------|---------|
| Allowed Origins | `*` (open) | Public ingest endpoint for monitors |
| Allowed Methods | POST, OPTIONS | Only required methods |
| Allowed Headers | Content-Type, X-Source-ID, X-Signature | Custom auth headers + JSON |
| Preflight Cache | 86400 seconds (1 day) | Standard preflight caching |

### CORS Configuration (Query Endpoint)

| Setting | Value | Purpose |
|---------|-------|---------|
| Allowed Origins | Server-side routing (varies by deployment) | Browser security |
| Allowed Methods | GET | Read-only query endpoint |
| Allowed Headers | Authorization | Bearer token header |

## Rate Limiting

### Server Rate Limiting

| Endpoint | Limit | Window | Storage | Location |
|----------|-------|--------|---------|----------|
| POST /events | Per-source token bucket (100 tokens/sec, 100 burst capacity) | Continuous with refill | In-memory with cleanup | `server/src/rate_limit.rs` (lines 322-335) |
| GET /ws (WebSocket) | Per-source token bucket (100 tokens/sec, 100 burst capacity) | Continuous with refill | In-memory with cleanup | `server/src/rate_limit.rs` |
| Rate limit header | `Retry-After` with seconds until available | Per-request | HTTP response | `server/src/routes.rs` (line 321) |

### Rate Limiting Implementation

| Feature | Implementation | Location |
|---------|----------------|----------|
| Algorithm | Token bucket with per-source tracking | `server/src/rate_limit.rs` (lines 94-196) |
| Stale entry cleanup | Background task removes inactive sources after 60 seconds | `server/src/rate_limit.rs` (lines 360-385, 438-455) |
| Distributed setup | In-memory only; requires Redis for multi-instance deployments | `server/src/rate_limit.rs` |

### Edge Function Rate Limiting

| Endpoint | Limit | Implementation |
|----------|-------|----------------|
| POST /ingest | Not implemented at function level | Supabase Edge Functions provide built-in rate limiting per project |
| GET /query | Not implemented at function level | Supabase Edge Functions provide built-in rate limiting per project |

## Batch Event Submission Security

### Batch Processing

| Aspect | Details | Location |
|--------|---------|----------|
| Batch size limit | Maximum 1000 events per request | `supabase/functions/ingest/index.ts` (line 20) |
| Signature scope | Single signature covers entire batch (all events in request) | `server/src/auth.rs` (line 195), `supabase/functions/_shared/auth.ts` (line 152) |
| Duplicate handling | Events with duplicate IDs are skipped (ON CONFLICT DO NOTHING) | `supabase/functions/ingest/index.ts` (line 363) |
| Partial success | Response includes count of inserted vs. skipped events | `supabase/functions/ingest/index.ts` (lines 244-247) |
| Non-blocking persistence | Events accepted immediately (HTTP 202/200), persisted asynchronously | `server/src/routes.rs` (line 384), `supabase/functions/ingest/index.ts` |

## Audit Logging

### Event Ingestion Logging

| Event | Logged Data | Location |
|-------|-------------|----------|
| Successful signature verification | source_id (via log context) | `server/src/auth.rs` implicit in route handlers |
| Signature verification failure | error type (UnknownSource, InvalidSignature, etc.) | `server/src/auth.rs` error variants |
| Failed authentication | Error response returned to client | `supabase/functions/ingest/index.ts` (lines 264-276) |
| Database insert failures | RPC error logged | `supabase/functions/ingest/index.ts` (line 338) |
| Batch acceptance | source_id, event_count | `server/src/routes.rs` (lines 378-382) |

### Query Logging

| Event | Logged Data | Location |
|-------|-------------|----------|
| Bearer token validation failure | Error response returned | `supabase/functions/query/index.ts` (lines 186-195) |
| RPC query errors | Database error details | `supabase/functions/query/index.ts` (line 153) |

## Unsafe Mode (Development Only)

### VIBETEA_UNSAFE_NO_AUTH Configuration

| Setting | Purpose | Impact |
|---------|---------|--------|
| `VIBETEA_UNSAFE_NO_AUTH=true` | Disable all signature and token verification | Allows unauthenticated access to WebSocket and POST /events endpoint |
| Default | false (auth required) | Production-safe default |
| Logging | Warning logged on startup | `server/src/config.rs` (line 96) |

**WARNING**: This must never be enabled in production. Used for local development testing only.

## Cryptographic Algorithms

### Ed25519 Implementation Details

| Component | Algorithm | Standard | Library |
|-----------|-----------|----------|---------|
| Signature verification (server) | Ed25519 verify_strict | RFC 8032 (strict) | `ed25519_dalek` v2.1.0 |
| Signature verification (edge function) | Ed25519 verify | RFC 8032 | `@noble/ed25519` v2.0.0 |
| Key generation (monitor) | Ed25519 from cryptographic random | RFC 8032 | `ed25519_dalek` + `rand` crate |
| Base64 encoding | Standard base64 (RFC 4648) | RFC 4648 | `base64` crate |

### Timing Attack Prevention

| Operation | Protection | Location |
|-----------|-----------|----------|
| Bearer token comparison | `subtle::ConstantTimeEq` (constant-time byte comparison) | `server/src/auth.rs` (line 290) |
| Signature verification | Built-in constant-time comparison in `ed25519_dalek::verify_strict()` | `server/src/auth.rs` (line 231) |

## Client-Side Security

### WebSocket Authentication

| Aspect | Implementation | Details |
|--------|----------------|---------|
| Token transmission | Query parameter (`?token=xxx`) | `client/src/hooks/useWebSocket.ts` |
| Connection retry | Exponential backoff (1s initial, 60s max) ± 25% jitter | `client/src/hooks/useWebSocket.ts` |
| Reconnection limits | Implemented via backoff caps | Prevents resource exhaustion |
| Error handling | Non-blocking; failures do not crash app | `client/src/hooks/useWebSocket.ts` |

---

*This document defines security controls. Update when security posture changes.*
