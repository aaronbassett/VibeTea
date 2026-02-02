# Security

> **Purpose**: Document authentication, authorization, security controls, and vulnerability status.
> **Generated**: 2026-02-02
> **Last Updated**: 2026-02-02

## Authentication

### Authentication Method

| Method | Implementation | Configuration | Status |
|--------|----------------|---------------|--------|
| Ed25519 Signatures | ed25519-dalek library with verify_strict() | `VIBETEA_PUBLIC_KEYS` env var | Implemented (Phase 3) |
| Bearer Token | Constant-time comparison in validate_token() | `VIBETEA_SUBSCRIBER_TOKEN` env var | Implemented (Phase 3) |
| Development bypass | `VIBETEA_UNSAFE_NO_AUTH=true` | Environment variable | Development only |
| Client Token Storage | localStorage with password input | `vibetea_token` key | Implemented (Phase 7) |

### Authentication Flow

**Monitor to Server (Event Ingestion)**:
- Monitor signs event payload with Ed25519 private key using `SigningKey::sign()`
- Signature sent as base64-encoded value in `X-Signature` header
- Source ID sent in `X-Source-ID` header for public key lookup
- Server verifies using `verify_signature()` function in `server/src/auth.rs:192-233`
- Verification uses `verify_strict()` for strict Ed25519 verification per RFC 8032
- Public keys stored as base64-encoded Ed25519 keys (32 bytes) in `VIBETEA_PUBLIC_KEYS` mapping
- Format: `source_id:base64_pubkey,...` parsed in `server/src/config.rs:157-203`

**Client to Server (WebSocket)**:
- Client stores bearer token in localStorage via `TokenForm` component (`client/src/components/TokenForm.tsx`)
- Token storage key: `vibetea_token` (defined in `useWebSocket.ts:18` and `TokenForm.tsx:15`)
- Token sent in WebSocket URL query parameter: `?token=xxx` via `buildWebSocketUrl()` (`useWebSocket.ts:75-79`)
- Server validates token presence and value using `validate_token()` in `server/src/auth.rs:269-295`
- Comparison uses constant-time bit comparison via `subtle::ConstantTimeEq` to prevent timing attacks
- No token expiration mechanism implemented (planned for Phase 3+)
- No per-client token scope differentiation

### Client Token Management (Phase 7)

From `client/src/components/TokenForm.tsx`:

- **Token input**: HTML password input field for user entry (`<input type="password">`)
- **Storage**: localStorage via `localStorage.setItem(TOKEN_STORAGE_KEY, token)`
- **Retrieval**: Retrieved by `useWebSocket()` at connection time: `localStorage.getItem(TOKEN_STORAGE_KEY)`
- **Status tracking**: Component tracks saved/not-saved state via visual indicator
- **Clear functionality**: `localStorage.removeItem()` to delete token
- **Cross-tab sync**: Listens to `storage` events to detect changes from other tabs/windows
- **Validation**: Token trimmed before save, empty tokens rejected

### Client WebSocket Connection (Phase 7)

From `client/src/hooks/useWebSocket.ts`:

- **Auto-reconnect**: Exponential backoff (1s initial, 60s max, ±25% jitter)
- **Calculation**: `calculateBackoff()` function doubles delay per attempt, capped at 60s
- **Jitter formula**: `1 + (Math.random() * 2 - 1) * 0.25` prevents thundering herd
- **Missing token handling**: Warns to console and prevents connection if token missing
- **Structural validation**: Basic event validation via `parseEventMessage()` (lines 97-122)
  - Checks for required fields: `id`, `source`, `timestamp`, `type`, `payload`
  - Invalid events silently dropped (logged only on parse error)
  - No schema validation against event types

### Development Mode Bypass

| Setting | Impact | Location |
|---------|--------|----------|
| `VIBETEA_UNSAFE_NO_AUTH=true` | Disables all auth (dev only) | `server/src/config.rs:57-58` |
| Behavior | Accepts any client, any source, any token | Validated in `Config::validate()` at `server/src/config.rs:108-126` |
| Logging | Warning logged on startup | `server/src/config.rs:94-98` |
| Route enforcement | Auth skipped in `server/src/routes.rs:272-304` | Conditional check before `verify_signature()` |

**Warning**: This setting logs a warning but is not otherwise restricted. Production deployments must never enable this. Configuration validation is enforced with comprehensive tests in `server/src/config.rs:205-415`.

### Signature Verification Details

From `server/src/auth.rs:192-233`:

1. **Source lookup**: Retrieves public key for source_id from `VIBETEA_PUBLIC_KEYS` map
2. **Key decoding**: Decodes base64 public key (must be exactly 32 bytes for Ed25519)
3. **Key parsing**: Constructs `VerifyingKey` from decoded bytes
4. **Signature decoding**: Decodes base64 signature (must be exactly 64 bytes)
5. **Verification**: Uses `verify_strict()` for RFC 8032 strict verification
6. **Error classification**: Returns specific `AuthError` variants for each failure mode

### Token Validation Details

From `server/src/auth.rs:269-295`:

1. **Trimming**: Leading/trailing whitespace removed from both tokens
2. **Empty check**: Both tokens must be non-empty after trimming
3. **Length check**: Token lengths must match (not constant-time, acceptable)
4. **Bit comparison**: `ct_eq()` constant-time comparison prevents timing attacks
5. **Error handling**: Returns `AuthError::InvalidToken` on any mismatch

## Authorization

### Authorization Model

