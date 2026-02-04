# External Integrations

**Status**: Phase 6 - GitHub Actions composite action for simplified monitor integration
**Last Updated**: 2026-02-04

## Summary

VibeTea is designed as a distributed event system with three components:
- **Monitor**: Captures Claude Code session events from local JSONL files, applies privacy sanitization, signs with Ed25519, and transmits to server via HTTP. Supports `export-key` command for GitHub Actions integration (Phase 4). Can be deployed in GitHub Actions workflows (Phase 5). Integrated via reusable GitHub Actions composite action (Phase 6).
- **Server**: Receives, validates, verifies Ed25519 signatures, and broadcasts events via WebSocket
- **Client**: Subscribes to server events via WebSocket for visualization with token-based authentication

All integrations use standard protocols (HTTPS, WebSocket) with cryptographic message authentication and privacy-by-design data handling.

## File System Integration

### Claude Code JSONL Files

**Source**: `~/.claude/projects/**/*.jsonl`
**Format**: JSON Lines (one JSON object per line)
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Parser Location**: `monitor/src/parser.rs` (SessionParser, ParsedEvent, ParsedEventKind)
**Watcher Location**: `monitor/src/watcher.rs` (FileWatcher, WatchEvent)
**Privacy Pipeline**: `monitor/src/privacy.rs` (PrivacyConfig, PrivacyPipeline, extract_basename)

**Privacy-First Approach**:
- Only metadata extracted: tool names, timestamps, file basenames
- Never processes code content, prompts, or assistant responses
- File path parsing for project name extraction (slugified format)
- All event payloads pass through PrivacyPipeline before transmission

**Session File Structure**:
```
~/.claude/projects/<project-slug>/<session-uuid>.jsonl
```

**Supported Event Types** (from Claude Code JSONL):
| Claude Code Type | Parsed As | VibeTea Event | Fields Extracted |
|------------------|-----------|---------------|------------------|
| `assistant` with `tool_use` | Tool invocation | ToolStarted | tool name, context |
| `progress` with `PostToolUse` | Tool completion | ToolCompleted | tool name, success |
| `user` | User activity | Activity | timestamp only |
| `summary` | Session end marker | Summary | session metadata |
| File creation | Session start | SessionStarted | project from path |

**Watcher Behavior**:
- Monitors `~/.claude/projects/` directory recursively
- Detects file creation, modification, deletion events
- Maintains position map for efficient tailing (no re-reading)
- Emits WatchEvent::FileCreated, WatchEvent::LinesAdded, WatchEvent::FileRemoved
- Automatic cleanup of removed files from tracking state

**Configuration** (`monitor/src/config.rs`):
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | Claude Code directory to monitor |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated file extensions to watch |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |

## Privacy & Data Sanitization

### Privacy Pipeline Architecture

**Location**: `monitor/src/privacy.rs` (1039 lines)

**Core Components**:

1. **PrivacyConfig** - Configuration management
   - Optional extension allowlist (e.g., `.rs`, `.ts`)
   - Loaded from `VIBETEA_BASENAME_ALLOWLIST` environment variable
   - Supports comma-separated format: `.rs,.ts,.md` or `rs,ts,md` (auto-dots)
   - Whitespace-tolerant: ` .rs , .ts ` normalized to `[".rs", ".ts"]`
   - Empty entries filtered: `.rs,,.ts,,,` becomes `[".rs", ".ts"]`

2. **PrivacyPipeline** - Event sanitization processor
   - Processes EventPayload before transmission to server
   - Strips sensitive contexts from dangerous tools
   - Extracts basenames from file paths
   - Applies extension allowlist filtering
   - Neutralizes session summary text

3. **extract_basename()** - Path safety function
   - Converts full paths to secure basenames
   - Handles Unix: `/home/user/src/auth.ts` → `auth.ts`
   - Handles Windows: `C:\Users\user\src\auth.ts` → `auth.ts`
   - Handles relative: `src/auth.ts` → `auth.ts`
   - Returns `None` for invalid/empty paths

**Sensitive Tools** (context always stripped):
- `Bash` - Commands may contain secrets, passwords, API keys
- `Grep` - Patterns reveal what user is searching for
- `Glob` - File patterns reveal project structure
- `WebSearch` - Queries reveal user intent
- `WebFetch` - URLs may contain sensitive parameters

**Privacy Processing Rules**:

| Payload Type | Processing |
|--------------|-----------|
| Session | Pass through (project already sanitized at parse time) |
| Activity | Pass through unchanged |
| Tool (sensitive) | Context set to `None` |
| Tool (other) | Context → basename, apply allowlist, pass if allowed else `None` |
| Agent | Pass through unchanged |
| Summary | Summary text replaced with "Session ended" |
| Error | Pass through (category already sanitized) |

**Extension Allowlist Filtering**:
- When `VIBETEA_BASENAME_ALLOWLIST` is not set: All extensions allowed
- When set to `.rs,.ts`: Only `.rs` and `.ts` files transmitted; others filtered to `None`
- If no extension and allowlist set: Context filtered to `None`
- Examples:
  - `file.rs` with allowlist `.rs,.ts` → ALLOWED
  - `file.py` with allowlist `.rs,.ts` → FILTERED
  - `Makefile` with allowlist `.rs,.ts` → FILTERED (no extension)

**Example Privacy Processing**:
```
Input:  Tool { context: Some("/home/user/project/src/auth.rs"), tool: "Read", ... }
Output: Tool { context: Some("auth.rs"), tool: "Read", ... }

Input:  Tool { context: Some("rm -rf /home"), tool: "Bash", ... }
Output: Tool { context: None, tool: "Bash", ... }  # Sensitive tool

Input:  Tool { context: Some("/home/user/config.py"), tool: "Read", allowlist: [.rs,.ts] }
Output: Tool { context: None, tool: "Read", ... }  # Filtered by allowlist
```

### Privacy Test Suite

**Location**: `monitor/tests/privacy_test.rs` (951 lines)

**Coverage**: 18+ comprehensive privacy compliance tests
**Validates**: Constitution I (Privacy by Design)

**Test Categories**:
1. **Path Sanitization**
   - No full paths in output (Unix, Windows, relative)
   - Basenames correctly extracted
   - Hidden files handled

2. **Sensitive Tool Stripping**
   - Bash commands removed entirely
   - Grep patterns omitted
   - Glob patterns stripped
   - WebSearch queries removed
   - WebFetch URLs removed

3. **Content Stripping**
   - File contents never transmitted
   - Diffs excluded from payloads
   - Code excerpts removed

4. **Prompt/Response Stripping**
   - User prompts not included
   - Assistant responses excluded
   - Message content sanitized

5. **Command Argument Removal**
   - Arguments separated from descriptions
   - Descriptions allowed for Bash context
   - Actual commands never sent

6. **Summary Neutralization**
   - Summary text set to generic "Session ended"
   - Original text discarded
   - No content leakage

