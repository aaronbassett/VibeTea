# Data Model: Supabase Authentication

**Date**: 2026-02-04
**Branch**: `002-supabase-auth`

## Entities

### 1. Supabase User Session (External)

Managed by Supabase SDK - not stored in VibeTea.

| Field | Type | Description |
|-------|------|-------------|
| `access_token` | string | JWT (HS256 signed by Supabase) |
| `refresh_token` | string | Long-lived token for refresh |
| `token_type` | string | Always "bearer" |
| `expires_in` | number | Seconds until access_token expires |
| `expires_at` | number | Unix timestamp of expiration |
| `user.id` | UUID | Supabase user ID |
| `user.email` | string | User's GitHub email |
| `user.app_metadata.provider` | string | "github" |

**Lifecycle**:
- Created: After successful GitHub OAuth
- Refreshed: Automatically by Supabase SDK
- Destroyed: On logout or session revocation

### 2. Server Session Token (In-Memory)

Stored in server's in-memory HashMap.

```rust
struct SessionEntry {
    token: String,       // 32-byte base64-url encoded (43 chars)
    user_id: String,     // Supabase user UUID
    created_at: Instant, // When session was created
    expires_at: Instant, // When session expires (5 min TTL)
}
```

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `token` | String | 32 bytes, base64-url, unique | Session token (key) |
| `user_id` | String | UUID format | Supabase user ID |
| `created_at` | Instant | Required | Creation timestamp |
| `expires_at` | Instant | Required | Expiration timestamp |

**Lifecycle**:
- Created: On successful JWT validation at `/auth/session`
- Extended: On successful WebSocket connection
- Expired: After 5 minutes without WebSocket activity
- Destroyed: On periodic cleanup sweep

**Validation Rules**:
- Token must be exactly 32 bytes (256 bits) of cryptographically random data
- Base64-URL encoding produces 43-character string
- Maximum capacity: 10,000 concurrent sessions (configurable)

### 3. Monitor Public Key (Supabase PostgreSQL)

Stored in Supabase database, served via edge function.