| Model | Implementation | Scope |
|-------|----------------|-------|
| Token-based | Bearer token presence check via `validate_token()` | Client access to WebSocket |
| Source verification | Public key verification via Ed25519 signature | Monitor identity for events |
| No granular RBAC | Not implemented | - |

### Permission Structure

- **Server accepts from**: Any monitor with registered public key (source_id matching in `VIBETEA_PUBLIC_KEYS`)
- **Client receives from**: Any client with valid bearer token matching `VIBETEA_SUBSCRIBER_TOKEN`
- **Server publishes to**: All connected WebSocket clients equally
- **No resource-level permissions**: All clients see all events
- **No user-level isolation**: No per-user filtering of events
- **WebSocket filtering available**: Clients can filter by source/type/project via query parameters (advisory, not enforced)

### Client Authorization Gaps

- No token management beyond localStorage storage
- No per-user or per-resource permissions
- No role-based access control (RBAC)
- No scope limitation on token capabilities
- All authenticated clients access identical data streams
- No server-side enforcement of client-specified filters

## Input Validation

### Validation Strategy

| Layer | Method | Implementation |
|-------|--------|-----------------|
| Event parsing | Deserialization validation | `serde` with Rust type system |
| Configuration | Structured parsing | `Config::from_env()` with validation |
| API input | Type safety | Rust compiler enforces types |
| Signature/Token validation | Cryptographic checks | `verify_signature()` and `validate_token()` |
| JSONL parsing | JSON deserialization + error handling | `monitor/src/parser.rs:354-359` |
| Client WebSocket messages | Basic structural validation | `parseEventMessage()` in `useWebSocket.ts:97-122` |
| Client token input | Trimming only | `TokenForm.tsx:103` |

### Event Validation (Server Types)

Event structure from `server/src/types.rs:1-163`:

- **EventType**: Enum-based (`session`, `activity`, `tool`, `agent`, `summary`, `error`)
- **EventPayload**: Untagged union with variant ordering for correct deserialization
- **Timestamp**: RFC 3339 UTC (`DateTime<Utc>`)
- **Session ID**: UUID format validated by chrono
- **Event ID**: Prefixed format (`evt_` + 20 alphanumeric chars)

All event fields are type-checked at deserialization via serde. Invalid JSON fails before reaching application logic. Validation occurs at `server/src/routes.rs:325-338`.

### Client Event Validation (Phase 7)

From `client/src/hooks/useWebSocket.ts:97-122`:

- **Message type check**: Validates `typeof data === 'string'`
- **JSON parsing**: Wrapped in try-catch, invalid JSON returns null
- **Field validation**: Checks for required properties: `id`, `source`, `timestamp`, `type`, `payload`
- **Type coercion**: Casts to `VibeteaEvent` without runtime schema validation
- **Silent failure**: Invalid events logged via console but not re-thrown

From `client/src/types/events.ts`:

- **TypeScript types**: EventType union, SessionPayload, ToolPayload, etc. (read-only properties)
- **Type guards**: Type narrowing possible with discriminated union
- **No runtime validators**: serde validation only available on server

### Claude Code JSONL Parsing

From `monitor/src/parser.rs`:

- **Privacy-first design**: Only metadata extracted (tool names, timestamps, file basenames)
- **Code content excluded**: Prompts, response text, tool results never extracted
- **Structure validation**: Raw JSON deserialization validates format
- **Event type mapping**: Untagged enum ensures only recognized events processed
- **Path safety**: File basenames extracted without full paths via `extract_basename()`
- **URL decoding**: Project names decoded safely with validation
- **Error handling**: Malformed JSON lines skipped with warning logs (line 357)

### Configuration Validation (Server)

From `server/src/config.rs:79-202`:

- Port: Parsed as `u16` (1-65535) with error handling at `server/src/config.rs:142-151`
- Public keys: Format validation `source:base64key` at `server/src/config.rs:157-203`
- Empty field checks for source_id and pubkey at `server/src/config.rs:182-197`
- Whitespace trimming in key pairs
- Conditional validation: auth fields required unless `VIBETEA_UNSAFE_NO_AUTH=true` at `server/src/config.rs:108-126`
- Comprehensive test coverage: `server/src/config.rs:205-415`

### Monitor Configuration Validation (Phase 6)

From `monitor/src/config.rs`:

- **Server URL**: Required for run command, no format validation yet
- **Source ID**: Optional, defaults to hostname via `gethostname` crate
- **Key path**: Optional, defaults to `~/.vibetea`
- **Claude directory**: Optional, defaults to `~/.claude`
- **Buffer size**: Optional, defaults to 1000 events
- **Basename allowlist**: Optional extension filter for privacy

### Sanitization

| Data Type | Sanitization | Location |
|-----------|--------------|----------|
| Event payloads | Privacy pipeline processing | `monitor/src/privacy.rs:278-396` |
| Configuration strings | Trimmed in parsing | `config.rs` functions |
| File paths | Basename extraction only | `monitor/src/privacy.rs:433-442` |
| Base64 keys | Validated during signature verification | `auth.rs:204-215` |
| Signatures | Base64 decoding with error handling | `auth.rs:218-225` |
| Tokens | Trimmed and length-checked | `auth.rs:270-287` |
| Client token input | Trimmed before storage | `TokenForm.tsx:103` |
| JSONL lines | Whitespace trimmed, empty lines filtered | `monitor/src/parser.rs:348-350`, `monitor/src/watcher.rs:562-565` |
| File paths from tool input | Basename extraction via privacy pipeline | `monitor/src/privacy.rs:433-442` |
| Project names | URL decoding with validation | `monitor/src/parser.rs:491-529` |
| Tool context (sensitive tools) | Context set to None | `monitor/src/privacy.rs:366-389` |
| Summary text | Stripped to neutral message | `monitor/src/privacy.rs:351-355` |