7. **Extension Allowlist Filtering**
   - Correct files allowed through
   - Disallowed extensions filtered
   - No-extension files handled properly

8. **Sensitive Pattern Detection**
   - Path patterns never appear (e.g., `/home/`, `/Users/`, `C:\`)
   - Command patterns removed (e.g., `rm -rf`, `sudo`, `curl -`, `Bearer`)
   - Credentials not transmitted

## Cryptographic Authentication & Key Management

### Phase 2: Enhanced Crypto Module with KeySource Tracking

**Module Location**: `monitor/src/crypto.rs` (438+ lines)

**KeySource Enum** (Phase 2 Addition):
- **Purpose**: Track where the private key was loaded from for audit/logging purposes
- **Variants**:
  - `EnvironmentVariable` - Key loaded from `VIBETEA_PRIVATE_KEY` environment variable
  - `File(PathBuf)` - Key loaded from file at specific path
- **Usage**: Enables reporting key source at startup for transparency
- **Logging**: Can be reported at INFO level to help users verify correct key usage

**Public Key Fingerprinting** (Phase 2 Addition):
- **public_key_fingerprint()**: New method returns first 8 characters of base64-encoded public key
  - Used for key verification in logs without exposing full key
  - Allows users to verify correct keypair with server registration
  - Always 8 characters long, guaranteed to be unique prefix of full key
  - Useful for quick visual verification in logs and documentation
  - Example: Full key `dGVzdHB1YmtleTExYWJjZGVmZ2hpams=` → Fingerprint `dGVzdHB1`

**Backward Compatibility**:
- KeySource and fingerprinting are tracking/logging features only
- Do not affect cryptographic operations (signing/verification)
- Existing code continues to work without modification
- New features are opt-in for enhanced observability

### Phase 3: Memory Safety & Environment Variable Key Loading

**Module Location**: `monitor/src/crypto.rs` (438+ lines)

**zeroize Crate Integration** (v1.8):
- Securely wipes sensitive memory (seed bytes, decoded buffers) after use
- Applied in key generation: seed zeroized after SigningKey construction
- Applied in load_from_env(): decoded buffer zeroized on both success and error paths
- Applied in load_with_fallback(): decoded buffer zeroized on error paths
- Prevents sensitive key material from remaining in memory dumps
- Complies with FR-020: Zero intermediate key material after key operations

**load_from_env() Method** (Phase 3 Addition):
- Loads Ed25519 private key from `VIBETEA_PRIVATE_KEY` environment variable
- Expects base64-encoded 32-byte seed (RFC 4648 standard)
- Trims whitespace (including newlines) before decoding
- Returns tuple: (Crypto instance, KeySource::EnvironmentVariable)
- Validates decoded length is exactly 32 bytes
- Error on missing/empty/invalid base64/wrong length
- Uses zeroize on both success and error paths
- Enables flexible key management without modifying code
- Example usage:
  ```bash
  export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)
  # Monitor loads from env var on next run
  ```

**load_with_fallback() Method** (Phase 3 Addition):
- Implements key precedence: environment variable first, then file
- If `VIBETEA_PRIVATE_KEY` is set, loads from it with NO fallback on error
- If env var not set, loads from `{dir}/key.priv` file
- Returns tuple: (Crypto instance, KeySource indicating source)
- Enables flexible key management without code changes
- Error handling: env var errors are terminal (no fallback)
- Useful for deployment workflows with different key sources

**seed_base64() Method** (Phase 3 Addition):
- Exports private key as base64-encoded string
- Inverse of load_from_env() for key migration workflows
- Suitable for storing in `VIBETEA_PRIVATE_KEY` environment variable
- Marked sensitive: handle with care, avoid logging
- Used for user-friendly key export workflows
- Example: `vibetea-monitor export-key` displays exportable key format

**CryptoError::EnvVar Variant** (Phase 3 Addition):
- New error variant for environment variable issues
- Returned when `VIBETEA_PRIVATE_KEY` is missing or empty
- Distinct from file-based key loading errors
- Enables precise error handling and logging

### Phase 4: Export-Key Command for GitHub Actions

**CLI Command Location**: `monitor/src/main.rs` (lines 101-109, 180-202)

**export-key Subcommand** (FR-003, FR-023, FR-026, FR-027, FR-028):
- **Command**: `vibetea-monitor export-key [--path <PATH>]`
- **Purpose**: Export private key for use in GitHub Actions secrets or other deployment systems
- **Implementation**: Loads key from disk via `Crypto::load()` (not environment variable)
- **Output**: Base64-encoded seed to stdout followed by exactly one newline
- **Diagnostics**: All error messages and logging go to stderr only
- **Exit Codes**:
  - 0 on success
  - 1 on configuration error (missing key, invalid path)
- **Features**:
  - Suitable for piping to clipboard tools (`pbpaste`, `xclip`)
  - Suitable for piping to secret management systems
  - No ANSI escape codes, no carriage returns
  - Clean output for automation and scripting
  - Optional `--path` argument for custom key directory

**run_export_key() Function** (lines 180-202):
- Accepts optional `path` parameter from `--path` flag
- Defaults to `get_key_directory()` if not provided
- Calls `Crypto::load()` to read from disk only
- Prints base64 seed to stdout with single trailing newline
- Errors printed to stderr with helpful context
- Exit code 1 if key file not found
- Example stderr message: "Error: No key found at /path/to/keys/key.priv"

**Usage Examples**:
```bash
# Export to environment variable for local testing
export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)

# Export to GitHub Actions secret
EXPORTED_KEY=$(vibetea-monitor export-key)
gh secret set VIBETEA_PRIVATE_KEY --body "$EXPORTED_KEY"

# Export from custom key directory
vibetea-monitor export-key --path ~/.keys/vibetea

# Pipe directly to file
vibetea-monitor export-key > private_key.txt
```

**Integration Test Suite** (`monitor/tests/key_export_test.rs` - 699 lines):

**Framework**:
- Uses `serial_test` crate with `#[serial]` attribute
- Ensures tests run with `--test-threads=1` to prevent env var interference
- **EnvGuard RAII pattern**: Saves/restores environment variables on drop for isolation

**Test Coverage** (13 tests total):

1. **Round-trip Tests** (FR-027, FR-028):
   - `roundtrip_generate_export_command_import_sign_verify`
     - Generate new key → Save → Export via command → Load from env → Sign message → Verify signature
     - Validates exported key can be loaded and used for cryptography
   - `roundtrip_export_command_signatures_are_identical`
     - Verifies Ed25519 determinism: same key produces identical signatures
     - Tests that exported key produces same signatures as original

2. **Output Format Tests** (FR-003):
   - `export_key_output_format_base64_with_single_newline`
     - Validates exact format: base64 seed + exactly one newline
     - No leading/trailing whitespace other than final newline
   - `export_key_output_is_valid_base64_32_bytes`
     - Decodes output as base64 and verifies 32-byte length
     - Ensures cryptographic validity of exported data

