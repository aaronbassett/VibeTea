# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04 (Phase 3: Client OAuth flow)

## Authentication

### Authentication Method

| Method | Implementation | Configuration |
|--------|----------------|---------------|
| Ed25519 Signatures | ed25519_dalek | `server/src/auth.rs` |
| Bearer Token (WebSocket) | Token validation | `server/src/auth.rs` |
| Environment Variables | Public key registration | `VIBETEA_PUBLIC_KEYS`, `VIBETEA_SUBSCRIBER_TOKEN` |
| Supabase JWT (Phase 2+3) | Remote validation via `/auth/v1/user` endpoint | `server/src/supabase.rs:244-318` |
| Session Token (Phase 2+3) | Cryptographically secure server-side sessions | `server/src/session.rs` |
| GitHub OAuth (Phase 3) | Supabase OAuth provider integration | `client/src/hooks/useAuth.ts` |

### Token Configuration

| Setting | Value | Location |
|---------|-------|----------|
| Monitor authentication | Ed25519 digital signatures | X-Source-ID, X-Signature headers |
| WebSocket authentication | Bearer token (static string) | ?token query parameter |
| Signing algorithm | Ed25519 (RFC 8032 strict) | Uses `verify_strict()` |
| Key storage (monitor) | Ed25519 raw seed (32 bytes) | `~/.vibetea/key.priv` (mode 0600) |
| Public key encoding | Base64 standard (RFC 4648) | Registered via `VIBETEA_PUBLIC_KEYS` |
| Private key format (env var) | Base64-encoded 32-byte seed | `VIBETEA_PRIVATE_KEY` environment variable |
| Session Token Type (Phase 2+3) | 32-byte random data, base64-url encoded | `server/src/session.rs:60-63` |
| Session Token Length (Phase 2+3) | 43 characters (base64-url encoded) | `server/src/session.rs:63` |
| Session TTL (Phase 2+3) | 5 minutes (300 seconds) | `server/src/session.rs:51` |
| JWT Validation (Phase 2+3) | Remote validation (simpler, handles revocation) | `server/src/supabase.rs:244-318` |
| WebSocket Grace Period (Phase 2+3) | 30 seconds extension on TTL | `server/src/session.rs:54` |
| Public Key Refresh (Phase 2+3) | Every 30 seconds with exponential backoff | `server/src/supabase.rs:410-452` |
| GitHub OAuth Provider (Phase 3) | Supabase OAuth configured for GitHub | `client/src/hooks/useAuth.ts:125` |
| OAuth Redirect | Supabase handles OAuth callback, returns session | `client/src/hooks/useAuth.ts:100-106` |

### Session Management (Phase 2+3)

| Setting | Value | Implementation |
|---------|-------|-----------------|
| Session Storage | In-memory HashMap with RwLock (thread-safe) | `server/src/session.rs:188-194` |
| Session Capacity | 10,000 concurrent sessions | `server/src/session.rs:57` |
| Session Duration | 5 minutes default | `server/src/session.rs:51` |
| TTL Extension | One-time only per session (for WebSocket) | `server/src/session.rs:166-176` |
| Lazy Cleanup | Expired sessions removed on access | `server/src/session.rs:355-360` |
| Batch Cleanup | `cleanup_expired()` for background tasks | `server/src/session.rs:514-531` |
| Client Session State | Supabase session stored via SDK | `client/src/hooks/useAuth.ts:70-72` |
| Session Persistence (Client) | Browser localStorage (via Supabase SDK) | `@supabase/supabase-js` |

### Client-Side OAuth Flow (Phase 3)

| Component | Implementation | Location |
|-----------|-----------------|----------|
| OAuth Provider | GitHub via Supabase | `client/src/hooks/useAuth.ts` |
| Sign-in Method | `supabase.auth.signInWithOAuth()` | `client/src/hooks/useAuth.ts:125` |
| Auth State Listener | `supabase.auth.onAuthStateChange()` | `client/src/hooks/useAuth.ts:100-106` |
| Session Access | `supabase.auth.getSession()` | `client/src/hooks/useAuth.ts:80` |
| Sign-out | `supabase.auth.signOut()` | `client/src/hooks/useAuth.ts:146` |
| User Data Available | GitHub user ID, email (optional) | `client/src/hooks/useAuth.ts:22-26` |
| Login Page | GitHub button with OAuth redirect | `client/src/pages/Login.tsx:154-177` |
| Auth Hook Return Type | User, Session, loading state | `client/src/hooks/useAuth.ts:22-33` |