## Data Protection

### Sensitive Data Handling

| Data Type | Protection | Storage | Notes |
|-----------|-----------|---------|-------|
| Private key | File permissions (0600) | `~/.vibetea/key.priv` | Monitor loads from disk (Phase 6) |
| Public key | Base64-encoded, file mode 0644 | `~/.vibetea/key.pub` and env var | On server and monitor (Phase 6) |
| Bearer token | Environment variable | `VIBETEA_SUBSCRIBER_TOKEN` | In-memory, passed by clients in query params |
| Client token | localStorage (browser) | `vibetea_token` key | Accessible to JavaScript, not secure storage (Phase 7) |
| Event payloads | Privacy pipeline sanitization | Memory/transit | Sent over HTTPS/WSS only |
| JSONL data | Read from disk | `~/.claude/projects/` | Watched by monitor, only metadata extracted |
| Tool context | Extension allowlist filtering | Memory/transit | Sensitive tools stripped, others filtered by extension |

### Encryption in Transit

| Channel | Protocol | Implementation |
|---------|----------|-----------------|
| Monitor → Server | HTTPS | TLS 1.2+ required (enforced by reqwest) |
| Server → Client | WSS (WebSocket Secure) | TLS 1.2+ (depends on deployment) |
| Client → Server (WebSocket) | WSS | TLS 1.2+ (depends on browser and deployment) |

**Deployment note**: VibeTea server endpoints must be served over HTTPS/WSS. Currently no explicit header configuration for security headers (HSTS, CSP, etc.).

### Encryption at Rest

| Data | Encryption | Key Management |
|------|-----------|-----------------|
| Event payloads | None | Not applicable |
| Private keys | None (file permissions) | OS filesystem security |
| Configuration | None | Environment variables |
| Client token | None (localStorage) | Browser's localStorage encryption (varies) |

**Note**: VibeTea does not implement application-level encryption. Sensitive credentials are protected by:
1. Environment variable isolation
2. File system permissions (private key in `~/.vibetea/key.priv` with mode 0600)
3. HTTPS/WSS transport security
4. Privacy pipeline sanitization (Phase 5)
5. Browser localStorage (Phase 7) - varies by browser and OS

## Cryptography

### Signature Scheme (Phase 6 - Full Implementation)

| Parameter | Value | Implementation |
|-----------|-------|-----------------|
| Algorithm | Ed25519 | `ed25519-dalek` 2.1 with `verify_strict()` |
| Key format | Base64-encoded public key | In `VIBETEA_PUBLIC_KEYS` and `~/.vibetea/key.pub` |
| Key storage | Raw 32-byte seed file | `~/.vibetea/key.priv` with mode 0600 |
| Key generation | OsRng (OS cryptographically secure RNG) | `Crypto::generate()` in `monitor/src/crypto.rs:88-94` |
| Signature verification | Per-event during POST /events | `server/src/routes.rs:289` |
| Constant-time token comparison | Via `subtle::ConstantTimeEq` | `server/src/auth.rs:290` |
| Dependencies | ed25519-dalek, base64, subtle, rand | Production-ready (Phase 3-6) |

**Status**: Ed25519 signature generation and verification fully implemented and tested. Token comparison uses constant-time comparison to prevent timing attacks.

### Keypair Generation and Storage (Phase 6)

From `monitor/src/crypto.rs`:

**Generation** (`Crypto::generate()`):
- Creates 32-byte seed using `rand::rng().fill()` with OS RNG
- Constructs `SigningKey` from seed bytes
- Returns `Crypto` struct wrapping the signing key

**Storage** (`Crypto::save()`):
- Creates `~/.vibetea/` directory if missing
- Writes raw 32-byte seed to `key.priv` with Unix mode 0600 (owner read/write only)
- Encodes public key as base64 and writes to `key.pub` with Unix mode 0644 (owner read/write, others read)
- Both operations atomic on Unix (file creation ensures exclusivity)

**Loading** (`Crypto::load()`):
- Reads raw 32-byte seed from `key.priv`
- Validates exactly 32 bytes read (rejects truncated files)
- Reconstructs `SigningKey` from seed bytes
- Returns error if file doesn't exist or has wrong length

**Public Key Export**:
- `public_key_base64()`: Returns base64-encoded 32-byte public key suitable for `VIBETEA_PUBLIC_KEYS`
- Example format: `base64-string-of-32-bytes` (44 characters with standard padding)

### Event Signing (Phase 6)

From `monitor/src/crypto.rs:259-275`:

**Signing Flow**:
1. Event serialized to JSON via `serde_json::to_string()`
2. JSON bytes passed to `Crypto::sign(json_bytes)`
3. `SigningKey::sign()` produces 64-byte Ed25519 signature (deterministic)
4. Signature base64-encoded for transmission in `X-Signature` header
5. Raw signature bytes available via `sign_raw()` if needed

**Signature Properties**:
- Deterministic: Same message always produces same signature
- Non-interactive: Only signing key needed (public key for verification)
- Unforgeable: Cannot create valid signature without private key
- Non-repudiation: Signer cannot deny signing

### Verification Implementation (Phase 3-6)

From `server/src/auth.rs`:

- `verify_signature()`: Handles full verification flow (lines 192-233)
  - Returns specific `AuthError` types for debugging
  - Validates key length and format
  - Uses strict verification per RFC 8032
  - Tested with 13 comprehensive test cases (lines 334-586)

- `validate_token()`: Handles constant-time token comparison (lines 269-295)
  - Trims whitespace from both sides
  - Performs length check first (acceptable, not timing-sensitive)
  - Uses `ct_eq()` for byte-level constant-time comparison
  - Tested with 15 test cases covering edge cases (lines 656-757)

### Key Management Best Practices

| Practice | Implementation | Status |
|----------|-----------------|--------|
| Key generation entropy | OS RNG via rand crate | Phase 6 |
| Private key permissions | Unix mode 0600 | Phase 6 |
| Public key distribution | Environment variable registration | Phase 3 |
| Key rotation | Manual (replace key.priv, update VIBETEA_PUBLIC_KEYS) | Not automated |
| Secure storage | File system with OS protections | Phase 6 |
| No hardcoded keys | Keys loaded from files/env | All phases |

## Privacy Pipeline (Phase 5)

### Privacy Guarantees

The privacy pipeline in `monitor/src/privacy.rs` provides Constitution I (Privacy by Design) guarantees:

| Guarantee | Implementation | Verification |
|-----------|----------------|--------------|
| No full paths | Path → basename conversion | `extract_basename()` function, 951 tests |
| No bash commands | Sensitive tools context stripped | SENSITIVE_TOOLS list, 303 tests |
| No grep patterns | Grep context set to None | Tool-specific filtering, 360 tests |
| No glob patterns | Glob context set to None | Pattern stripping, 416 tests |
| No web search queries | WebSearch context stripped | Extension to sensitive tools, 459 tests |
| No web fetch URLs | WebFetch context stripped | URL filtering, 502 tests |
| No summary text | Summary neutralized to "Session ended" | Text replacement, 548 tests |
| Extension allowlist | Optional filtering by file type | HashSet-based matching, 730 tests |

### PrivacyConfig

Configuration for privacy pipeline (`monitor/src/privacy.rs:85-220`):

- **Allowlist source**: `VIBETEA_BASENAME_ALLOWLIST` environment variable
- **Format**: Comma-separated extensions (e.g., `.rs,.ts,.md`)
- **Parsing**: Automatic dot-prefix addition if missing
- **Validation**: Filters out empty or invalid entries
- **Default**: No allowlist (all extensions allowed)

From `from_env()` at lines 136-158:
- Reads `VIBETEA_BASENAME_ALLOWLIST` from environment
- Splits on comma and trims whitespace
- Ensures each extension starts with dot
- Returns `None` if variable not set

### PrivacyPipeline

Event processing pipeline (`monitor/src/privacy.rs:222-396`):

**Processing rules**:
1. **Session events**: Pass through unchanged (project pre-sanitized by parser)
2. **Activity events**: Pass through unchanged
3. **Tool events**: Context processing based on tool type
   - Sensitive tools (Bash, Grep, Glob, WebSearch, WebFetch): context → None
   - Other tools: basename extraction + allowlist filtering
4. **Agent events**: Pass through unchanged
5. **Summary events**: Text replaced with "Session ended"
6. **Error events**: Pass through unchanged (category pre-sanitized)

**Tool context processing** (`process_tool_context()` at lines 366-389):
- Check if tool in `SENSITIVE_TOOLS` (line 368)
- Extract basename from path using `extract_basename()` (line 375)
- Apply allowlist filter if configured (line 381)
- Return processed context or None

### Basename Extraction

Function `extract_basename()` at lines 433-442:

```
Input: "/home/user/project/src/auth.rs"
Process: Path::new() → file_name() → to_str()
Output: Some("auth.rs")
```

Handles:
- Unix absolute paths: `/home/user/file.rs` → `file.rs`
- Windows paths: `C:\Users\user\file.rs` → `file.rs`
- Relative paths: `src/file.rs` → `file.rs`
- Already basenames: `file.rs` → `file.rs`
- Invalid inputs: `/`, empty string → None

### Test Coverage

`monitor/tests/privacy_test.rs` (951 lines) provides comprehensive privacy verification:

**Test categories**:
1. **Path stripping** (tests 1-2): 10 test cases for full path → basename conversion
2. **Bash command stripping** (test 3): 10 dangerous command patterns verified stripped
3. **Grep pattern stripping** (test 4): 7 sensitive patterns verified stripped
4. **Glob pattern stripping** (test 5): 7 glob patterns verified stripped
5. **WebSearch stripping** (test 6): 5 query patterns verified stripped
6. **WebFetch stripping** (test 7): 5 URL patterns verified stripped
7. **Summary stripping** (test 8): 5 sensitive summaries verified neutralized
8. **Comprehensive safety** (test 9): All event types with sensitive data verified safe
9. **Extension allowlist** (test 10): 6 filtered + 2 allowed file types verified
10. **Basename edge cases** (test 11+): Unicode, complex paths, case sensitivity

**Privacy assertions**:
- `assert_no_sensitive_paths()`: Verifies no path patterns in JSON
- `assert_no_sensitive_commands()`: Verifies no command patterns in JSON
- Individual checks for specific patterns per tool type

## HTTP Sender (Phase 6)

### Connection and Buffering

From `monitor/src/sender.rs`:

| Feature | Implementation | Details |
|---------|-----------------|---------|
| Connection pooling | reqwest Client with pool | 10 connections per host max |
| Request timeout | 30 seconds | Per-request timeout via `Client::timeout()` |
| Event buffering | VecDeque with FIFO eviction | 1000 events max (configurable) |
| Buffer status | `buffer_len()`, `is_empty()` | Methods to query buffer state |
| Graceful shutdown | `shutdown()` with timeout | Flushes remaining events before exit |

### Event Transmission

**Direct send** (`send(&event)`):
- Sends single event immediately without buffering
- Serializes event to JSON
- Signs JSON with private key (HMAC-like signing)
- Adds headers: `Content-Type: application/json`, `X-Source-Id`, `X-Signature`
- Retries with exponential backoff on transient failures

**Batch send** (`send_batch(events)`):
- Sends multiple events in single request
- JSON array serialization
- Single signature for entire batch
- Used internally by `flush()`

**Buffered queue** (`queue(event)`):
- Adds event to buffer without sending
- Evicts oldest events if buffer full
- Returns number of evicted events
- Used for background event accumulation

### Retry Strategy

From `monitor/src/sender.rs:350-387`:

| Parameter | Value | Purpose |
|-----------|-------|---------|
| Initial delay | 1 second | First retry wait time |
| Max delay | 60 seconds | Retry backoff ceiling |
| Jitter | ±25% | Prevents thundering herd |
| Max attempts | 10 | Retries before failure |
| Backoff formula | Exponential (delay * 2 each time) | Doubles until hitting max |

**Retry logic**:
1. On transient error (timeout, connection error, 5xx): `wait_with_backoff()` then retry
2. On rate limit (429): Parse `Retry-After` header, wait that duration, retry
3. On auth error (401): Immediate failure (no retry)
4. On client error (4xx except 429): Immediate failure (no retry)
5. On success (2xx): Reset retry delay to 1 second

**Rate limit handling** (lines 292-305):
- Checks for `Retry-After` header in 429 response
- Accepts seconds (integer) or HTTP date format
- Falls back to current retry delay if header missing
- Respects Retry-After before continuing retries

### Sender Configuration (Phase 6)

From `monitor/src/sender.rs:108-136`:

```rust
pub struct SenderConfig {
    pub server_url: String,           // e.g., "https://vibetea.fly.dev"
    pub source_id: String,            // Monitor identifier
    pub buffer_size: usize,           // Max events in buffer (default 1000)
}
```

**Configuration methods**:
- `SenderConfig::new(url, source_id, buffer_size)` - Full config
- `SenderConfig::with_defaults(url, source_id)` - Uses 1000 as buffer size

**Integration with Monitor** (Phase 6 main.rs):
```rust
let sender_config = SenderConfig::new(
    config.server_url.clone(),
    config.source_id.clone(),
    config.buffer_size,
);
let mut sender = Sender::new(sender_config, crypto);
```

### Error Handling

From `monitor/src/sender.rs:76-105`:

| Error Type | Cause | Recovery |
|------------|-------|----------|
| `Http(reqwest::Error)` | HTTP client failure | Non-retryable |
| `ServerError { status, message }` | 4xx or 5xx response | Retryable for 5xx |
| `AuthFailed` | 401 Unauthorized | Non-retryable |
| `RateLimited { retry_after_secs }` | 429 Too Many Requests | Retryable with backoff |
| `BufferOverflow { evicted_count }` | Buffer full on queue | Oldest events discarded |
| `MaxRetriesExceeded { attempts }` | 10 retries exhausted | Operation fails |
| `Json(serde_json::Error)` | Event serialization failed | Non-retryable |

## CLI Commands (Phase 6)

### Monitor Init Command

From `monitor/src/main.rs:147-190`:

**Purpose**: Generate and register Ed25519 keypair for server authentication

**Flow**:
1. Check if keys exist: `Crypto::exists(&key_dir)`
2. If exist and not `--force`: Prompt user for confirmation
3. Generate keypair: `Crypto::generate()`
4. Save to `~/.vibetea/`: `crypto.save(&key_dir)`
5. Display public key for server registration
6. Show `VIBETEA_PUBLIC_KEYS` format example

**Output**:
```
Keypair saved to: /home/user/.vibetea

Public key (register with server):

  <base64-encoded-32-bytes>

To register this monitor with the server, add to VIBETEA_PUBLIC_KEYS:

  export VIBETEA_PUBLIC_KEYS="monitor-name:<public-key>"
```

**Security**:
- Prompts confirmation before overwriting existing keys
- `--force` flag skips confirmation for automation
- Private key file created with mode 0600 (owner-only access)
- Public key file created with mode 0644 (world-readable)

### Monitor Run Command

From `monitor/src/main.rs:192-245`:

**Purpose**: Start the monitor daemon for continuous session monitoring

**Flow**:
1. Initialize structured logging: `init_logging()`
2. Load config from environment: `Config::from_env()`
3. Load cryptographic keys: `Crypto::load(&config.key_path)`
4. Create HTTP sender with pooling: `Sender::new(config, crypto)`
5. Set up file watcher (placeholder for Phase 7)
6. Wait for shutdown signal: `wait_for_shutdown()`
7. Graceful shutdown: `sender.shutdown(5_second_timeout)`
8. Report unflushed events if any

**Environment setup**:
- Required: `VIBETEA_SERVER_URL`
- Optional: `VIBETEA_SOURCE_ID` (defaults to hostname)
- Optional: `VIBETEA_KEY_PATH` (defaults to `~/.vibetea`)
- Optional: `VIBETEA_CLAUDE_DIR` (defaults to `~/.claude`)
- Optional: `VIBETEA_BUFFER_SIZE` (defaults to 1000)