3. **Diagnostic Output Tests** (FR-023):
   - `export_key_diagnostics_go_to_stderr`
     - Confirms stdout contains only base64 characters
     - No diagnostic patterns in stdout (no labels, no prose)
   - `export_key_error_messages_go_to_stderr`
     - Verifies errors written to stderr, not stdout
     - Stdout empty on error, stderr contains error message

4. **Exit Code Tests** (FR-026):
   - `export_key_exit_code_success` - Returns 0 on success
   - `export_key_exit_code_missing_key_file` - Returns 1 for missing key.priv
   - `export_key_exit_code_nonexistent_path` - Returns 1 for non-existent directory

5. **Edge Case Tests**:
   - `export_key_handles_path_with_spaces` - Paths with spaces handled correctly
   - `export_key_suitable_for_piping` - No ANSI codes, no carriage returns for clean piping
   - `export_key_reads_from_key_priv_file` - Verifies correct file is read (key.priv)

**Test Infrastructure**:
- Uses `tempfile` crate for isolated test directories (no interference)
- Uses `Command::new()` to invoke vibetea-monitor binary
- Tests find compiled binary via `get_monitor_binary_path()`
- Uses `base64` crate for decoding verification
- Uses `ed25519_dalek::Verifier` for signature validation
- All tests marked with `#[test]` and `#[serial]` attributes
- Comprehensive error message assertions with stderr capture

**Requirements Addressed**:
- **FR-003**: Export-key command outputs base64 key with single newline (piping-friendly)
- **FR-023**: Diagnostics on stderr, key only on stdout (machine-readable)
- **FR-026**: Exit codes 0 (success), 1 (config/missing key error), 2 (runtime error)
- **FR-027**: Exported key can be loaded via `VIBETEA_PRIVATE_KEY` environment variable
- **FR-028**: Round-trip verified: generate → export → load → sign → verify

### Phase 6: Monitor Cryptographic Operations

**Module Location**: `monitor/src/crypto.rs` (438 lines)

**Crypto Module Features**:

1. **Keypair Generation**
   - `Crypto::generate()` creates new Ed25519 keypair
   - Uses OS cryptographically secure RNG via `rand` crate
   - Returns Crypto struct managing SigningKey

2. **Key Persistence**
   - `save(dir)` writes keypair to files
   - Private key: `key.priv` (raw 32-byte seed, permissions 0600)
   - Public key: `key.pub` (base64-encoded, permissions 0644)
   - Creates directory if not present
   - Error on invalid file permissions (Unix)

3. **Key Loading**
   - `load(dir)` reads existing keypair
   - Validates private key is exactly 32 bytes
   - Returns CryptoError if format invalid
   - Reconstructs SigningKey from seed bytes

4. **Key Existence Check**
   - `exists(dir)` checks if private key file present
   - Used to prevent accidental overwrite

5. **Public Key Export**
   - `public_key_base64()` returns base64-encoded public key
   - Format suitable for `VIBETEA_PUBLIC_KEYS` environment variable
   - Derived from SigningKey via VerifyingKey

6. **Event Signing**
   - `sign(message)` returns base64-encoded Ed25519 signature
   - Message is JSON-encoded event payload (bytes)
   - Signature verifiable by server with public key
   - Uses RFC 8032 compliant signing via ed25519-dalek

**CryptoError Types**:
- `Io` - File system errors
- `InvalidKey` - Seed not 32 bytes or malformed
- `Base64` - Public key decoding error
- `KeyExists` - Files already present (can be overwritten)
- `EnvVar` - Environment variable missing or empty (Phase 3)

**File Locations** (configurable):
- Default key directory: `~/.vibetea/`
- Override with `VIBETEA_KEY_PATH` environment variable
- Private key: `{key_dir}/key.priv`
- Public key: `{key_dir}/key.pub`

**Key Loading Workflow** (Phase 3):
```
Priority 1: Check VIBETEA_PRIVATE_KEY env var
  - If set and valid: Use it
  - If set but invalid: Error (no fallback)
Priority 2: Load from {VIBETEA_KEY_PATH}/key.priv
  - If exists and valid: Use it
  - If missing or invalid: Error
```

### Monitor → Server Authentication

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Ed25519 digital signatures | Rust `ed25519-dalek` crate |
| **Protocol** | HTTPS POST with signed payload | Event signatures in X-Signature header |
| **Key Management** | Source-specific public key registration | `VIBETEA_PUBLIC_KEYS` env var |
| **Key Format** | Base64-encoded Ed25519 public keys | `source1:pubkey1,source2:pubkey2` |
| **Verification** | Constant-time comparison using `subtle` crate | `server/src/auth.rs` |
| **Flow** | Monitor signs event → Server validates signature | `server/src/auth.rs`, `server/src/routes.rs` |
| **Fallback** | Unsafe no-auth mode (dev only) | `VIBETEA_UNSAFE_NO_AUTH=true` |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_PUBLIC_KEYS` (required unless unsafe mode enabled)
- Parses `VIBETEA_UNSAFE_NO_AUTH` (dev-only authentication bypass)
- Validates on every server startup with comprehensive error messages
- Supports multiple comma-separated source:key pairs

**Example Key Format**:
```
VIBETEA_PUBLIC_KEYS=monitor-prod:dGVzdHB1YmtleTEx,monitor-dev:dGVzdHB1YmtleTIy
```

**Implementation Details**:
- Uses `HashMap<String, String>` to map source_id to base64-encoded keys
- Public keys stored in plain text (no decryption needed)
- Empty public_keys map allowed if unsafe_no_auth is enabled
- Error handling with ConfigError enum for missing/invalid formats
- Constant-time comparison prevents timing attacks on signature verification

**Signature Verification Process** (`server/src/auth.rs`):
- Decode base64 signature from X-Signature header
- Decode base64 public key from configuration
- Extract Ed25519 VerifyingKey from public key bytes
- Use `ed25519_dalek::Signature::verify()` for verification
- Apply `subtle::ConstantTimeEq` to compare results

### Client Authentication (Server → Client)

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Bearer token in WebSocket headers | Static token per deployment |
| **Protocol** | WebSocket upgrade with `Authorization: Bearer <token>` | Client sends on connect |
| **Token Type** | Opaque string (no expiration in Phase 4) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| **Scope** | All clients use the same token | No per-user differentiation |
| **Validation** | Server-side validation only | In-memory, no persistence |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_SUBSCRIBER_TOKEN` (required unless unsafe mode enabled)
- Token required for all WebSocket connections
- No token refresh mechanism in Phase 5
- Stored as `Option<String>` in Config struct

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

## HTTP Sender & Event Transmission

### Phase 6: Event Sender Module

**Module Location**: `monitor/src/sender.rs` (544 lines)

**Sender Features**:

1. **HTTP Client Configuration**
   - Built with `reqwest` Client
   - Connection pooling: 10 max idle connections per host
   - Request timeout: 30 seconds
   - Automatic redirect handling

2. **Event Buffering**
   - VecDeque-based buffer with FIFO eviction
   - Default capacity: 1000 events
   - Configurable via `buffer_size` parameter
   - Tracks buffer overflow events with warnings
   - Supports queuing before sending

3. **Exponential Backoff Retry**
   - Initial delay: 1 second
   - Maximum delay: 60 seconds
   - Jitter: ±25% per attempt
   - Max retry attempts: 10 per batch
   - Resets on successful send

4. **Rate Limit Handling**
   - Recognizes HTTP 429 (Too Many Requests)
   - Reads `Retry-After` header from server
   - Respects server-provided delay
   - Falls back to exponential backoff if no header

5. **Event Signing**
   - Signs JSON event payload with Ed25519
   - X-Signature header contains base64-encoded signature
   - X-Source-ID header contains monitor source identifier
   - Compatible with server `auth.rs` verification

6. **Batch Sending**
   - `send_batch()` for efficient transmission
   - Single HTTP request with event array or single event
   - JSON request body with event(s)
   - 202 Accepted response expected

7. **Buffer Management**
   - `queue(event)` - Add to buffer
   - `flush()` - Send all buffered events
   - `send(event)` - Send single event immediately
   - `buffer_len()` - Current buffer size
   - `is_empty()` - Check if buffer empty

8. **Graceful Shutdown**
   - `shutdown(timeout)` - Flushes remaining events
   - Returns count of unflushed events
   - Waits for timeout before giving up
   - Allows time for final retry attempts

**SenderConfig**:
```rust
pub struct SenderConfig {
    pub server_url: String,     // e.g., https://vibetea.fly.dev
    pub source_id: String,      // e.g., hostname
    pub buffer_size: usize,     // e.g., 1000
}
```

**SenderError Types**:
- `Http` - HTTP client error (network, TLS, etc.)
- `ServerError { status, message }` - Non-202 response
- `AuthFailed` - 401 Unauthorized (invalid signature)
- `RateLimited { retry_after_secs }` - 429 with delay
- `BufferOverflow { evicted_count }` - Events evicted
- `MaxRetriesExceeded { attempts }` - All retries failed
- `Json` - Event serialization error

**Connection Details**:
- Server URL from `VIBETEA_SERVER_URL` env var
- POST to `{server_url}/events` endpoint
- HTTPS recommended for production
- HTTP allowed for local development

## GitHub Actions Integration

### Phase 5: Workflow File

**Location**: `.github/workflows/ci-with-monitor.yml` (114 lines)

### Phase 6: Composite Action File

**Location**: `.github/actions/vibetea-monitor/action.yml` (167 lines)

### Features

**Monitor Binary Download**:
- Fetches pre-built monitor from GitHub releases
- Target: x86_64-unknown-linux-gnu (Linux x86_64)
- URL pattern: `https://github.com/aaronbassett/VibeTea/releases/latest/download/vibetea-monitor-x86_64-unknown-linux-gnu`
- Graceful fallback: Continues if download fails (with warning)
- Exit code validation: Checks for successful execution
- Version control: Supports pinning specific versions

**Background Execution**:
- Starts monitor daemon: `./vibetea-monitor run &`
- Executes before main CI jobs (formatting, linting, tests, builds)
- Captures PID: `MONITOR_PID=$!` for later termination
- Non-blocking: Doesn't halt workflow on start failures

**Environment Setup**:
- **VIBETEA_PRIVATE_KEY**: From GitHub Actions secret
  - Base64-encoded 32-byte Ed25519 seed
  - Generated via `vibetea-monitor export-key`
  - Securely stored in repository secrets
- **VIBETEA_SERVER_URL**: From GitHub Actions secret
  - Server endpoint (e.g., `https://vibetea.fly.dev`)
  - Must be running and accessible
- **VIBETEA_SOURCE_ID**: Custom format for traceability
  - Format: `github-{owner}/{repo}-{run_id}`
  - Example: `github-aaronbassett/VibeTea-12345678`
  - Enables filtering events by workflow run in dashboards

**Graceful Shutdown**:
- Condition: `if: always()` (runs even if previous steps fail)
- Signal: `kill -TERM $MONITOR_PID`
- Grace period: 2-second flush window (`sleep 2`)
- Flushes buffered events before termination
- Prevents event loss on workflow completion

**Non-Blocking Behavior**:
- Network failures: Don't fail workflow
- Monitor startup failures: Don't fail workflow
- HTTP errors: Monitor retries with exponential backoff
- Rate limiting: Monitor respects Retry-After header

**CI Integration**:
- Runs alongside standard Rust/TypeScript checks
- Monitors active during: formatting, linting, tests, builds
- Events captured: All Claude Code activity during CI
- Example use cases: Track code generation, tool usage, agent decisions

**Binary Caching**:
- Uses GitHub Actions cache for cargo registry and dependencies
- Cache keys: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- Fallback keys: `${{ runner.os }}-cargo-`
- Reduces build times on subsequent runs

### Composite Action Inputs

- `server-url` (required): VibeTea server URL
- `private-key` (required): Base64-encoded Ed25519 private key
- `source-id` (optional): Custom source identifier (defaults to `github-<repo>-<run_id>`)
- `version` (optional): Monitor version to download (default: `latest`)
- `shutdown-timeout` (optional): Seconds to wait for graceful shutdown (default: `5`)

### Composite Action Outputs

- `monitor-pid`: Process ID of running monitor
- `monitor-started`: Boolean indicating successful startup

### Configuration

**Required Secrets** (set in repository settings):

1. **VIBETEA_PRIVATE_KEY**
   ```bash
   # Generate on local machine
   vibetea-monitor init         # If needed
   vibetea-monitor export-key   # Outputs base64-encoded key

   # Store in GitHub:
   # Settings → Secrets and variables → Actions → New repository secret
   # Name: VIBETEA_PRIVATE_KEY
   # Value: <paste output from export-key>
   ```

2. **VIBETEA_SERVER_URL**
   ```bash
   # Set to your running VibeTea server
   # Examples:
   # - https://vibetea.fly.dev
   # - https://your-domain.example.com
   # - http://localhost:3000 (not recommended for public workflows)
   ```

**Monitor Configuration**:
- Public key registration: Server must have corresponding public key registered
  - Formula: Export public key from local machine (`cat ~/.vibetea/key.pub`)
  - Register with server: `VIBETEA_PUBLIC_KEYS="github-{owner}/{repo}:{public_key}"`
  - Pattern: Source ID prefix matches monitor source ID in workflow

**Source ID Tracking**:
- Custom format enables filtering events by workflow run
- Dashboard can filter: `WHERE source = "github-aaronbassett/VibeTea-12345678"`
- Useful for correlating events with specific CI runs
- Prevents mixing events from multiple workflows