### Key Loading Strategy

| Source | Priority | Behavior | Location |
|--------|----------|----------|----------|
| Environment variable | First (takes precedence) | `VIBETEA_PRIVATE_KEY` must be valid base64-encoded 32 bytes | `monitor/src/crypto.rs:149-176` |
| File fallback | Second | `{VIBETEA_KEY_PATH}/key.priv` (defaults to `~/.vibetea`) | `monitor/src/crypto.rs:272-292` |
| Auto-generation | N/A | Can generate new keypair if neither exists | `monitor/src/crypto.rs:114-122` |

### Key Material Security

| Aspect | Implementation | Location |
|--------|----------------|----------|
| Private key seed (32 bytes) | Zeroed after SigningKey creation (zeroize crate) | `monitor/src/crypto.rs:120,173,235,289` |
| Decoded key buffer | Zeroed on error and success paths | `monitor/src/crypto.rs:163,225,281` |
| Environment variable trimming | Whitespace/newlines removed before decoding | `monitor/src/crypto.rs:153` |
| Key length validation | Exactly 32 bytes required, errors on mismatch | `monitor/src/crypto.rs:161,222,279` |
| File permissions | Unix mode 0600 (owner read/write only) | `monitor/src/crypto.rs:334-338` |

### Signature Verification

| Component | Detail |
|-----------|--------|
| Public key bytes | 32 bytes Ed25519 |
| Signature bytes | 64 bytes Ed25519 |
| Constant-time comparison | Used with `subtle::ConstantTimeEq` |
| Message content | Full HTTP request body (prevents tampering) |

## Authorization

### Authorization Model

| Model | Description |
|-------|-------------|
| Source-based | Event sources identified by source_id and registered public keys |
| Token-based | WebSocket clients authenticate with single shared token |
| Session-based (Phase 2+3) | Client provides session token in requests |
| Session-based Auth (Phase 2+3) | Server validates session token before processing |
| Resource ownership (Phase 2+3) | Users can only access their own events/monitors (foundation) |
| OAuth-based (Phase 3) | GitHub users authenticated via Supabase; any GitHub user can access |
| No granular roles | All sources have same permissions; all WebSocket clients have same permissions |

### Permissions

| Actor | Permissions | Scope |
|-------|------------|-------|
| Registered monitor (source_id) | Submit events to POST /events | Events matching authenticated source_id |
| WebSocket client (valid token) | Subscribe to event stream | All events (with optional filtering by source/type/project) |
| Authenticated GitHub user (Phase 3) | Access dashboard, view events | All events once authenticated |
| Unknown monitor | Rejected at authentication stage | N/A |
| Invalid token | Rejected at authentication stage | N/A |
| Unauthenticated web client (Phase 3) | Redirected to login | N/A |

### Permission Checks

| Location | Pattern | Example |
|----------|---------|---------|
| API events endpoint | Signature verification | `server/src/routes.rs:293` |
| WebSocket endpoint | Token validation | `server/src/routes.rs:483` |
| Event validation | Source ID matching | `server/src/routes.rs:348` - Events must match authenticated source |
| Session validation (Phase 2+3) | Session token in store | `server/src/session.rs:324-330` |
| JWT validation (Phase 2+3) | Remote check via Supabase | `server/src/supabase.rs:271-318` |
| Client auth check (Phase 3) | Supabase session validation | `client/src/hooks/useAuth.ts:80` |
| Login redirect (Phase 3) | User is null redirects to login | Application-level routing |

## Input Validation

### Validation Strategy