**Logging**:
- Structured logging via `tracing` crate
- Log level from `RUST_LOG` env var (default: info)
- Includes target module and level in output

**Shutdown handling**:
- Listens for SIGINT (Ctrl+C) and SIGTERM (unix)
- Waits up to 5 seconds to flush buffered events
- Reports unsent events in error log
- Graceful exit code 0

### Help Command

From `monitor/src/main.rs:101-137`:

**Purpose**: Display usage information

**Output format**:
- Commands list (init, run, help, version)
- Options per command
- Environment variable reference
- Example usage patterns

### Version Command

From `monitor/src/main.rs:139-145`:

**Purpose**: Display application version

**Output**: `vibetea-monitor X.Y.Z` from `CARGO_PKG_VERSION`

## Rate Limiting

### Implementation

| Aspect | Details | Location |
|--------|---------|----------|
| Algorithm | Token bucket | `server/src/rate_limit.rs:94-196` |
| Rate | Configurable (default 100 tokens/sec) | `server/src/rate_limit.rs:42-46` |
| Capacity | Configurable (default 100 tokens) | `server/src/rate_limit.rs:42-46` |
| Per-source tracking | `HashMap<String, TokenBucket>` | `server/src/rate_limit.rs:233-243` |
| Granularity | Per X-Source-ID header | `server/src/routes.rs:307` |
| Status | Fully implemented and integrated | `server/src/main.rs:84-91` |

### Rate Limiter Details

From `server/src/rate_limit.rs`:

**Token Bucket Algorithm**:
- Each source gets independent bucket with `capacity` tokens
- Tokens refill at `rate` tokens per second
- Each request consumes 1 token
- No tokens = request rejected with 429 Too Many Requests
- Retry-After header indicates seconds until next token available

**RateLimiter Structure**:
- Thread-safe via `RwLock` for concurrent access
- Automatic bucket creation on first request per source
- Stale entry cleanup every 60 seconds (configurable via `cleanup_stale_entries_with_timeout()`)
- Background cleanup task spawned at server startup (line 85-91 in main.rs)

**Integration**:
- Middleware check in `server/src/routes.rs:306-322`
- Returns 429 with `Retry-After` header when limited
- Applied to all event ingestion regardless of auth mode
- Independent per source ID to prevent cross-source throttling

### Response Handling

From `server/src/routes.rs:307-322`:

```
If allowed: Continue to authentication/processing
If limited: Return 429 Too Many Requests
  - Header: Retry-After: {seconds}
  - Body: {"error": "rate limit exceeded", "code": "rate_limited"}
  - Log: info!(source, retry_after, "Rate limit exceeded")
```

## CORS Configuration

| Setting | Value | Purpose |
|---------|-------|---------|
| Allowed origins | Via tower-http (not configured) | Not yet configured |
| Methods | GET, POST | Likely via tower-http |
| Headers | Authorization, Content-Type | Likely via tower-http |
| Credentials | true | If client auth needed |

**Status**: CORS is available via `tower-http` dependency but configuration is pending implementation.

## Security Headers

Not yet configured. Recommended headers for production:

| Header | Recommended Value | Purpose |
|--------|-------------------|---------|
| Strict-Transport-Security | `max-age=31536000; includeSubDomains` | HTTPS enforcement |
| X-Content-Type-Options | `nosniff` | MIME sniffing prevention |
| X-Frame-Options | `DENY` | Clickjacking protection |
| Content-Security-Policy | `default-src 'self'` | XSS protection |

**Action needed**: Configure via tower-http middleware before production.

## Secrets Management

### Environment Variables

