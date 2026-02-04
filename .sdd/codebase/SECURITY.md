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
| Environment variable | First (takes precedence) | `VIBETEA_PRIVATE_KEY` must be valid base64-encoded 32 bytes | `monitor/src/crypto.rs:143` |
| File fallback | Second | `{VIBETEA_KEY_PATH}/key.priv` (defaults to `~/.vibetea`) | `monitor/src/crypto.rs:206` |
| Auto-generation | N/A | Can generate new keypair if neither exists | `monitor/src/crypto.rs:108` |

### Key Material Security

| Aspect | Implementation | Location |
|--------|----------------|----------|
| Private key seed (32 bytes) | Zeroed after SigningKey creation (zeroize crate) | `monitor/src/crypto.rs:114,169,233,287` |
| Decoded key buffer | Zeroed on error and success paths | `monitor/src/crypto.rs:157,221` |
| Environment variable trimming | Whitespace/newlines removed before decoding | `monitor/src/crypto.rs:147` |
| Key length validation | Exactly 32 bytes required, errors on mismatch | `monitor/src/crypto.rs:155,219,276` |
| File permissions | Unix mode 0600 (owner read/write only) | `monitor/src/crypto.rs:329-335` |

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
| Private key (env var) | Whitespace trimmed before base64 decode | `monitor/src/crypto.rs:147` |

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
| Key fingerprint | Only 8-char prefix logged for identification | `monitor/src/crypto.rs:429` |
| Source identification | KeySource enum distinguishes env var vs file | `monitor/src/crypto.rs:42` |

## Key Export Functionality (Phase 4-6)

### Export-Key Command

| Feature | Implementation | Location |
|---------|-----------------|----------|
| CLI subcommand | `vibetea-monitor export-key` | `monitor/src/main.rs:101-109` |
| Output target | stdout only | `monitor/src/main.rs:191-192` |
| Output format | Base64-encoded seed + single newline | `monitor/src/main.rs:192` |
| Diagnostic messages | All go to stderr | `monitor/src/main.rs:196-199` |
| Exit code success | 0 | `monitor/src/main.rs:193` |
| Exit code failure | 1 (configuration error) | `monitor/src/main.rs:199` |
| Path argument | Optional --path flag for key directory | `monitor/src/main.rs:107-108` |
| Error handling | Missing key returns error message to stderr | `monitor/src/main.rs:196-199` |

### Export Use Cases

| Use Case | Purpose | Integration | Phase |
|----------|---------|-------------|-------|
| GitHub Actions setup | Export key for CI/CD environment | Pipe output to GitHub secret creation | Phase 5 |
| Key migration | Move keys between systems | Base64 output compatible with `VIBETEA_PRIVATE_KEY` | Phase 4 |
| Backup verification | Verify exported key roundtrips | Integration tests verify signing consistency | Phase 4 |
| Composite GitHub Action | Pre-built reusable action wrapper | Used in GitHub Actions workflows | Phase 6 |

### GitHub Actions Integration (Phase 5-6)

#### Phase 5: Manual Workflow Setup

| Component | Implementation | Location |
|-----------|-----------------|----------|
| Private key secret | Stored as `VIBETEA_PRIVATE_KEY` | `.github/workflows/ci-with-monitor.yml:35` |
| Server URL secret | Stored as `VIBETEA_SERVER_URL` | `.github/workflows/ci-with-monitor.yml:36` |
| Source ID | Dynamic: `github-{repo}-{run_id}` | `.github/workflows/ci-with-monitor.yml:39` |
| Binary download | Pre-built binary from releases | `.github/workflows/ci-with-monitor.yml:49-50` |
| Process management | Background task with signal handling | `.github/workflows/ci-with-monitor.yml:63-70, 108-113` |

#### Phase 6: Composite GitHub Action

| Component | Implementation | Location |
|-----------|-----------------|----------|
| Action name | `VibeTea Monitor` | `.github/actions/vibetea-monitor/action.yml:16-17` |
| Action inputs | `server-url`, `private-key`, `source-id`, `version`, `shutdown-timeout` | `.github/actions/vibetea-monitor/action.yml:24-46` |
| Action outputs | `monitor-pid`, `monitor-started` | `.github/actions/vibetea-monitor/action.yml:48-55` |
| Binary download step | Handles version resolution and platform-specific URLs | `.github/actions/vibetea-monitor/action.yml:61-87` |
| Monitor startup step | Sets environment variables, validates config, starts process | `.github/actions/vibetea-monitor/action.yml:90-144` |
| Graceful shutdown | Documents SIGTERM signal handling and cleanup | `.github/actions/vibetea-monitor/action.yml:146-166` |
| Non-blocking errors | Network failures don't fail workflow; warnings logged | `.github/actions/vibetea-monitor/action.yml:101-120` |