| Layer | Method | Implementation |
|-------|--------|-----------------|
| Event payload (API) | JSON deserialization | serde with custom Event types |
| Private key (env var) | Base64 + length validation | Standard RFC 4648 base64, exactly 32 bytes |
| Private key (file) | File size validation | Exactly 32 bytes required |
| Session name | Alphanumeric + hyphens/underscores | `monitor/src/tui/widgets/setup_form.rs:90-114` |
| Headers | String length and format checks | Empty string rejection |
| Request body | Size limit | 1 MB maximum (DefaultBodyLimit) |
| Event fields | Type validation | Timestamp, UUID, enum types enforced by serde |
| Session tokens (Phase 2+3) | Format check + TTL validation | `server/src/session.rs:324-330` |
| JWT tokens (Phase 2+3) | Remote validation | `server/src/supabase.rs:271-318` |
| Supabase environment (Phase 3) | URL and anon key required | `client/src/services/supabase.ts:15-20` |

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| Event JSON | Deserialization validates structure | `server/src/routes.rs:329` |
| Headers | Whitespace trimming + empty check | `server/src/auth.rs:270-276` |
| Source ID | Must not be empty | `server/src/config.rs:182-187` |
| Public key | Must not be empty, must be valid base64 | `server/src/config.rs:189-193` |
| Private key (env var) | Whitespace trimmed before base64 decode | `monitor/src/crypto.rs:153` |
| Session name | 64 char max, alphanumeric + `-_` only | `monitor/src/tui/widgets/setup_form.rs:99-107` |
| Supabase config (Phase 3) | Throws if env vars missing | `client/src/services/supabase.ts:15-20` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Storage | Location |
|-----------|-------------------|---------|----------|
| Private keys (monitor file) | Raw 32-byte seed | `~/.vibetea/key.priv` (mode 0600, owner-only) | `monitor/src/crypto.rs` |
| Private keys (monitor env var) | Base64-encoded 32-byte seed | `VIBETEA_PRIVATE_KEY` environment variable | `monitor/src/crypto.rs` |
| Public keys (server) | Base64-encoded format | Environment variable `VIBETEA_PUBLIC_KEYS` | `server/src/config.rs` |
| Subscriber token | Plain string comparison (constant-time) | Environment variable `VIBETEA_SUBSCRIBER_TOKEN` | `server/src/auth.rs` |
| Event payload | No encryption at rest | In-memory broadcast channel | `server/src/routes.rs` |
| Supabase JWT (Phase 2+3) | Remote validation only (never stored) | Temporary | `server/src/supabase.rs:271-318` |
| Session tokens (Phase 2+3) | 32 bytes cryptographically random | In-memory | `server/src/session.rs` |
| User ID (Phase 2+3) | Stored in session metadata | In-memory | `server/src/session.rs:128` |
| User email (Phase 2+3) | Stored in session metadata (optional) | In-memory | `server/src/session.rs:131` |
| Anon key (Phase 2+3) | Loaded from environment | Server memory | `server/src/supabase.rs:200` |
| GitHub user session (Phase 3) | Supabase stores JWT in browser localStorage | Browser | `client/src/hooks/useAuth.ts:70-72` |
| GitHub user ID (Phase 3) | Available in session.user.id | Session context | `client/src/hooks/useAuth.ts:24` |
| GitHub user email (Phase 3) | Available in session.user.email | Session context (optional) | `client/src/hooks/useAuth.ts:24` |

### Encryption

| Type | Algorithm | Implementation |
|------|-----------|-----------------|
| In transit | TLS 1.3+ | Requires HTTPS/WSS in production |
| At rest | None | Events are in-memory only, not persisted |
| Signing | Ed25519 deterministic | Uses standard RFC 8032 implementation |
| Token generation (Phase 2+3) | Cryptographically secure random (rand crate) | OS entropy via `rand::rng()` |
| Signature verification (Phase 2+3) | Ed25519 (RFC 8032) | Via ed25519-dalek with constant-time comparison |
| OAuth transport (Phase 3) | HTTPS/TLS | Supabase handles OAuth provider communication |

### Private Key Security

| Aspect | Implementation |
|--------|-----------------|
| Generation | Uses OS cryptographically secure RNG (`rand::rng()`) |
| File permissions | Unix mode 0600 (owner read/write only) |
| Format | Raw 32-byte seed, not PKCS#8 or other wrapper |
| File validation | Fails if file size != 32 bytes |
| Environment variable | Alternative loading via `VIBETEA_PRIVATE_KEY` (base64-encoded) |
| Memory zeroing | All intermediate buffers zeroed after use (zeroize crate) |
| Seed array | Explicitly zeroed after SigningKey construction |
| Error paths | Key material zeroed on decode failures |

