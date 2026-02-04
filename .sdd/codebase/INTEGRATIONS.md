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

### Auth Flow (Updated for Phase 3)

#### Client-Side Authentication (New in Phase 3)

**Components involved:**
- `client/src/hooks/useAuth.ts` - Manages auth state and provides sign-in/sign-out methods
- `client/src/pages/Login.tsx` - GitHub OAuth entry point
- `client/src/pages/Dashboard.tsx` - Protected dashboard with user info
- `client/src/services/supabase.ts` - Supabase client configuration

**Flow:**
1. User visits application, `useAuth` hook initializes and checks current session
2. If no session exists, app displays Login page
3. User clicks "Sign in with GitHub" button
4. `@supabase/supabase-js` initiates OAuth2 flow with GitHub via Supabase
5. GitHub redirects back with authorization token
6. Supabase exchanges token for JWT and stores session
7. `onAuthStateChange` listener detects session, updates hook state
8. App routes to Dashboard with authenticated user
9. User metadata (name, email, avatar_url) available from Supabase user object

#### Server-Side Session Management

- **JWT Validation**: Client can exchange Supabase JWT for short-lived session token
- **Session Token Generation**: 32 bytes random, base64-url encoded (43 chars), 5-minute TTL
- **Session Storage**: In-memory HashMap with 10,000 capacity limit
- **WebSocket Auth**: Session token required in WebSocket upgrade request header or query param
- **Grace Period**: 30-second extension window for ongoing connections at session expiry

#### Complete Auth Lifecycle

1. **Client Login**: Supabase GitHub OAuth → JWT in local session
2. **Session Exchange**: JWT → Server session token (5-minute TTL)
3. **WebSocket Connection**: Session token in upgrade header
4. **Server Validation**: Call Supabase `/auth/v1/user` endpoint with JWT
5. **Revocation Detection**: Token validation catches revoked tokens
6. **Sign Out**: Client calls `supabase.auth.signOut()`, state listener triggers redirect

## External APIs

### First-Party APIs (Our Services)

| Service | Purpose | Base URL Config | Client Location |
|---------|---------|-----------------|-----------------|
| Supabase Auth API | User authentication and JWT issuance | `VITE_SUPABASE_URL` | `client/src/services/supabase.ts`, `client/src/hooks/useAuth.ts` |
| Supabase Edge Function: public-keys | Fetch monitor public keys | `{VIBETEA_SUPABASE_URL}/functions/v1/public-keys` | `server/src/supabase.rs` |
| VibeTea Server WebSocket | Real-time event streaming | `VITE_WS_URL` (default: ws(s)://current-host/ws) | `client/src/hooks/useWebSocket.ts` |

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

### GitHub OAuth Request (Client to Supabase)

Initiated via `@supabase/supabase-js`:
```javascript
await supabase.auth.signInWithOAuth({
  provider: 'github',
});
```

**Supabase handles**:
- Redirect to GitHub authorization endpoint
- OAuth token exchange with GitHub
- JWT generation and session creation
- Session storage in browser localStorage and cookie

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
| GitHub OAuth | Provider unavailable | Supabase displays error, user can retry |
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

## Client Auth State Management (Phase 3)

### useAuth Hook Integration Points

The `useAuth` hook (`client/src/hooks/useAuth.ts`) provides:

- **User state**: Current authenticated user or null
- **Session state**: Supabase session object with JWT and metadata
- **Loading state**: Initial auth check completion status
- **Auth methods**: `signInWithGitHub()`, `signOut()`
- **Event listener**: Automatic state update on auth changes via `onAuthStateChange`

### Consumer Components

- **App.tsx**: Routes to Login or Dashboard based on user state
- **Login.tsx**: Displays OAuth button, calls `signInWithGitHub()`
- **Dashboard.tsx**: Shows user info (avatar, name, email) from user metadata
- **TokenForm.tsx**: Requires authentication to access token management

### Session State Persistence

- Supabase stores session in browser localStorage
- `useAuth` hook subscribes to session changes on mount
- Auth state listener automatically syncs when session changes (OAuth redirect, sign-out)
- User object includes metadata from GitHub profile: `user_metadata.avatar_url`, `user_metadata.name`

---

## What Does NOT Belong Here

- Internal code architecture → ARCHITECTURE.md
- Testing infrastructure → TESTING.md
- Security policies and vulnerabilities → SECURITY.md
- Dependency versions and frameworks → STACK.md

---

*This document maps external service dependencies for Phase 3 (Client Authentication). Updated to include useAuth hook, Login/Dashboard page components, and complete client-to-server auth flow including GitHub OAuth, session management, and Supabase integration.*