| Category | Variable | Required | Default | Notes |
|----------|----------|----------|---------|-------|
| Server - Auth | `VIBETEA_PUBLIC_KEYS` | Yes* | - | Format: `id1:b64key1,id2:b64key2` |
| Server - Auth | `VIBETEA_SUBSCRIBER_TOKEN` | Yes* | - | Bearer token for clients |
| Server - Config | `PORT` | No | 8080 | HTTP port |
| Server - Dev | `VIBETEA_UNSAFE_NO_AUTH` | No | false | Development bypass |
| Monitor - URL | `VIBETEA_SERVER_URL` | Yes | - | Server endpoint URL |
| Monitor - Identity | `VIBETEA_SOURCE_ID` | No | hostname | Monitor name |
| Monitor - Keys | `VIBETEA_KEY_PATH` | No | `~/.vibetea` | Directory with keys |
| Monitor - Watch | `VIBETEA_CLAUDE_DIR` | No | `~/.claude` | Claude Code directory |
| Monitor - Tuning | `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |
| Monitor - Filter | `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated extensions (Phase 5) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true` (dev only)

### Secrets Storage by Environment

| Environment | Method | Tool |
|-------------|--------|------|
| Development | `.env.local` | Gitignored local file |
| CI/CD | GitHub Secrets | Repository settings |
| Production | Environment variables | Deployment platform (Fly.io) |

**Note**: No `.env` example file included in repo. Secrets are documented here and set during deployment.

## Audit Logging

| Event | Logged Data | Implementation |
|-------|-------------|-----------------|
| Auth failures | Via error variants and warn logs | `server/src/routes.rs:290, 459` |
| Rate limiting | source identifier and retry_after | `server/src/routes.rs:310-314` |
| Configuration errors | At startup | Via `tracing::warn!` in config validation |
| Development mode enabled | Warning message | In `server/src/config.rs:94-98` |
| WebSocket connections | Connection/disconnection events | `server/src/routes.rs:493, 554` |
| Event broadcasts | Per-event trace logs | `server/src/routes.rs:345-350` |
| File watcher events | File creation/modification/removal | `monitor/src/watcher.rs:329-362, 366-400, 426-456` |
| Parser events | JSONL parsing failures logged as warnings | `monitor/src/parser.rs:357` |
| Privacy pipeline | Debug logs for context processing | `monitor/src/privacy.rs:369, 382, 385` |
| Monitor startup | Config loaded, keys loaded, running state | `monitor/src/main.rs:197-230` |
| Event transmission | Events sent successfully | `monitor/src/sender.rs:284` |
| Sender errors | Auth failed, rate limited, retries | `monitor/src/sender.rs:289-322` |
| Client WebSocket messages | Invalid message format logged | `useWebSocket.ts:119-120` |
| Token operations | Token save/clear in client | `TokenForm.tsx:108, 121` (localStorage ops, not logged) |

**Status**: Basic error logging present. Structured auth decision logging and comprehensive audit trails pending.

## Error Handling Security

### Information Disclosure

| Error Type | Message Content | Risk |
|------------|-----------------|------|
| Auth failures | Specific error codes | Low - helpful for debugging |
| Unknown source | "unknown source: {source_id}" | Low - expected behavior |
| Invalid signature | "invalid signature" | Low - doesn't expose details |
| Base64 errors | "invalid base64 encoding for {field}" | Low - field name only |
| Rate limit | "rate limit exceeded" | Low - expected for clients |
| Config errors | "configuration validation failed" | Low - visible only to operator |
| Internal errors | "server configuration error" | Low - no sensitive details exposed |
| Parser errors | Logged as warnings without details | Low - JSON parsing failures don't expose content |
| Privacy filter debug | Debug logs only, not exposed in responses | Low - development visibility only |
| Crypto errors | File not found, invalid key length | Low - no key material exposed |
| Client token errors | Warning to console if token missing | Low - development debugging only |
| Invalid WebSocket message | Silently dropped, no client error | Low - malformed events ignored |

### Error Response Handling

Errors from `server/src/routes.rs:188-208` and `server/src/auth.rs:49-92`:

- `AuthError::UnknownSource` - Returns 401 "unknown source"
- `AuthError::InvalidSignature` - Returns 401 "invalid signature"
- `AuthError::InvalidBase64` - Returns 401 "invalid signature encoding"
- `AuthError::InvalidPublicKey` - Returns 500 "server configuration error"
- `AuthError::InvalidToken` - Returns 401 "invalid token"
- Rate limit errors - Returns 429 with Retry-After
- Parser errors - Logged as warnings, non-fatal to file monitoring
- Sender errors - Logged with context, retried or reported during shutdown

No SQL errors, path traversal details, or stack traces exposed to clients.

## Dependency Security

### Core Security Dependencies

| Package | Version | Purpose | Status |
|---------|---------|---------|--------|
| ed25519-dalek | 2.1 | Cryptographic signing/verification | Current, production |
| base64 | 0.22 | Key/signature encoding | Current |
| subtle | Latest | Constant-time comparison | Critical for security |
| reqwest | 0.12 | HTTPS client | Current |
| serde | 1.0 | Safe deserialization | Current |
| serde_json | Latest | JSON parsing | Current |
| rand | 0.9 | Random number gen | Current |
| tokio | Latest | Async runtime | Current |
| axum | Latest | HTTP framework | Current |
| thiserror | Latest | Error handling | Current |
| uuid | Latest | Session IDs | Current |
| chrono | Latest | Timestamps | Current |
| notify | Latest | File watching | Current (Phase 4) |
| directories | Latest | Home directory resolution | Current (Phase 4) |
| tracing | Latest | Structured logging | Current (Phase 5) |
| gethostname | Latest | Monitor hostname detection | Current (Phase 6) |
| anyhow | Latest | Context error handling | Current (Phase 6) |

### Dependency Audit

**Status**: No known CVEs in current dependencies (as of 2026-02-02).

**Process**: To check for vulnerabilities, install and run cargo-audit:
```bash
cargo install cargo-audit
cd /home/ubuntu/Projects/VibeTea && cargo audit
```

## Client-Side Security (TypeScript)

### Event Types and Validation

From `client/src/types/events.ts:1-248`:

- **EventType**: String discriminator union validated via `isValidEventType()`
- **VibeteaEvent**: Generic interface with type-safe payload mapping
- **Type guards**: Runtime validators for all event types
- **readonly fields**: Immutability for all payload interfaces

### State Management

From `client/src/hooks/useEventStore.ts:1-172`:

- **Zustand store**: Centralized state with selective subscriptions
- **Event buffer**: Last 1000 events with FIFO eviction (no size limit security risk)
- **Session tracking**: Derived from event stream aggregation
- **No authentication state**: Bearer token not stored in client-side store
- **No sensitive data**: Events are passed through without validation

### WebSocket Connection (Phase 7)

From `client/src/hooks/useWebSocket.ts`:

**Token Management**:
- Token stored in localStorage under `vibetea_token` key
- Retrieved at connection time from localStorage
- Built into WebSocket URL as query parameter: `?token={token}`
- No token expiration check on client side
- No refresh token mechanism

**Reconnection**:
- Exponential backoff: 1s → 60s with ±25% jitter
- Auto-reconnect enabled by default, disabled on manual disconnect
- Attempt counter resets on successful connection
- Missing token prevents connection with console warning

**Event Handling**:
- Events validated with `parseEventMessage()` (structural check only)
- Invalid events silently dropped
- Valid events dispatched to Zustand store
- No schema validation against expected event types

### Client Authorization Gaps

- No token management implementation (localStorage only)
- No authorization checks in event filtering
- No rate limiting on client side
- No CORS origin validation (server responsibility)
- All authenticated clients see all events
- No user-scoped filtering of data

### Token Form Component (Phase 7)

From `client/src/components/TokenForm.tsx`:

**Token Storage**:
- Password input field for user entry
- Stored in localStorage via `localStorage.setItem()`
- Retrieved and checked via `localStorage.getItem()`
- Cleared via `localStorage.removeItem()`
- Cross-tab sync via `storage` event listener

**Security Properties**:
- Token displayed as password field (hidden from UI)
- Token trimmed before storage (removes whitespace)
- Empty token submission rejected
- Clear button allows immediate token removal
- Status indicator shows saved/not-saved state

**Security Gaps**:
- localStorage is not secure storage (accessible to all JavaScript)
- No encryption of token at rest (browser-dependent)
- Token visible in browser dev tools and network tab
- No CSRF token for form submission
- No HTTPS enforcement at client level

### Connection Status Component (Phase 7)

From `client/src/components/ConnectionStatus.tsx`:

- Visual status indicator (colored dot + optional label)
- Color mapping: Green (connected), Yellow (connecting/reconnecting), Red (disconnected)
- Selective Zustand subscription to status changes
- No authentication or authorization checks
- Purely informational UI component

## Phase 7 Security Changes

New client-side components for user authentication:

### TokenForm Component (`client/src/components/TokenForm.tsx`)

- **Purpose**: Manage bearer token for WebSocket authentication
- **Storage**: localStorage (browser storage, not cryptographically protected)
- **Key**: `vibetea_token` (consistent with useWebSocket)
- **User interaction**: Password input with save/clear buttons
- **Status tracking**: Visual indicator of saved/not-saved state
- **Cross-tab sync**: Listens to storage events for multi-window updates

Security properties:
- Token hidden in password field during input
- Token trimmed before storage
- Token never logged or displayed after saving
- Clear button allows immediate token removal
- localStorage accessible to all JavaScript in origin

### useWebSocket Hook (`client/src/hooks/useWebSocket.ts`)

- **Purpose**: Manage WebSocket connection with auto-reconnect
- **Token retrieval**: localStorage.getItem(TOKEN_STORAGE_KEY)
- **Connection URL**: Includes token as query parameter: `?token={token}`
- **Auto-reconnect**: Exponential backoff (1s → 60s, ±25% jitter)
- **Event validation**: Basic structural validation via parseEventMessage()
- **Status tracking**: connection state (connecting/connected/disconnected/reconnecting)

Security properties:
- Token missing blocks connection with console warning
- Reconnection disabled on manual disconnect
- Attempt counter resets on successful connection
- Invalid events silently dropped
- No schema validation of event structure

### ConnectionStatus Component (`client/src/components/ConnectionStatus.tsx`)

- **Purpose**: Visual indicator of WebSocket connection status
- **Display**: Colored dot (green/yellow/red) with optional label
- **Selective rendering**: Zustand subscription prevents unnecessary re-renders
- **ARIA labels**: Accessibility support for screen readers
- **No security impact**: Purely informational

## Known Vulnerabilities & Gaps

**Fixed in Phase 3:**
- Ed25519 signature verification fully implemented with strict verification
- Token comparison using constant-time comparison to prevent timing attacks
- Per-source rate limiting with token bucket algorithm
- Comprehensive error handling with specific AuthError variants

**Fixed in Phase 5:**
- Privacy pipeline fully implemented and tested
- Extension allowlist filtering for sensitive file types
- Bash/Grep/Glob/WebSearch/WebFetch context stripping
- Summary text neutralization
- Path anonymization via basename extraction

**Fixed in Phase 6:**
- Keypair generation with OS RNG entropy
- Secure key storage with proper Unix file permissions (0600)
- Event signing implementation (deterministic Ed25519)
- HTTP sender with connection pooling and retry logic
- Rate limit handling with Retry-After respect
- Monitor CLI with init and run commands
- Graceful shutdown with event buffer flushing
- Structured logging throughout monitor components

**New in Phase 7:**
- Client-side token management via TokenForm component
- WebSocket connection with auto-reconnect via useWebSocket hook
- Connection status visual indicator via ConnectionStatus component
- Token storage in localStorage with cross-tab sync
- Basic event validation via parseEventMessage()

**Remaining gaps:**
- No rate limiting middleware for other endpoints (only event ingestion protected)
- No granular authorization/RBAC (design phase)
- No encryption at rest for configuration/events (acceptable for MVP)
- No comprehensive audit logging beyond error messages
- No CORS header configuration (pending)
- No secure token storage (localStorage is insecure)
- No token refresh mechanism (client side)
- No token expiration validation (client side)
- No per-client isolation or scoping (all clients see all events)
- No TLS certificate validation in monitor HTTP client (reqwest default)
- No URL format validation in monitor config (pending)
- No integration tests for watcher + parser + privacy + sender pipeline
- No CSRF protection on token form
- No encryption of token in localStorage (browser-dependent)

---

## What Does NOT Belong Here

- Tech debt and risks → CONCERNS.md
- Testing strategy → TESTING.md
- Code conventions → CONVENTIONS.md

---

*This document defines security controls. Update when security posture changes.*