### Logging and Secrets

| Practice | Implementation | Location |
|----------|-----------------|----------|
| Private key logging | Never logged (no string conversion of seed) | Throughout `monitor/src/crypto.rs` |
| Key fingerprint | Only 8-char prefix logged for identification | `monitor/src/crypto.rs:551-553` |
| Source identification | KeySource enum distinguishes env var vs file | `monitor/src/crypto.rs:44-49` |
| Session token logging (Phase 2+3) | No tokens logged at any level | `server/src/session.rs` (verified in tests) |
| JWT logging (Phase 2+3) | No JWTs logged, even on failure | `server/src/supabase.rs` (verified in tests) |
| Privacy compliance (Phase 2+3) | Comprehensive test suite verifies no leakage | `server/tests/auth_privacy_test.rs` |
| Supabase client logging (Phase 3) | Errors logged without token values | `server/src/supabase.rs` |
| OAuth token logging (Phase 3) | Sessions never logged with token values | `client/src/hooks/useAuth.ts` |

## Key Management

### Key Generation

- **Method**: `Crypto::generate()` using OS RNG via `rand::rng()`
- **Location**: `monitor/src/crypto.rs:114-122`
- **Seed**: 32 random bytes, zeroized after key construction

### Key Backup Functionality (Phase 9)

| Feature | Implementation | Location |
|---------|----------------|----------|
| Backup existing keys | `backup_existing_keys()` method | `monitor/src/crypto.rs:404-440` |
| Backup format | Timestamp suffix: `key.priv.backup.YYYYMMDD_HHMMSS` | Line 414 |
| Idempotent backup | Returns `Ok(None)` if no keys exist | Line 410 |
| Atomic operations | Private key backed up first; restores on public key failure | Lines 417-436 |
| Generate with backup | `generate_with_backup()` high-level API | Lines 480-489 |
| Tests | Comprehensive backup test suite (14 tests) | Lines 954-1177 |
| Timestamp validation | Format YYYYMMDD_HHMMSS (15 chars), parseable | Line 980-981 |
| Permission preservation | Backup files retain original Unix permissions | Line 1152-1176 |

### Key Loading

| Method | Source | Priority | Location |
|--------|--------|----------|----------|
| `load_from_env()` | `VIBETEA_PRIVATE_KEY` env var | - | Lines 149-176 |
| `load()` | File at `{dir}/key.priv` | - | Lines 272-292 |
| `load_with_fallback()` | Env var first, file fallback | Env takes precedence | Lines 210-247 |

### Key Export

| Feature | Implementation | Location |
|---------|-----------------|----------|
| CLI subcommand | `vibetea-monitor export-key` | `monitor/src/main.rs` |
| Output target | stdout only | Stdout contains base64 key + newline |
| Output format | Base64-encoded seed + single newline | FR-003 compliant |
| Diagnostic messages | All go to stderr | FR-023 compliant |
| Exit code success | 0 | - |
| Exit code failure | 1 (configuration error) | - |
| Path argument | Optional --path flag for key directory | - |
| Error handling | Missing key returns error message to stderr | - |
| Test Coverage | `monitor/tests/key_export_test.rs` (integration tests) | Comprehensive roundtrip validation |

### Setup Form Key Option (Phase 9)

| Feature | Behavior | Location |
|---------|----------|----------|
| Key option display | Conditional based on `existing_keys_found` | `monitor/src/tui/widgets/setup_form.rs:309-353` |
| No keys found | Only "Generate new key" shown (no radio toggle) | Lines 345-352 |
| Keys found | Both options shown with radio button indicators | Lines 310-343 |
| User selection | Toggle between "Use existing" and "Generate new" | Enforces choice during setup |

## Cryptographic Patterns

### Random Token Generation (Phase 2+3)

```rust
// Location: server/src/session.rs:564-568
// 32 bytes of cryptographically secure random data
let mut bytes = [0u8; 32];
rand::rng().fill(&mut bytes);  // Uses OS entropy
let token = URL_SAFE_NO_PAD.encode(bytes);  // base64-url without padding
```

### Supabase Client Configuration (Phase 2+3)