### Workflow Steps

1. **Checkout code**
   - Action: `actions/checkout@v4`
   - Prepares repository for CI jobs

2. **Download monitor binary**
   - Downloads pre-built monitor from releases
   - Sets execute permission if successful
   - Logs warning if download fails (graceful degradation)

3. **Start VibeTea Monitor**
   - Checks for binary and secrets
   - Starts background daemon
   - Captures PID for later shutdown
   - Logs source ID for tracking

4. **Install Rust toolchain**
   - Action: `dtolnay/rust-toolchain@stable`
   - Includes rustfmt and clippy

5. **Setup cache**
   - Caches cargo registry, git, and build artifacts
   - Improves subsequent build times

6. **Check formatting**
   - Command: `cargo fmt --all -- --check`
   - Validates code formatting

7. **Run clippy**
   - Command: `cargo clippy --all-targets -- -D warnings`
   - Lints code with clippy

8. **Run tests**
   - Command: `cargo test --workspace -- --test-threads=1`
   - Executes all tests sequentially (for env var isolation)

9. **Build release**
   - Command: `cargo build --workspace --release`
   - Produces optimized binaries

10. **Stop VibeTea Monitor**
    - Sends SIGTERM to background process
    - Allows 2-second grace period for event flushing
    - Runs even if previous steps failed

### Event Tracking During CI

**What Gets Captured**:
- Claude Code session events during workflow execution
- Tool usage: Read, Write, Grep, Bash, etc.
- Activity events: User interactions
- Agent state changes
- Session start/end markers

**What Doesn't Get Sent**:
- Code content (privacy filtered)
- Prompts or responses (privacy filtered)
- Full file paths (reduced to basenames)
- Sensitive command arguments (stripped)

**Dashboard Integration**:
- Filter by source: `github-{owner}/{repo}-{run_id}`
- Correlate with GitHub Actions run
- Track tool usage across CI jobs
- Analyze patterns in automated sessions

### Example Workflow Configuration

```yaml
# In GitHub repository settings:
# Secrets → VIBETEA_PRIVATE_KEY
# Value: base64-encoded key from vibetea-monitor export-key

# Secrets → VIBETEA_SERVER_URL
# Value: https://your-vibetea-server.example.com

# On server, register public key:
# export VIBETEA_PUBLIC_KEYS="github-owner/repo:$(cat ~/.vibetea/key.pub)"
```

## CLI & Key Management

### Phase 6: Monitor CLI

**Module Location**: `monitor/src/main.rs` (301 lines, expanded to 566 lines in Phase 4)

**Command Structure**:

1. **init Command**: Generate Ed25519 keypair
   ```bash
   vibetea-monitor init [--force]
   ```
   - Generates new keypair using `Crypto::generate()`
   - Saves to `~/.vibetea/` or `VIBETEA_KEY_PATH`
   - Displays public key for server registration
   - Prompts for overwrite confirmation (unless --force)
   - Provides copy-paste ready export command

2. **run Command**: Start monitor daemon
   ```bash
   vibetea-monitor run
   ```
   - Loads configuration from environment variables
   - Loads cryptographic keys from disk or env var (Phase 3)
   - Creates sender with buffering and retry
   - Initializes file watcher (future: Phase 7)
   - Waits for shutdown signal
   - Graceful shutdown with event flushing

3. **export-key Command**: Export private key (Phase 4)
   ```bash
   vibetea-monitor export-key [--path <PATH>]
   ```
   - Loads private key from disk
   - Outputs base64-encoded seed to stdout (+ single newline)
   - All diagnostics to stderr
   - Exit code 0 on success, 1 on error
   - Suitable for piping to clipboard or secret management tools

4. **help Command**: Show documentation
   ```bash
   vibetea-monitor help
   vibetea-monitor --help
   vibetea-monitor -h
   ```
   - Displays usage information
   - Lists all available commands
   - Shows environment variables
   - Provides example commands

5. **version Command**: Show version
   ```bash
   vibetea-monitor version
   vibetea-monitor --version
   vibetea-monitor -V
   ```
   - Prints binary version from Cargo.toml

**CLI Framework** (Phase 4):
- Uses `clap` crate with Subcommand and Parser derive macros
- Type-safe command parsing with automatic help generation
- Replaces manual argument parsing from Phase 6
- Command enum variants: Init, ExportKey, Run
- Flag support: `--force/-f` for init, `--path` for export-key

**Environment Variables Used**:

| Variable | Required | Default | Command |
|----------|----------|---------|---------|
| `VIBETEA_SERVER_URL` | Yes | - | run |
| `VIBETEA_SOURCE_ID` | No | hostname | run |
| `VIBETEA_PRIVATE_KEY` | No* | - | run (Phase 3 - loads from env) |
| `VIBETEA_KEY_PATH` | No | ~/.vibetea | init, run, export-key |
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | run |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | run |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | run |
| `RUST_LOG` | No | info | all |

*Either VIBETEA_PRIVATE_KEY (env) or VIBETEA_KEY_PATH/key.priv (file) required

**Logging**:
- Structured logging via `tracing` crate
- Environment-based filtering (`RUST_LOG`)
- JSON output support
- Logs configuration, key loading, shutdown events
- Info level by default

**Signal Handling**:
- Listens for SIGINT (Ctrl+C)
- Listens for SIGTERM on Unix
- Cross-platform support via `tokio::signal`
- Graceful shutdown sequence on signal

**Key Registration Workflow**:
1. User runs: `vibetea-monitor init`
2. Binary displays public key
3. User copies to: `export VIBETEA_PUBLIC_KEYS="...:<public_key>"`
4. User adds to server configuration
5. User runs: `vibetea-monitor run`

**Phase 3 Key Loading Workflow**:
```bash
# Option 1: Use environment variable (new in Phase 3)
export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)
vibetea-monitor run

# Option 2: Use file (Phase 2)
vibetea-monitor init
vibetea-monitor run

# Option 3: Fallback behavior (both checked in order)
export VIBETEA_PRIVATE_KEY=...  # Checked first
# If not set, falls back to ~/.vibetea/key.priv
vibetea-monitor run
```

**Phase 4 GitHub Actions Workflow**:
```bash
# Export key from development machine
exported_key=$(vibetea-monitor export-key)

# Register in GitHub Actions secret
gh secret set VIBETEA_PRIVATE_KEY --body "$exported_key"

# Use in workflow
- name: Export monitor key
  env:
    VIBETEA_PRIVATE_KEY: ${{ secrets.VIBETEA_PRIVATE_KEY }}
  run: vibetea-monitor run
```

## Client-Side Integrations (Phase 7-10)

### Browser WebSocket Connection

**Module Location**: `client/src/hooks/useWebSocket.ts` (321 lines)

**WebSocket Hook Features**:

1. **Connection Management**
   - Establishes WebSocket connection to server
   - Validates token from localStorage before connecting
   - Tracks connection state: connecting, connected, reconnecting, disconnected
   - Provides manual `connect()` and `disconnect()` methods

2. **Auto-Reconnection**
   - Exponential backoff: 1s initial, 60s maximum
   - Jitter: ±25% randomization per attempt
   - Resets attempt counter on successful connection
   - Respects user's disconnect intent (no auto-reconnect after manual disconnect)

3. **Token Management**
   - Reads token from `localStorage` key: `vibetea_token`
   - Token set via TokenForm component
   - Returns error if token missing, prevents connection
   - Token passed as query parameter in WebSocket URL

4. **Event Processing**
   - Receives JSON-encoded VibeteaEvent messages
   - Validates message structure (id, source, timestamp, type, payload)
   - Dispatches valid events to Zustand store via `addEvent()`
   - Silently discards invalid/unparseable messages

5. **Integration with Event Store**
   - `useEventStore` for state management
   - `addEvent(event)` - Add event to store
   - `setStatus(status)` - Update connection status
   - Status field synced with component state

6. **Error Handling**
   - Logs connection errors to console
   - Logs message parsing failures
   - Graceful handling of malformed messages
   - No crashes on connection errors

7. **Cleanup & Lifecycle**
   - Proper cleanup on component unmount
   - Clears pending reconnection timeouts
   - Closes WebSocket connection
   - Prevents memory leaks

**Hook Return Type**:
```typescript
export interface UseWebSocketReturn {
  readonly connect: () => void;          // Manually initiate connection
  readonly disconnect: () => void;        // Manually disconnect
  readonly isConnected: boolean;          // Connection state
}
```

**Constants**:
- `TOKEN_STORAGE_KEY`: `"vibetea_token"` (matches TokenForm)
- `INITIAL_BACKOFF_MS`: 1000ms
- `MAX_BACKOFF_MS`: 60000ms
- `JITTER_FACTOR`: 0.25 (25%)

**Default WebSocket URL**:
- Protocol: `ws://` (HTTP) or `wss://` (HTTPS) based on location protocol
- Host: Current browser location host
- Path: `/ws`
- Query param: `token=<token_from_localStorage>`

### Connection Status Component

**Module Location**: `client/src/components/ConnectionStatus.tsx` (106 lines)

**Features**:

1. **Visual Indicator**
   - Colored dot (2.5x2.5 rem) showing connection state
   - Green (#22c55e) for connected
   - Yellow (#eab308) for connecting/reconnecting
   - Red (#ef4444) for disconnected
   - Uses Tailwind CSS classes

2. **Optional Status Label**
   - Shows text status if `showLabel` prop is true
   - Labels: "Connected", "Connecting", "Reconnecting", "Disconnected"
   - Styled as small gray text
   - Dark mode support

3. **Performance Optimization**
   - Selective Zustand subscription: only re-renders when status changes
   - Uses selector to extract only status field
   - Prevents re-renders on other store updates

4. **Accessibility**
   - `role="status"` for semantic meaning
   - `aria-label` with full status description
   - Visual indicator marked as `aria-hidden="true"`
   - Screen reader friendly

5. **Component Props**:
```typescript
interface ConnectionStatusProps {
  readonly showLabel?: boolean;    // Show status text (default: false)
  readonly className?: string;     // Additional CSS classes
}
```

6. **Styling**
   - Flexbox layout with gap-2
   - Responsive and composable
   - Integrates seamlessly with other UI elements
   - Dark mode aware styling

### Token Form Component

**Module Location**: `client/src/components/TokenForm.tsx` (201 lines)

**Features**:

1. **Token Input & Storage**
   - Password input field for secure token entry
   - Persists token to `localStorage` via `TOKEN_STORAGE_KEY`
   - Matches key used by `useWebSocket` hook
   - Non-empty validation before saving

2. **Button Controls**
   - **Save Token** button
     - Disabled when input is empty
     - Saves trimmed token to localStorage
     - Resets input field after save
     - Invokes optional callback
   - **Clear Token** button
     - Disabled when no token saved
     - Removes token from localStorage
     - Resets input and status
     - Invokes optional callback

3. **Status Indicator**
   - Green dot when token saved
   - Gray dot when no token saved
   - Text shows "Token saved" or "No token saved"
   - Updates in real-time as user changes

4. **Cross-Window Synchronization**
   - Listens to `storage` events
   - Detects token changes from other tabs/windows
   - Updates status accordingly
   - Handles multi-tab scenarios

5. **Component Props**:
```typescript
interface TokenFormProps {
  readonly onTokenChange?: () => void;  // Called when token saved/cleared
  readonly className?: string;          // Additional CSS classes
}
```

6. **Callback Support**
   - `onTokenChange()` invoked on save or clear
   - Allows parent to reconnect WebSocket
   - Enables form submission handlers

7. **Accessibility**
   - Label element linked to input
   - `aria-describedby` for status association
   - Status region with `aria-live="polite"`
   - Semantic form structure
   - Proper button states for disabled

8. **Styling**
   - Tailwind CSS dark mode (bg-gray-800, text-white)
   - Responsive layout
   - Visual feedback on focus (blue ring)
   - Disabled state styling (gray background, cursor not-allowed)
   - Button hover effects

9. **Behavior**
   - Stores token under key `vibetea_token` (matches useWebSocket)
   - Input placeholder changes based on save state
   - Form submission on button click or Enter key
   - Input cleared after successful save
   - Token masked as password field

## Network Communication

### Monitor → Server (Event Publishing)

**Endpoint**: `https://<server-url>/events`
**Method**: POST
**Authentication**: Ed25519 signature in X-Signature header (Phase 6)
**Content-Type**: application/json

**Flow**:
1. Monitor watches local JSONL files via file watcher
2. Parser extracts metadata from new/modified JSONL lines
3. Events processed through PrivacyPipeline (Phase 5)
4. Monitor signs event payload with Ed25519 private key (Phase 6)
5. Monitor POSTs signed event to server with X-Source-ID and X-Signature headers (Phase 6)
6. Server validates signature against registered public key
7. Server rate limits based on source ID (100 events/sec default)
8. Server broadcasts to all connected clients via WebSocket

**Rate Limiting** (`server/src/rate_limit.rs`):
- Token bucket algorithm per source
- 100.0 tokens/second refill rate (configurable)
- 100 token capacity (configurable)
- Exceeded limit returns 429 Too Many Requests with Retry-After header
- Automatic cleanup of inactive sources after 60 seconds

**Client Library**: `reqwest` crate (HTTP client)
**Configuration**: `monitor/src/config.rs`
- `VIBETEA_SERVER_URL` - Server endpoint (required)
- `VIBETEA_SOURCE_ID` - Source identifier for event attribution (default: hostname)
- Uses gethostname crate to get system hostname if not provided

**Phase 6 Enhancements**:
- Crypto module signs all events before transmission
- Sender module handles buffering, retry, rate limiting
- CLI allows easy key management and monitor startup

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request headers
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket connection with Bearer token
2. Server validates token and establishes connection
3. Server broadcasts events as they arrive from monitors
4. Optional: Server filters events based on query parameters (source, type, project)
5. Client processes and stores events in Zustand state via `addEvent()`
6. Client UI renders session information from state

**Server Broadcasting** (`server/src/broadcast.rs`):
- EventBroadcaster wraps tokio broadcast channel
- 1000-event capacity for burst handling
- Thread-safe, cloneable for sharing across handlers
- SubscriberFilter enables selective delivery by event type, source, project

**WebSocket Upgrade** (`server/src/routes.rs`):
- GET /ws endpoint handles upgrade requests
- Validates bearer token before upgrade
- Returns 101 Switching Protocols on success
- Returns 401 Unauthorized on token validation failure

**Client-Side Handling** (Phase 7-10):
- WebSocket proxy configured in `client/vite.config.ts` (target: ws://localhost:8080)
- State management via `useEventStore` hook (Zustand)
- Event type guards for safe type access in `client/src/types/events.ts`
- ConnectionStatus transitions: disconnected → connecting → connected → reconnecting
- Token management via `TokenForm` component
- Connection control via `useWebSocket` hook
- Virtual scrolling display via EventStream component (Phase 8)
- Session management via SessionOverview component (Phase 10)

**Connection Details**:
- Address/port: Configured via `PORT` environment variable (default: 8080)
- Persistent connection model
- Automatic reconnection with exponential backoff (Phase 7)
- No message queuing (direct streaming)
- Events processed with selective subscriptions to prevent unnecessary re-renders

### Monitor → File System (JSONL Watching)

**Target**: `~/.claude/projects/**/*.jsonl`
**Mechanism**: `notify` crate file system events (inotify/FSEvents)
**Update Strategy**: Incremental line reading with position tracking

**Flow**:
1. FileWatcher initialized with watch directory
2. Recursive file system monitoring begins
3. File creation detected → WatchEvent::FileCreated emitted
4. File modification detected → New lines read from position marker
5. Lines sent in WatchEvent::LinesAdded with accumulated lines
6. Position marker updated to avoid re-reading
7. File deletion detected → WatchEvent::FileRemoved emitted, cleanup position state

**Efficiency Features**:
- Position tracking prevents re-reading file content
- Only new lines since last position are extracted
- BufReader with Seek for efficient line iteration
- Arc<RwLock<>> for thread-safe concurrent access

## HTTP API Endpoints

### POST /events

**Purpose**: Ingest events from monitors

**Request Headers**:
| Header | Required | Value |
|--------|----------|-------|
| X-Source-ID | Yes | Monitor identifier (non-empty string) |
| X-Signature | No* | Base64-encoded Ed25519 signature (Phase 6) |
| Content-Type | Yes | application/json |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**Request Body**: Single Event or array of Events (JSON)

**Response Codes**:
- 202 Accepted - Events accepted and broadcasted
- 400 Bad Request - Invalid JSON or malformed events
- 401 Unauthorized - Missing/empty X-Source-ID or signature verification failed
- 429 Too Many Requests - Rate limit exceeded (includes Retry-After header)

**Flow** (`server/src/routes.rs`):
1. Extract X-Source-ID from headers
2. Check rate limit for source
3. If unsafe_no_auth is false, verify X-Signature against public key
4. Deserialize event(s) from body
5. Broadcast each event via EventBroadcaster
6. Return 202 Accepted

### GET /ws

**Purpose**: WebSocket subscription for event streaming

**Query Parameters**:
| Parameter | Required | Example |
|-----------|----------|---------|
| token | No* | my-secret-token |
| source | No | monitor-1 |
| type | No | session |
| project | No | my-project |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**WebSocket Messages**: JSON-encoded Event objects (one per message)

**Response Codes**:
- 101 Switching Protocols - WebSocket upgrade successful
- 401 Unauthorized - Token validation failed

**Filtering** (`server/src/broadcast.rs`):
- SubscriberFilter applied if query parameters provided
- Matches event.event_type against type parameter
- Matches event.source against source parameter
- Matches event.payload.project against project parameter

### GET /health

**Purpose**: Health check and uptime reporting

**Response**:
```json
{
  "status": "ok",
  "uptime_secs": 3600
}
```

**Response Code**: 200 OK (always succeeds, no auth required)

## Development & Local Configuration

### Local Server Setup

**Environment Variables**:
```bash
PORT=8080                                        # Server port
VIBETEA_PUBLIC_KEYS=localhost:cHVia2V5MQ==      # Monitor public key (base64)
VIBETEA_SUBSCRIBER_TOKEN=dev-token-secret        # Client WebSocket token
VIBETEA_UNSAFE_NO_AUTH=false                     # Set true to disable all auth
RUST_LOG=debug                                   # Logging level
```

**Unsafe Development Mode**:
When `VIBETEA_UNSAFE_NO_AUTH=true`:
- All monitor authentication is bypassed (X-Signature ignored)
- All client authentication is bypassed (token parameter ignored)
- Suitable for local development only
- Never use in production
- Warning logged on startup when enabled

**Validation Behavior**:
- With unsafe_no_auth=false: Requires both VIBETEA_PUBLIC_KEYS and VIBETEA_SUBSCRIBER_TOKEN
- With unsafe_no_auth=true: Both auth variables become optional
- PORT defaults to 8080 if not specified
- Invalid PORT formats rejected with ParseIntError

### Local Monitor Setup

**Environment Variables**:
```bash
VIBETEA_SERVER_URL=http://localhost:8080         # Server endpoint
VIBETEA_SOURCE_ID=my-monitor                     # Custom source identifier
VIBETEA_KEY_PATH=~/.vibetea                      # Directory with private/public keys
VIBETEA_PRIVATE_KEY=<base64-seed>                # Env var key loading (Phase 3)
VIBETEA_CLAUDE_DIR=~/.claude                     # Claude Code directory to watch
VIBETEA_BUFFER_SIZE=1000                         # Event buffer capacity
VIBETEA_BASENAME_ALLOWLIST=.ts,.tsx,.rs          # Optional file extension filter (Phase 5)
RUST_LOG=debug                                   # Logging level
```

**Configuration Loading**: `monitor/src/config.rs`
- Required: VIBETEA_SERVER_URL (no default)
- Optional defaults use directories crate for platform-specific paths
- Home directory determined via BaseDirs::new()
- Hostname fallback when VIBETEA_SOURCE_ID not set
- Buffer size parsed as usize, validated for positive integers
- Allowlist split by comma, whitespace trimmed, empty entries filtered

**Key Management** (Phase 3):
- `vibetea-monitor init` generates Ed25519 keypair
- `vibetea-monitor export-key` exports private key as base64 (Phase 4 feature)
- Keys stored in ~/.vibetea/ or VIBETEA_KEY_PATH
- Private key: key.priv (0600 permissions)
- Public key: key.pub (0644 permissions)
- Public key must be registered with server via VIBETEA_PUBLIC_KEYS
- Private key can be loaded from VIBETEA_PRIVATE_KEY env var (Phase 3)

**Privacy Configuration** (Phase 5):
- `VIBETEA_BASENAME_ALLOWLIST` loads into PrivacyConfig via `from_env()`
- Format: `.rs,.ts,.md` or `rs,ts,md` (dots auto-added)
- Whitespace tolerance: ` .rs , .ts ` → `[".rs", ".ts"]`
- Empty entries filtered: `.rs,,.ts,,,` → `[".rs", ".ts"]`
- When not set: All extensions allowed (default behavior)
- Applied during PrivacyPipeline event processing

**File System Monitoring**:
- Watches directory: VIBETEA_CLAUDE_DIR
- Monitors for file creation, modification, deletion, and directory changes
- Uses `notify` crate (version 8.0) for cross-platform inotify/FSEvents
- Optional extension filtering via VIBETEA_BASENAME_ALLOWLIST
- Phase 4: FileWatcher tracks position to efficiently tail JSONL files

**JSONL Parsing**:
- Phase 4: SessionParser extracts metadata from Claude Code JSONL
- Privacy-first: Never processes code content or prompts
- Tool tracking: Extracts tool name and context from assistant tool_use events
- Progress tracking: Detects tool completion from progress PostToolUse events

**Privacy Pipeline** (Phase 5):
- PrivacyPipeline processes all events before transmission
- PrivacyConfig loaded from `VIBETEA_BASENAME_ALLOWLIST`
- Sensitive tools stripped: Bash, Grep, Glob, WebSearch, WebFetch
- Paths reduced to basenames with extension allowlist filtering
- Summary text neutralized to "Session ended"

**Cryptographic Signing** (Phase 6):
- Crypto module signs all events with Ed25519 private key
- Signature sent in X-Signature header (base64-encoded)
- Monitor must be initialized before first run: `vibetea-monitor init`

**HTTP Transmission** (Phase 6):
- Sender module handles event buffering (1000 events default)
- Exponential backoff retry: 1s → 60s with ±25% jitter
- Rate limit handling: Respects 429 with Retry-After header
- Connection pooling: 10 max idle connections per host
- 30-second request timeout

### GitHub Actions Setup (Phase 5)

**Prerequisites**:
1. A running VibeTea server with your public key registered
2. An existing keypair on your local machine (run `vibetea-monitor init` if needed)

**Step 1: Export Your Private Key**
```bash
# Export to clipboard (macOS)
vibetea-monitor export-key | pbcopy

# Export to stdout (Linux/Windows)
vibetea-monitor export-key
```

**Step 2: Register GitHub Actions Secret**
```bash
# Using GitHub CLI
gh secret set VIBETEA_PRIVATE_KEY --body "$(vibetea-monitor export-key)"

# Or manually in GitHub web interface:
# Settings → Secrets and variables → Actions → New repository secret
# Name: VIBETEA_PRIVATE_KEY
# Value: <paste output from export-key command>
```

**Step 3: Register Server URL Secret**
```bash
# Using GitHub CLI
gh secret set VIBETEA_SERVER_URL --body "https://your-vibetea-server.example.com"

# Or manually in GitHub web interface:
# Settings → Secrets and variables → Actions → New repository secret
# Name: VIBETEA_SERVER_URL
# Value: https://your-vibetea-server.example.com
```

**Step 4: Register Public Key with Server**
```bash
# Export public key from local machine
cat ~/.vibetea/key.pub

# On server, register with source ID pattern:
export VIBETEA_PUBLIC_KEYS="github-{owner}/{repo}:$(cat ~/.vibetea/key.pub)"

# Example:
export VIBETEA_PUBLIC_KEYS="github-aaronbassett/VibeTea:dGVzdHB1YmtleTExYWJjZGVmZ2hpams="
```

**Step 5: Add Workflow File**
- Copy `.github/workflows/ci-with-monitor.yml` example
- Customize for your repository and CI needs
- Commit and push to main branch
- Workflow will run on next push/PR

## Configuration Quick Reference

### Server Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `PORT` | number | 8080 | No | HTTP server listening port |
| `VIBETEA_PUBLIC_KEYS` | string | - | Yes* | Source public keys (source:key,source:key) |
| `VIBETEA_SUBSCRIBER_TOKEN` | string | - | Yes* | Bearer token for clients |
| `VIBETEA_UNSAFE_NO_AUTH` | boolean | false | No | Disable all authentication (dev only) |
| `RUST_LOG` | string | info | No | Logging level (debug, info, warn, error) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor Environment Variables

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | string | - | Yes | Server endpoint (e.g., https://vibetea.fly.dev) |
| `VIBETEA_SOURCE_ID` | string | hostname | No | Monitor identifier |
| `VIBETEA_PRIVATE_KEY` | string | - | No* | Base64-encoded private key (Phase 3) |
| `VIBETEA_KEY_PATH` | string | ~/.vibetea | No | Directory with key.priv/key.pub |
| `VIBETEA_CLAUDE_DIR` | string | ~/.claude | No | Claude Code directory to watch |
| `VIBETEA_BUFFER_SIZE` | number | 1000 | No | Event buffer capacity |
| `VIBETEA_BASENAME_ALLOWLIST` | string | - | No | Comma-separated file extensions to watch (Phase 5) |
| `RUST_LOG` | string | info | No | Logging level (debug, info, warn, error) |

*Either VIBETEA_PRIVATE_KEY (env) or VIBETEA_KEY_PATH/key.priv (file) required

### Client localStorage Keys (Phase 7)

| Key | Purpose | Format |
|-----|---------|--------|
| `vibetea_token` | WebSocket authentication token | String |

## Future Integration Points

### Planned (Not Yet Integrated)

- **Main event loop**: Integrate file watcher, parser, privacy pipeline, and HTTP sender (Phase 6 in progress)
- **Database/Persistence**: Store events beyond memory (Phase 5+)
- **Authentication Providers**: OAuth2, API key rotation (Phase 5+)
- **Monitoring Services**: Datadog, New Relic, CloudWatch (Phase 5+)
- **Message Queues**: Redis, RabbitMQ for event buffering (Phase 5+)
- **Webhooks**: External service notifications (Phase 6+)
- **Background Task Spawning**: Async watcher and sender pipeline (Phase 6+)
- **Session Persistence**: Store events in database for replay (Phase 7+)
- **Advanced Authentication**: Per-user tokens, OAuth2 flows (Phase 7+)
- **Event Search/Filtering**: Full-text search and advanced filtering UI (Phase 7+)
- **Performance Monitoring**: Client-side performance metrics (Phase 8+)
