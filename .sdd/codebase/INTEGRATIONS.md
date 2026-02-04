# External Integrations

> **Purpose**: Document all external services, APIs, databases, and third-party integrations.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Databases & Data Stores

| Service | Type | Purpose | Configuration Location |
|---------|------|---------|------------------------|
| PostgreSQL (Supabase) | Relational DB | Store monitor public keys and user data | Supabase project dashboard |

### Connection Patterns

- **Database access**: Via Supabase edge functions and PostgreSQL client
- **ORM/Query builder**: Supabase JavaScript SDK for edge functions; raw SQL via PostgreSQL
- **Migration approach**: SQL migrations in `supabase/migrations/`
- **Public key table**: `monitor_public_keys` with source_id PK, base64 public_key, and auto-updated timestamps

## Authentication & Authorization

| Provider | Purpose | Configuration Location |
|----------|---------|------------------------|
| Supabase Auth | User authentication via GitHub OAuth | Client: `VITE_SUPABASE_URL`, `VITE_SUPABASE_ANON_KEY` |
| Supabase JWT Validation | Server-side JWT validation via `/auth/v1/user` endpoint | Server: `VIBETEA_SUPABASE_URL`, `VIBETEA_SUPABASE_ANON_KEY` |

### Auth Flow

- **Client authentication**: OAuth2 via GitHub through Supabase Auth
  - Client uses `@supabase/supabase-js` to initiate GitHub OAuth flow
  - Supabase returns JWT token valid for session duration
  - JWT is stored in client-side state (Zustand)

- **Server-side JWT validation**:
  - Client exchanges Supabase JWT for short-lived session token (32 bytes, 5-minute TTL)
  - Server validates JWT by calling Supabase `/auth/v1/user` endpoint
  - Returns user ID and email in response
  - Enables token revocation detection and eliminates local key management

- **Session management**:
  - Session tokens stored in-memory with 10,000 capacity limit
  - TTL-based cleanup with lazy deletion on access
  - 30-second grace period for WebSocket connections
  - Supports single TTL extension per session for continuous connections

## External APIs

### First-Party APIs (Our Services)

| Service | Purpose | Base URL Config | Client Location |
|---------|---------|-----------------|-----------------|
| Supabase Auth API | User authentication and JWT issuance | `VITE_SUPABASE_URL` | `src/auth/` (client) |
| Supabase Edge Function: public-keys | Fetch monitor public keys | `{VIBETEA_SUPABASE_URL}/functions/v1/public-keys` | `server/src/supabase.rs` |
| VibeTea Server WebSocket | Real-time event streaming | `VITE_WS_URL` (default: ws(s)://current-host/ws) | Client routes |

### Third-Party APIs

| Provider | Purpose | SDK/Client | Configuration |
|----------|---------|------------|---------------|
| GitHub OAuth (via Supabase) | OAuth identity provider | @supabase/supabase-js | Configured in Supabase dashboard |

## Public Key Management

### Edge Function: `/functions/v1/public-keys`

- **Location**: `supabase/functions/public-keys/index.ts`
- **Authentication**: Public endpoint (no auth required per FR-015)
- **Purpose**: Exposes Ed25519 public keys from `monitor_public_keys` table
- **Response format**: `{ "keys": [{ "source_id": string, "public_key": string }, ...] }`
- **Caching**: 10-second cache-control header to reduce database load
- **Frequency**: Server fetches every 30 seconds with exponential backoff retry

### Public Key Refresh Mechanism

- **Polling interval**: 30 seconds
- **Retry strategy**: Exponential backoff with jitter (max 5 attempts)
  - Delay formula: `min(2^attempt * 100ms + random(0-100ms), 10s)`
- **Failure handling**: Falls back to cached keys if refresh fails
- **Server startup**: Requires successful initial fetch before accepting connections

## Environment Variables

### Server Configuration

| Variable | Required | Purpose | Example |
|----------|----------|---------|---------|
| `PORT` | No | HTTP server port (default: 8080) | `8080` |
| `VIBETEA_SUPABASE_URL` | Yes | Supabase project URL | `https://xxx.supabase.co` |
| `VIBETEA_SUPABASE_ANON_KEY` | Yes | Supabase anonymous key for API calls | `eyJ...` |
| `VIBETEA_PUBLIC_KEYS` | No | Monitor public keys (legacy format) | `monitor1:BASE64_KEY1,monitor2:BASE64_KEY2` |
| `VIBETEA_SUBSCRIBER_TOKEN` | No | Auth token for legacy subscriber clients | Secret string |
| `VIBETEA_UNSAFE_NO_AUTH` | No | Disable authentication (dev only) | `true` |

### Client Configuration

| Variable | Required | Purpose | Example |
|----------|----------|---------|---------|
| `VITE_SUPABASE_URL` | Yes | Supabase project URL | `https://xxx.supabase.co` |
| `VITE_SUPABASE_ANON_KEY` | Yes | Supabase anonymous key | `eyJ...` |
| `VITE_WS_URL` | No | WebSocket server URL (default: current host) | `wss://vibetea-server.fly.dev/ws` |

## HTTP Request/Response Patterns

### JWT Validation Request (Server to Supabase)

```
GET /auth/v1/user HTTP/1.1
Host: {SUPABASE_URL}
Authorization: Bearer {JWT_TOKEN}
apikey: {SUPABASE_ANON_KEY}
```

**Success Response (200)**:
```json
{
  "id": "user-uuid",
  "email": "user@example.com"
}
```

**Unauthorized Response (401)**:
- Maps to `ServerError::JwtInvalid`
- Returns HTTP 401 to client

### Public Keys Request (Server to Supabase)

```
GET /functions/v1/public-keys HTTP/1.1
Host: {SUPABASE_URL}
apikey: {SUPABASE_ANON_KEY}
```

**Success Response (200)**:
```json
{
  "keys": [
    {
      "source_id": "monitor-1",
      "public_key": "base64-encoded-32-byte-ed25519-key"
    }
  ]
}
```

## Failure Modes & Fallback Behavior

| Component | Failure | Fallback |
|-----------|---------|----------|
| JWT validation | Supabase unavailable | Return 503 Service Unavailable (FR-030) |
| Public key refresh | Network timeout | Retry with exponential backoff, use cached keys |
| Session store | Capacity exceeded | Return 503 Service Unavailable (FR-022) |
| WebSocket auth | Invalid/expired token | Disconnect client with 401 error |

## Database Schema

### Table: `monitor_public_keys`

```sql
CREATE TABLE monitor_public_keys (
  source_id TEXT PRIMARY KEY,           -- Monitor identifier
  public_key TEXT NOT NULL,             -- Base64-encoded Ed25519 public key
  description TEXT,                     -- Optional description
  created_at TIMESTAMPTZ,               -- Auto-set on creation
  updated_at TIMESTAMPTZ                -- Auto-updated on changes
);

-- Indexes
CREATE INDEX idx_monitor_public_keys_source_id ON monitor_public_keys(source_id);

-- Access control
GRANT SELECT ON monitor_public_keys TO anon;  -- For edge function access
```

---

## What Does NOT Belong Here

- Internal code architecture → ARCHITECTURE.md
- Testing infrastructure → TESTING.md
- Security policies and vulnerabilities → SECURITY.md
- Dependency versions and frameworks → STACK.md

---

*This document maps external service dependencies for Phase 2 (Supabase Authentication Foundational). Updated to reflect JWT validation via `/auth/v1/user` endpoint, session token management, and public key refresh mechanism.*