| Setting | Value | Purpose |
|---------|-------|---------|
| Request Timeout | 5 seconds | Prevent hanging requests |
| Retry Attempts | 5 with exponential backoff | Reliable startup |
| Backoff Formula | min(2^attempt * 100ms + jitter, 10s) | Smart retry delays |
| Jitter | 0-100ms random addition | Prevent thundering herd |

### GitHub OAuth Flow (Phase 3)

| Step | Implementation | Location |
|------|-----------------|----------|
| 1. Initiate OAuth | Call `signInWithOAuth({ provider: 'github' })` | `client/src/hooks/useAuth.ts:125-127` |
| 2. Redirect to GitHub | Supabase SDK handles redirect | `@supabase/supabase-js` |
| 3. User authenticates | GitHub login/approval by user | GitHub OAuth provider |
| 4. Supabase callback | Supabase receives auth code | Supabase OAuth integration |
| 5. Create session | Supabase creates JWT session | `@supabase/supabase-js` |
| 6. Store session | Browser localStorage via SDK | `client/src/hooks/useAuth.ts:102-104` |
| 7. App recognizes auth | `onAuthStateChange` fires with session | `client/src/hooks/useAuth.ts:100-106` |
| 8. Dashboard access | User can access protected routes | `client/src/pages/Login.tsx` |

## Signature Algorithm

- **Algorithm**: Ed25519 (RFC 8032 compliant)
- **Library**: `ed25519_dalek` with `verify_strict()`
- **Benefit**: Deterministic, pre-hashing protection, strong curve

## Constant-Time Operations

- **Token comparison**: `subtle::ConstantTimeEq` for bearer tokens
- **Location**: `server/src/auth.rs:290`
- **Purpose**: Prevent timing attacks on token validation

## Error Handling

- **No detailed errors on invalid signature**: Returns generic `InvalidSignature`
- **Unknown source**: Returns specific `UnknownSource` (source discovery risk accepted)
- **Implementation**: `server/src/auth.rs:49-145`

## Privacy Controls

### Sensitive Tool Filtering

| Tool | Context Handling | Purpose |
|------|------------------|---------|
| Bash | Always stripped | Prevent shell command exposure |
| Grep | Always stripped | Prevent search pattern exposure |
| Glob | Always stripped | Prevent path pattern exposure |
| WebSearch | Always stripped | Prevent query exposure |
| WebFetch | Always stripped | Prevent URL exposure |

### Path Sanitization

- Full paths reduced to basenames before transmission
- Example: `/home/user/project/src/auth.ts` → `auth.ts`
- Implementation: `monitor/src/privacy.rs:433-442`

### Extension Allowlist

- Environment variable: `VIBETEA_BASENAME_ALLOWLIST` (comma-separated)
- Example: `.rs,.ts,.md` - only these files transmitted
- Files with disallowed extensions: context stripped
- Default: Allow all extensions

### Privacy Pipeline

- **Location**: `monitor/src/privacy.rs`
- **Processing**: All event payloads filtered before transmission
- **Session events**: Passed through (project sanitized at parse time)
- **Summary events**: Text stripped to "Session ended"
- **Error events**: Passed through (category pre-sanitized)

### Privacy Compliance (Phase 2+3)

- **No token logging**: Session tokens and JWTs excluded from all log levels
- **Log capture testing**: `server/tests/auth_privacy_test.rs` verifies sensitive data not logged
- **TRACE level safe**: Even at TRACE level, tokens do not appear in logs
- **Constitution I compliance**: Privacy by design in authentication flows

## Secrets Management

### Environment Variables

| Category | Naming | Example | Purpose |
|----------|--------|---------|---------|
| Private Key | `VIBETEA_PRIVATE_KEY` | Base64-encoded 32-byte seed | Monitor signing key |
| Public Keys | `VIBETEA_PUBLIC_KEYS` | Space/comma-separated base64 keys | Server key registry |
| WebSocket Token | `VIBETEA_SUBSCRIBER_TOKEN` | Bearer token string | Client authentication |
| Supabase URL (Phase 2+3) | `SUPABASE_URL` | HTTPS URL | Supabase project URL |
| Supabase Anon Key (Phase 2+3) | `SUPABASE_ANON_KEY` | JWT format | Supabase authentication |
| Supabase URL (Client, Phase 3) | `VITE_SUPABASE_URL` | HTTPS URL | Client-side Supabase URL |
| Supabase Anon Key (Client, Phase 3) | `VITE_SUPABASE_ANON_KEY` | JWT format | Client-side Supabase anon key |