```sql
CREATE TABLE monitor_public_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `id` | UUID | PK, auto-generated | Row identifier |
| `source_id` | TEXT | UNIQUE, NOT NULL | Monitor identifier (e.g., hostname) |
| `public_key` | TEXT | NOT NULL | Base64-encoded Ed25519 public key |
| `active` | BOOLEAN | NOT NULL, default true | Whether key is currently valid |
| `created_at` | TIMESTAMPTZ | NOT NULL, default now() | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | NOT NULL, auto-updated | Last modification timestamp |

**Lifecycle**:
- Created: Admin adds new monitor via Supabase dashboard or API
- Deactivated: Admin sets `active = false` (key rejected on next refresh)
- Deleted: Admin removes row (key rejected on next refresh)

**Server Behavior**:
- Fetches active keys every 30 seconds
- Caches keys locally in memory
- Falls back to cached keys if refresh fails
- Logs warning on refresh failure, continues with stale keys

### 4. Cached Public Keys (In-Memory)

Server's local cache of monitor public keys.

```rust
struct PublicKeyCache {
    keys: RwLock<HashMap<String, String>>,  // source_id -> base64 public key
    last_refresh: RwLock<Instant>,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `keys` | HashMap<String, String> | source_id to public_key mapping |
| `last_refresh` | Instant | Last successful refresh time |

**Lifecycle**:
- Initialized: On server startup (required, retries up to 5 times)
- Refreshed: Every 30 seconds via background task
- Preserved: On refresh failure, keep existing keys

## Relationships

```
┌─────────────────────┐
│   GitHub User       │
└─────────┬───────────┘
          │ OAuth
          ▼
┌─────────────────────┐
│  Supabase Session   │──────┐
│  (SDK managed)      │      │
└─────────┬───────────┘      │
          │ JWT              │
          ▼                  │
┌─────────────────────┐      │
│  Token Exchange     │      │
│  POST /auth/session │      │
└─────────┬───────────┘      │
          │ Session Token    │
          ▼                  │
┌─────────────────────┐      │
│  Server Session     │      │ (validates JWT via)
│  (in-memory)        │◄─────┘
└─────────┬───────────┘
          │ Used for
          ▼
┌─────────────────────┐
│  WebSocket          │
│  Connection         │
└─────────────────────┘

┌─────────────────────┐
│  Monitor            │
│  (Ed25519 signing)  │
└─────────┬───────────┘
          │ signature
          ▼
┌─────────────────────┐
│  Server Verifies    │◄──── Public Key Cache
│  via cached keys    │            │
└─────────────────────┘            │
                                   │ refreshes from
                                   ▼
                         ┌─────────────────────┐
                         │  Supabase Edge Fn   │
                         │  /functions/v1/     │
                         │  public-keys        │
                         └─────────┬───────────┘
                                   │ queries
                                   ▼
                         ┌─────────────────────┐
                         │  monitor_public_keys│
                         │  (PostgreSQL)       │
                         └─────────────────────┘
```

## State Transitions

### Session Token States

```
                  ┌──────────┐
                  │  (none)  │
                  └────┬─────┘
                       │ POST /auth/session
                       │ (JWT validation success)
                       ▼
                  ┌──────────┐
             ┌───►│  Active  │◄───┐
             │    └────┬─────┘    │
             │         │          │
  WebSocket  │         │ 5min     │ WebSocket
  connection │         │ timeout  │ connection
  (extends)  │         ▼          │ (extends)
             │    ┌──────────┐    │
             └────┤ Expiring ├────┘
                  └────┬─────┘
                       │ cleanup sweep
                       │ (30 sec grace)
                       ▼
                  ┌──────────┐
                  │ Removed  │
                  └──────────┘
```

### Public Key Cache States

```
                  ┌──────────────┐
                  │   Starting   │
                  └──────┬───────┘
                         │ fetch (up to 5 retries)
                         ▼
              ┌──────────────────────┐
     success  │                      │  all retries fail
    ┌─────────┤   Fetching Keys      ├─────────┐
    │         │                      │         │
    ▼         └──────────────────────┘         ▼
┌──────────┐                           ┌───────────────┐
│  Ready   │                           │  Fatal Error  │
│ (cached) │                           │  (exit)       │
└────┬─────┘                           └───────────────┘
     │
     │ every 30 seconds
     ▼
┌──────────────────────┐
│   Refreshing Keys    │
└──────────┬───────────┘
     │              │
     │ success      │ failure
     ▼              ▼
┌──────────┐   ┌───────────────┐
│  Updated │   │ Keep Existing │
│  (new)   │   │ (log warning) │
└──────────┘   └───────────────┘
```

## Validation Rules

### JWT Validation (via Supabase API)

1. Call `GET {SUPABASE_URL}/auth/v1/user` with JWT in Authorization header
2. Success (200): JWT is valid, extract user_id from response
3. Unauthorized (401): JWT is invalid or expired
4. Server error (5xx): Supabase unavailable, return 503

### Session Token Validation

1. Lookup token in HashMap
2. If not found: Return 401
3. If found but `expires_at + 30s grace < now()`: Return 401
4. If valid and on WebSocket connect: Extend `expires_at` by TTL

### Monitor Signature Validation

1. Lookup `source_id` in public key cache
2. If not found: Return 401 (unknown source)
3. Decode base64 signature and public key
4. Verify signature using `ed25519_dalek::verify_strict()`
5. Use constant-time comparison

## Indexes

### Supabase Table Indexes

```sql
-- Primary key index (automatic)
-- monitor_public_keys.id

-- Unique index (automatic from constraint)
-- monitor_public_keys.source_id

-- Partial index for active keys lookup
CREATE INDEX idx_monitor_public_keys_active
ON monitor_public_keys(active)
WHERE active = true;
```

### In-Memory Indexes

- **Session tokens**: HashMap keyed by token string (O(1) lookup)
- **Public keys**: HashMap keyed by source_id string (O(1) lookup)