### Composite Action Security Properties

| Property | Implementation | Location |
|----------|-----------------|----------|
| Environment variables passed safely | Uses GitHub Actions env context | `.github/actions/vibetea-monitor/action.yml:93-96` |
| Secret masking | Private key automatically masked by GitHub Actions | `VIBETEA_PRIVATE_KEY` parameter |
| Source ID interpolation | Dynamic generation with repo and run_id | `.github/actions/vibetea-monitor/action.yml:96` |
| Process lifecycle | Monitor runs in background; output via job logs | `.github/actions/vibetea-monitor/action.yml:127-144` |
| Signal handling documented | Post-job cleanup requires manual SIGTERM step | `.github/actions/vibetea-monitor/action.yml:146-166` |
| Failure tolerance | Missing binary or config logs warning, doesn't fail | `.github/actions/vibetea-monitor/action.yml:101-120` |

### Action Integration with README

| Section | Content | Location |
|---------|---------|----------|
| GitHub Actions Setup | Prerequisites and key export instructions | `README.md:134-166` |
| Manual Workflow Setup | Step-by-step instructions for manual workflows | `README.md:169-210` |
| Reusable Action Usage | Basic usage with the composite action | `README.md:212-252` |
| Custom Source ID | Example with custom source identifier | `README.md:254-263` |
| Pinned Version | Example with specific monitor version | `README.md:265-274` |
| Action Inputs | Complete input parameter reference | `README.md:276-284` |
| Action Outputs | Available output parameters | `README.md:286-291` |

### Security Properties of Export

| Property | Guarantee |
|----------|-----------|
| Key integrity | Exported key matches file content exactly |
| Output purity | stdout contains only key data, no diagnostic text |
| Roundtrip compatibility | Exported key can be loaded via `VIBETEA_PRIVATE_KEY` |
| Signature consistency | Ed25519 is deterministic; exported key produces identical signatures |
| Memory safety | Seed array explicitly zeroed after use (zeroize crate) |

## Project Activity Tracking (Phase 11)

### Privacy-First Design

| Aspect | Implementation | Assurance |
|--------|-----------------|-----------|
| No code transmission | Only metadata extracted from session files | Pattern matching on JSON, no file content read |
| No prompt transmission | Session content never read beyond summary detection | Only JSONL headers scanned for `"type": "summary"` |
| Path transmission only | Absolute project paths sent as metadata | Derived from directory slug parsing |
| Session ID transmission | UUID identifiers sent as activity markers | Filenames validated as UUID format |
| No secrets exposure | Claude API keys, file contents never accessed | File watching limited to `.jsonl` files in `~/.claude/projects/` |

### ProjectTracker Implementation

| Component | Security Detail | Location |
|-----------|-----------------|----------|
| Directory scope | Watches only `~/.claude/projects/` (user-local) | `monitor/src/trackers/project_tracker.rs:337-344` |
| File filtering | Processes only `.jsonl` files | `monitor/src/trackers/project_tracker.rs:582` |
| Filename validation | Session IDs must match UUID format (8-4-4-4-12 hex) | `monitor/src/trackers/project_tracker.rs:751-772` |
| Content access | Only reads file to detect summary events | `monitor/src/trackers/project_tracker.rs:775-778` |
| Summary detection | Parses JSONL lines as JSON, checks `type: "summary"` field only | `monitor/src/trackers/project_tracker.rs:157-173` |
| Path reconstruction | Reverses slug format (`-` to `/`) without validation | `monitor/src/trackers/project_tracker.rs:119-123` |
| Event emission | Sends project path, session ID, active flag only | `monitor/src/types.rs:189-196` |

### ProjectActivityEvent Type

| Field | Content | Privacy Impact | Location |
|-------|---------|-----------------|----------|
| `project_path` | Absolute filesystem path (e.g., `/home/user/Projects/VibeTea`) | Metadata only; no content leaked | `monitor/src/types.rs:191` |
| `session_id` | UUID from filename (e.g., `6e45a55c-...`) | Activity tracking identifier | `monitor/src/types.rs:193` |
| `is_active` | Boolean (true if no summary event detected) | Session status indicator | `monitor/src/types.rs:195` |

### Data Extracted from Session Files

| Data Type | Extracted | Transmitted | Reason |
|-----------|-----------|-------------|--------|
| Project path | Yes (from directory slug) | Yes | Activity dashboard |
| Session ID | Yes (from filename) | Yes | Session correlation |
| Session status | Yes (presence of summary event) | Yes | Activity indication |
| Prompt content | No | No | Privacy |
| Code snippets | No | No | Intellectual property protection |
| File contents | No | No | Data minimization |
| Session metadata (messages, timestamps) | No | No | Data minimization |

### File Watching Security

