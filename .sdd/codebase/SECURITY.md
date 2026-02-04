# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

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
| Public key encoding | Base64 standard (RFC 4648) | Registered via `VIBETEA_PUBLIC_KEYS` |
| Private key format (env var) | Base64-encoded 32-byte seed | `VIBETEA_PRIVATE_KEY` environment variable |

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
| Private key (env var) | Base64 + length validation | Standard RFC 4648 base64, exactly 32 bytes |
| Private key (file) | File size validation | Exactly 32 bytes required |
| Session name | Alphanumeric + hyphens/underscores | `monitor/src/tui/widgets/setup_form.rs:90-114` |
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
| Private key (env var) | Whitespace trimmed before base64 decode | `monitor/src/crypto.rs:153` |
| Session name | 64 char max, alphanumeric + `-_` only | `monitor/src/tui/widgets/setup_form.rs:99-107` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection Method | Storage |
|-----------|-------------------|---------|
| Private keys (monitor file) | Raw 32-byte seed | `~/.vibetea/key.priv` (mode 0600, owner-only) |
| Private keys (monitor env var) | Base64-encoded 32-byte seed | `VIBETEA_PRIVATE_KEY` environment variable |
| Public keys (server) | Base64-encoded format | Environment variable `VIBETEA_PUBLIC_KEYS` |
| Subscriber token | Plain string comparison (constant-time) | Environment variable `VIBETEA_SUBSCRIBER_TOKEN` |
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

## Key Management

### Key Generation

- **Method**: `Crypto::generate()` using OS RNG via `rand::rng()`
- **Location**: `monitor/src/crypto.rs:114-122`
- **Seed**: 32 random bytes, zeroized after key construction

### Key Backup Functionality (Phase 9 - NEW)

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

## Cryptographic Best Practices

### Signature Algorithm

- **Algorithm**: Ed25519 (RFC 8032 compliant)
- **Library**: `ed25519_dalek` with `verify_strict()`
- **Benefit**: Deterministic, pre-hashing protection, strong curve

### Constant-Time Operations

- **Token comparison**: `subtle::ConstantTimeEq` for bearer tokens
- **Location**: `server/src/auth.rs:290`
- **Purpose**: Prevent timing attacks on token validation

### Error Handling

- **No detailed errors on invalid signature**: Returns generic `InvalidSignature`
- **Unknown source**: Returns specific `UnknownSource` (source discovery risk accepted)
- **Implementation**: `server/src/auth.rs:49-145`

## Secrets Management

### Environment Variables

| Category | Naming | Example | Purpose |
|----------|--------|---------|---------|
| Private Key | `VIBETEA_PRIVATE_KEY` | Base64-encoded 32-byte seed | Monitor signing key |
| Public Keys | `VIBETEA_PUBLIC_KEYS` | Space/comma-separated base64 keys | Server key registry |
| WebSocket Token | `VIBETEA_SUBSCRIBER_TOKEN` | Bearer token string | Client authentication |

### Secrets Storage

| Environment | Method | Security Notes |
|-------------|--------|-----------------|
| Development | `.env` files (gitignored) or export statements | File-based, cleartext |
| GitHub Actions | GitHub Secrets | AES-128 encrypted at rest, rotatable |
| Production | Environment variables via container orchestration | Depends on orchestration platform |
| Private keys (file) | File-based (~/.vibetea/key.priv with mode 0600) | Owner-only readable |
| Private keys (env var) | Environment variable (base64-encoded) | Visible in process listing; GitHub Secrets encrypted |

### No Hardcoded Secrets

- All sensitive values loaded from environment variables
- No default credentials in code
- Key files stored with restrictive permissions (0600 for private, 0644 for public)

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

*This document defines active security controls. Review when adding authentication methods or cryptographic operations.*