### Secrets Storage

| Environment | Method | Security Notes |
|-------------|--------|-----------------|
| Development | `.env` files (gitignored) or export statements | File-based, cleartext |
| GitHub Actions | GitHub Secrets | AES-128 encrypted at rest, rotatable |
| Production | Environment variables via container orchestration | Depends on orchestration platform |
| Private keys (file) | File-based (~/.vibetea/key.priv with mode 0600) | Owner-only readable |
| Private keys (env var) | Environment variable (base64-encoded) | Visible in process listing; GitHub Secrets encrypted |
| Client secrets (Phase 3) | Environment variables (VITE_*) | Buildtime variables; not sensitive in Vite |

### No Hardcoded Secrets

- All sensitive values loaded from environment variables
- No default credentials in code
- Key files stored with restrictive permissions (0600 for private, 0644 for public)
- Supabase credentials loaded at runtime from env vars

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

## Audit Logging

| Event | Logged Data | Implementation |
|-------|-------------|-----------------|
| Signature Verification | Success/failure, source_id | `server/src/auth.rs` (error types) |
| Token Validation | Success/failure | `server/src/auth.rs:269-295` |
| Key Load | Source (file or env var) + fingerprint | `KeySource` enum + startup logging |
| Key Generation | Fingerprint (first 8 chars of pubkey) | `monitor/src/crypto.rs:551-553` |
| Rate limit exceeded | source_id, retry_after | info level |
| WebSocket connect | filter params | info level |
| JWT validation (Phase 2+3) | User ID only (no token logged) | `server/src/supabase.rs` |
| Session creation (Phase 2+3) | User ID, email (no token logged) | `server/src/session.rs` |
| GitHub OAuth (Phase 3) | OAuth success/failure events | `client/src/hooks/useAuth.ts` |
| Auth state change (Phase 3) | User authenticated/signed out | `client/src/hooks/useAuth.ts:102-105` |

## Security Headers

| Header | Status | Note |
|--------|--------|------|
| X-Source-ID | Required | Must be non-empty string |
| X-Signature | Required (when auth enabled) | Base64-encoded Ed25519 signature |
| Content-Type | Assumed application/json | Not validated, accepted from client |
| Retry-After | Conditional | Only set on 429 responses |
| Content-Security-Policy | Not yet configured | XSS protection |
| X-Frame-Options | Not yet configured | Clickjacking protection |

## CORS Configuration

| Setting | Value |
|---------|-------|
| Allowed origins | All (no CORS policy enforced in codebase) |
| Allowed methods | POST (events), GET (ws, health) |
| Credentials | WebSocket token via query param |
| Public endpoint CORS (Phase 2+3) | `*` (Supabase public-keys endpoint) |

## Security Testing

### Test Coverage

| Test Type | Location | Coverage |
|-----------|----------|----------|
| Privacy compliance (Phase 2+3) | `server/tests/auth_privacy_test.rs` | 11 test cases |
| Session store (Phase 2+3) | `server/src/session.rs:571-936` | 20 test cases |
| Supabase client (Phase 2+3) | `server/src/supabase.rs:481-948` | 25 test cases |
| Error handling | `server/src/error.rs:522-911` | 40+ test cases |

### Privacy Test Coverage (Phase 2+3)

- Session token not logged on creation
- Session token not logged on validation
- Session token not logged on expiry/cleanup
- Session token not logged on TTL extension
- Session token not logged on removal
- JWT not logged on validation attempt
- JWT not logged on validation failure
- JWT not logged on server error
- Combined auth flow does not leak secrets
- Debug output does not leak tokens
- Capacity warnings do not leak tokens

### Client-Side Auth Testing (Phase 3)

- `useAuth` hook initializes session from Supabase
- `signInWithGitHub` initiates OAuth flow
- `signOut` clears session and local state
- Auth state changes trigger UI updates
- Error handling for auth failures
- Loading states during auth operations

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

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines active security controls. Review when adding authentication methods or cryptographic operations. Last updated with Phase 3 client OAuth integration.*