| Aspect | Implementation |
|--------|-----------------|
| Event source | `notify` crate file system watcher |
| Recursive watching | Monitors `~/.claude/projects/` and subdirectories |
| Event filtering | Only processes `Create` and `Modify` events |
| Debouncing | None (per research.md: 0ms debounce for project files) |
| Path validation | Events checked against projects directory boundary |
| Error handling | File not found treated as session completion |

### Slug Format Limitations

| Limitation | Impact | Mitigation |
|-----------|--------|------------|
| Dashes in directory names ambiguous | `/home/user/my-project` becomes `/home/user/my/project` | Document in privacy considerations; use alternative separators in project names |
| No validation of decoded path | May produce invalid paths | Used for display only; not interpreted as filesystem operations |
| No round-trip guarantee | Cannot reliably recover original path if it contains dashes | Use camelCase or underscores in project directory names |

### Thread Safety

| Component | Thread-Safe | Mechanism |
|-----------|-------------|-----------|
| Watcher creation | Yes | Single ownership in `ProjectTracker` struct |
| File event delivery | Yes | `mpsc` channel for thread-safe delivery |
| Session scanning | Yes | Async tasks with channel coordination |
| Event emission | Yes | Sender type enforces single owner per channel |

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

## Secrets Management

### Environment Variables

| Category | Naming | Example | Required |
|----------|--------|---------|----------|
| Monitor auth | `VIBETEA_SOURCE_ID` | monitor-1 | No (defaults to hostname) |
| Monitor keys | `VIBETEA_KEY_PATH` | ~/.vibetea | No (defaults) |
| Monitor private key | `VIBETEA_PRIVATE_KEY` | base64-encoded-32-bytes | No (fallback to file) |
| Server public keys | `VIBETEA_PUBLIC_KEYS` | source1:base64key,source2:base64key | Yes (unless unsafe_no_auth) |
| WebSocket token | `VIBETEA_SUBSCRIBER_TOKEN` | secret-token-string | Yes (unless unsafe_no_auth) |
| Server URL (monitor) | `VIBETEA_SERVER_URL` | https://vibetea.fly.dev | Yes (required) |

### Secrets Storage

| Environment | Method | Security Notes |
|-------------|--------|-----------------|
| Development | `.env` files (gitignored) or export statements | File-based, cleartext |
| GitHub Actions | GitHub Secrets | AES-128 encrypted at rest, rotatable |
| Production | Environment variables via container orchestration | Depends on orchestration platform |
| Private keys (file) | File-based (~/.vibetea/key.priv with mode 0600) | Owner-only readable |
| Private keys (env var) | Environment variable (base64-encoded) | Visible in process listing; GitHub Secrets encrypted |

### Key Provisioning

| Process | Location |
|---------|----------|
| Monitor key generation | `monitor/src/crypto.rs:108` - Crypto::generate() |
| Environment variable loading | `monitor/src/crypto.rs:143` - Crypto::load_from_env() |
| File with env fallback | `monitor/src/crypto.rs:206` - Crypto::load_with_fallback() |
| Key export for external use | `monitor/src/main.rs:181-202` - run_export_key() |
| Server registration | Manual environment variable setup |
| Public key fingerprint | `monitor/src/crypto.rs:429` - public_key_fingerprint() (8 chars) |

### GitHub Actions Key Management (Phase 5-6)

#### Manual Workflow Setup

| Step | Command | Security |
|------|---------|----------|
| 1. Export | `vibetea-monitor export-key` | Outputs base64 key to stdout only |
| 2. Store | Add to GitHub Secrets as `VIBETEA_PRIVATE_KEY` | GitHub encrypts at rest |
| 3. Use | Injected as env var during workflow | Masked in logs; available to monitor process |
| 4. Cleanup | Automatically cleared after workflow | GitHub Actions cleanup |

#### Composite Action Usage

| Step | Implementation | Security |
|------|-----------------|----------|
| 1. Input | `private-key` parameter with `secrets.VIBETEA_PRIVATE_KEY` | GitHub automatically masks in logs |
| 2. Load | Action passes private-key to monitor via env var | Masked by GitHub Actions |
| 3. Start | Monitor starts in background with env vars set | Process inherits masked variable |
| 4. Cleanup | Manual SIGTERM step or automatic job cleanup | Documents recommended post-job step |

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

## Audit Logging

| Event | Logged Data | Level |
|-------|-------------|-------|
| Key source identification | Environment variable vs file | info |
| Authentication failures | source_id, error type | warn/debug |
| Signature verification failures | source_id, error details | warn |
| Rate limit exceeded | source_id, retry_after | info |
| WebSocket connect | filter params | info |
| WebSocket disconnect | - | info |
| Configuration errors | Missing variables | error |
| Project activity tracking | project_path, session_id, is_active | debug/trace |

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

*This document defines active security controls. Review when adding authentication methods or cryptographic operations.*
