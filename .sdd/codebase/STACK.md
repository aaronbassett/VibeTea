# Technology Stack

**Status**: Phase 6 - GitHub Actions composite action for monitor integration
**Last Updated**: 2026-02-04

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, JSONL parsing, privacy filtering, event signing, HTTP transmission, CLI with export-key |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization |
| GitHub Actions | YAML/Bash | - | Composite action for monitor integration and workflow orchestration |

## Frameworks & Runtime Libraries

### Rust (Monitor & Server)

| Package            | Version | Purpose | Used By |
|--------------------|---------|---------|----------|
| tokio              | 1.43    | Async runtime with full features | Server, Monitor |
| axum               | 0.8     | HTTP/WebSocket server framework | Server |
| tower              | 0.5     | Composable middleware | Server |
| tower-http         | 0.6     | HTTP utilities (CORS, tracing) | Server |
| reqwest            | 0.12    | HTTP client library with connection pooling | Monitor, Server (tests) |
| serde              | 1.0     | Serialization/deserialization | All |
| serde_json         | 1.0     | JSON serialization | All |
| ed25519-dalek      | 2.1     | Ed25519 cryptographic signing | Server, Monitor |
| uuid               | 1.11    | Unique identifiers for events | Server, Monitor |
| chrono             | 0.4     | Timestamp handling | Server, Monitor |
| thiserror          | 2.0     | Error type derivation | Server, Monitor |
| anyhow             | 1.0     | Flexible error handling | Server, Monitor |
| tracing            | 0.1     | Structured logging framework | Server, Monitor |
| tracing-subscriber | 0.3     | Logging implementation (JSON, env-filter) | Server, Monitor |
| notify             | 8.0     | File system watching | Monitor |
| base64             | 0.22    | Base64 encoding/decoding | Server, Monitor |
| rand               | 0.9     | Random number generation | Server, Monitor |
| directories        | 6.0     | Standard directory paths | Monitor |
| gethostname        | 1.0     | System hostname retrieval | Monitor |
| subtle             | 2.6     | Constant-time comparison for cryptography | Server (auth) |
| zeroize            | 1.8     | Secure memory wiping for cryptographic material | Monitor (Phase 3) |
| futures-util       | 0.3     | WebSocket stream utilities | Server |
| futures            | 0.3     | Futures trait and utilities | Monitor (async coordination) |
| clap               | 4.5     | CLI argument parsing with derive macros | Monitor (clap Subcommand/Parser for export-key, Phase 4) |

### TypeScript/JavaScript (Client)

| Package                    | Version  | Purpose |
|---------------------------|----------|---------|
| React                      | ^19.2.4  | UI framework |
| React DOM                  | ^19.2.4  | DOM rendering |
| TypeScript                 | ^5.9.3   | Language and type checking |
| Vite                       | ^7.3.1   | Build tool and dev server |
| Tailwind CSS               | ^4.1.18  | Utility-first CSS framework |
| Zustand                    | ^5.0.11  | Lightweight state management |
| @tanstack/react-virtual    | ^3.13.18 | Virtual scrolling for large lists (Phase 8) |
| @vitejs/plugin-react       | ^5.1.3   | React Fast Refresh for Vite |
| @tailwindcss/vite          | ^4.1.18  | Tailwind CSS Vite plugin |
| vite-plugin-compression2   | ^2.4.0   | Brotli compression for builds |

## Build Tools & Package Managers

| Tool     | Version  | Purpose |
|----------|----------|---------|
| cargo    | -        | Rust package manager and build system |
| pnpm     | -        | Node.js package manager (client) |
| rustfmt  | -        | Rust code formatter |
| clippy   | -        | Rust linter |
| prettier | ^3.8.1   | Code formatter (TypeScript) |
| ESLint   | ^9.39.2  | JavaScript/TypeScript linter |

## Development & Testing

### Rust Testing
| Package      | Version | Purpose |
|--------------|---------|---------|
| tokio-test   | 0.4     | Tokio testing utilities |
| tempfile     | 3.15    | Temporary file/directory management for tests |
| serial_test  | 3.2     | Serial test execution for environment variable tests |
| wiremock     | 0.6     | HTTP mocking for integration tests |

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit/component testing framework |
| @testing-library/react | ^16.3.2  | React testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM matchers for testing |
| jsdom                  | ^28.0.0  | DOM implementation for Node.js |

## Configuration Files

| File | Framework | Purpose |
|------|-----------|---------|
| `client/vite.config.ts` | Vite | Build configuration, WebSocket proxy to server on port 8080 |
| `client/tsconfig.json` | TypeScript | Strict mode, ES2020 target |
| `client/eslint.config.js` | ESLint | Flat config format with TypeScript support |
| `Cargo.toml` (workspace) | Cargo | Rust workspace configuration and shared dependencies |
| `server/Cargo.toml` | Cargo | Server package configuration |
| `monitor/Cargo.toml` | Cargo | Monitor package configuration |
| `.github/actions/vibetea-monitor/action.yml` | GitHub Actions | Composite action for monitor deployment (Phase 6) |

## Runtime Environment

| Aspect | Details |
|--------|---------|
| Server Runtime | Rust binary (tokio async) |
| Client Runtime | Browser (ES2020+) |
| Monitor Runtime | Native binary (Linux/macOS/Windows) with CLI |
| Node.js | Required for development and client build only |
| Async Model | Tokio (Rust), Promises (TypeScript) |
| WebSocket Support | Native (server-side via axum, client-side via browser) |
| WebSocket Proxy | Vite dev server proxies /ws to localhost:8080 |
| File System Monitoring | Rust notify crate (inotify/FSEvents) for JSONL tracking |
| CLI Support | clap Subcommand enum for command parsing (init, run, export-key via clap derive macros, Phase 4) |
| CI/CD Integration | GitHub Actions workflow with monitor deployment and background execution (Phase 5), composite action wrapper (Phase 6) |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature with X-Signature header |
| Server → Client | WebSocket | JSON | Bearer token |
| Client → Server | WebSocket | JSON | Bearer token |
| Monitor → File System | Native | JSONL | N/A (local file access) |
| GitHub Actions | SSH/HTTPS | Binary download + env var injection | GitHub Actions secrets |

## Data Serialization

| Component | Serialization | Notes |
|-----------|---------------|-------|
| Server/Monitor | serde (Rust) | JSON with snake_case for env configs |
| Client | TypeScript/JSON | camelCase for API contracts |
| Events | serde_json | Standardized event schema across components |
| Claude Code Files | JSONL (JSON Lines) | Privacy-first parsing extracting only metadata |
| Cryptographic Keys | Base64 + Raw bytes | Public keys base64 encoded, private keys raw 32-byte seeds |

## Build Output

| Component | Output | Format | Deployment |
|-----------|--------|--------|-----------|
| Server | Binary | ELF (Linux) | Docker container on Fly.io |
| Monitor | Binary | ELF/Mach-O/PE | Standalone executable for users or GitHub Actions |
| Client | Static files | JS + CSS (Brotli compressed) | CDN (Netlify/Vercel/Cloudflare) |

## Module Organization

### Client (`client/src`)
- `components/` - React components
  - `ConnectionStatus.tsx` - **Phase 7**: Visual WebSocket connection status indicator
  - `TokenForm.tsx` - **Phase 7**: Token management and persistence UI
  - `EventStream.tsx` - **Phase 8**: Virtual scrolling event stream with 1000+ event support
  - `Heatmap.tsx` - **Phase 9**: Activity heatmap with CSS Grid, color scale, 7/30-day views, accessibility
  - `SessionOverview.tsx` - **Phase 10**: Session cards with activity indicators and status badges
- `hooks/useEventStore.ts` - Zustand store for WebSocket event state with session tracking and timeout management
- `hooks/useWebSocket.ts` - **Phase 7**: WebSocket connection management with auto-reconnect
- `hooks/useSessionTimeouts.ts` - **Phase 10**: Session timeout checking (5min active→inactive, 30min removal)
- `types/events.ts` - Event type definitions with discriminated union types matching Rust schema
- `utils/` - Utility functions
  - `formatting.ts` - **Phase 8**: Timestamp and duration formatting utilities (5 functions, 331 lines)
- `__tests__/` - Test files
  - `formatting.test.ts` - **Phase 8**: Comprehensive formatting utility tests (33 test cases)
- `App.tsx` - Root component
- `main.tsx` - Entry point
- `index.css` - Global styles

### Server (`server/src`)
- `config.rs` - Environment variable parsing and validation (public keys, subscriber token, port)
- `auth.rs` - Ed25519 signature verification and token validation with constant-time comparison
- `broadcast.rs` - Event broadcaster using tokio broadcast channels with subscriber filtering
- `rate_limit.rs` - Per-source token bucket rate limiting (100 events/sec default)
- `routes.rs` - HTTP endpoints (POST /events, GET /ws, GET /health)
- `error.rs` - Error types and handling
- `types.rs` - Event types and data models
- `lib.rs` - Public library interface
- `main.rs` - Server entry point

### Monitor (`monitor/src`)
- `config.rs` - Configuration from environment variables (server URL, source ID, key path, buffer size)
- `error.rs` - Error types
- `types.rs` - Event types
- `parser.rs` - Claude Code JSONL parser (privacy-first metadata extraction)
- `watcher.rs` - File system watcher for `.claude/projects/**/*.jsonl` files with position tracking
- `privacy.rs` - **Phase 5**: Privacy pipeline for event sanitization before transmission
- `crypto.rs` - **Phase 3-6**: Ed25519 keypair generation, loading, saving, and event signing with memory safety
- `sender.rs` - **Phase 6**: HTTP client with event buffering, exponential backoff retry, and rate limit handling
- `main.rs` - **Phase 4**: CLI entry point with init, run, and export-key commands (clap Subcommand enum)
- `lib.rs` - Public interface

### GitHub Actions (Phase 6)
- `.github/actions/vibetea-monitor/action.yml` - **Phase 6**: Composite action for monitor integration
  - Downloads monitor binary from releases
  - Configures environment variables
  - Starts monitor in background
  - Manages process lifecycle
  - Outputs monitor PID and status for downstream cleanup

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary, minimal base image |
| Client | CDN | Static files | Optimized builds with compression |
| Monitor | Local + GitHub Actions | Native binary + workflow | Users download locally or use in CI workflows (Phase 5) |

## GitHub Actions Integration (Phase 6)

### Composite Action File
**Location**: `.github/actions/vibetea-monitor/action.yml` (167 lines)

### Features
- **Composite action**: Simplified reusable workflow action for monitor integration
- **Binary download**: Fetches pre-built monitor from releases with version control
- **Background execution**: Starts monitor daemon with process tracking
- **Environment setup**: Accepts private key, server URL, and source ID as inputs
- **Process management**: Outputs monitor PID and status for lifecycle management
- **Graceful shutdown**: Documented pattern for cleanup step
- **Non-blocking**: Network failures don't fail workflows

### Action Inputs
- `server-url` (required): VibeTea server URL
- `private-key` (required): Base64-encoded Ed25519 private key
- `source-id` (optional): Custom source identifier (defaults to `github-<repo>-<run_id>`)
- `version` (optional): Monitor version to download (default: `latest`)
- `shutdown-timeout` (optional): Seconds to wait for graceful shutdown (default: `5`)

### Action Outputs
- `monitor-pid`: Process ID of running monitor
- `monitor-started`: Boolean indicating successful startup

### Workflow Integration
- Simplifies monitor setup from multi-step manual process to single reusable action
- Handles binary download with fallback warnings
- Validates required environment variables before starting monitor
- Enables version pinning for reproducible CI/CD pipelines

## GitHub Actions Workflow File

### Workflow File
**Location**: `.github/workflows/ci-with-monitor.yml` (114 lines)

### Features:
- **Monitor Binary Download**: Fetches pre-built monitor from releases (x86_64-unknown-linux-gnu)
- **Background Execution**: Starts monitor daemon in background with `./vibetea-monitor run &`
- **Environment Setup**: Sets VIBETEA_PRIVATE_KEY and VIBETEA_SERVER_URL from GitHub secrets
- **Source ID Tracking**: Uses `github-{owner}/{repo}-{run_id}` format for workflow traceability
- **Graceful Shutdown**: Sends SIGTERM to monitor at workflow end with 2-second flush window
- **Non-Blocking**: Network failures don't fail workflow (monitor exits gracefully)
- **CI Integration**: Runs alongside existing Rust/TypeScript tests and builds
- **Binary Caching**: Uses standard GitHub Actions cache for cargo dependencies

### Configuration:
- Private key from `secrets.VIBETEA_PRIVATE_KEY` (base64-encoded via export-key)
- Server URL from `secrets.VIBETEA_SERVER_URL`
- Custom source ID format enables filtering events by workflow run
- Monitor captures Claude Code events during PR reviews, linting, testing, builds

### Workflow Steps:
1. Checkout code
2. Download monitor binary from releases (with graceful fallback)
3. Start monitor in background before CI steps
4. Run standard CI jobs (formatting, linting, tests, builds)
5. Stop monitor gracefully on workflow completion (always runs)

---

## Phase 2 Enhancements

**Monitor Crypto Module Enhancements** (`monitor/src/crypto.rs`):
- **KeySource enum**: Tracks where private key was loaded from
  - `EnvironmentVariable` - Key from VIBETEA_PRIVATE_KEY env var
  - `File(PathBuf)` - Key loaded from file at specific path
- **public_key_fingerprint()**: Returns first 8 characters of base64-encoded public key
  - Used for key verification in logs without exposing full key
  - Allows users to verify correct keypair with server registration
  - Always 8 characters long, prefix of full public key base64
- **Enhanced logging**: Can now report KeySource at startup (INFO level)
- **Backward compatible**: KeySource is for tracking only, doesn't affect signing/verification

## Phase 3 Enhancements

**Memory Safety & Key Loading Improvements** (`monitor/src/crypto.rs`):
- **zeroize crate integration** (v1.8):
  - Securely wipes sensitive memory (seed bytes, decoded buffers) after use
  - Applied in key generation: seed zeroized after SigningKey construction
  - Applied in load_from_env(): decoded buffer zeroized on both success and error paths
  - Applied in load_with_fallback(): decoded buffer zeroized on error paths
  - Prevents sensitive key material from remaining in memory dumps
  - Complies with FR-020: Zero intermediate key material after key operations

- **load_from_env() method**:
  - Loads Ed25519 private key from `VIBETEA_PRIVATE_KEY` environment variable
  - Expects base64-encoded 32-byte seed (RFC 4648 standard)
  - Trims whitespace (including newlines) before decoding
  - Returns tuple: (Crypto instance, KeySource::EnvironmentVariable)
  - Validates decoded length is exactly 32 bytes
  - Error on missing/empty/invalid base64/wrong length
  - Uses zeroize on both success and error paths

- **load_with_fallback() method**:
  - Implements key precedence: environment variable first, then file
  - If `VIBETEA_PRIVATE_KEY` is set, loads from it with NO fallback on error
  - If env var not set, loads from `{dir}/key.priv` file
  - Returns tuple: (Crypto instance, KeySource indicating source)
  - Enables flexible key management without code changes
  - Error handling: env var errors are terminal (no fallback)

- **seed_base64() method**:
  - Exports private key as base64-encoded string
  - Inverse of load_from_env() for key migration workflows
  - Suitable for storing in `VIBETEA_PRIVATE_KEY` environment variable
  - Marked sensitive: handle with care, avoid logging
  - Used for user-friendly key export workflows

- **CryptoError::EnvVar variant**:
  - New error variant for environment variable issues
  - Returned when `VIBETEA_PRIVATE_KEY` is missing or empty
  - Distinct from file-based key loading errors
  - Enables precise error handling and logging

## Phase 4 Additions

**Monitor CLI Enhancement** (`monitor/src/main.rs` - 566 lines):
- **Clap Subcommand enum**: Structured command parsing with derive macros (Phase 4)
  - Replaces manual argument parsing with type-safe clap framework
  - Command variants: Init, ExportKey, Run
  - Automatic help generation and version output

- **ExportKey subcommand** (lines 101-109):
  - Command: `vibetea-monitor export-key [--path <PATH>]`
  - Loads private key from disk (not environment variable)
  - Outputs base64-encoded seed to stdout (only the key + single newline)
  - All diagnostic messages go to stderr
  - Exit code 0 on success, 1 on missing key/invalid path
  - Suitable for piping to clipboard tools or secret management systems
  - Enables GitHub Actions workflow integration (FR-003, FR-023)

- **run_export_key() function** (lines 180-202):
  - Accepts optional `--path` argument for custom key directory
  - Defaults to `get_key_directory()` if path not provided
  - Calls `Crypto::load()` to read from disk (not env var precedence)
  - Prints only the base64 seed followed by newline to stdout
  - Errors written to stderr with helpful message
  - Exit with code 1 if key not found or unreadable

- **Init command enhancement**:
  - Existing `vibetea-monitor init [--force]` still works
  - Uses clap derive for flags (--force/-f)
  - Displays instructions for exporting key

- **Run command**:
  - Existing functionality preserved
  - Uses tokio runtime builder for async execution

- **CLI help text**:
  - Updated with export-key example
  - Lists all environment variables
  - Shows example workflows

**Integration Test Suite** (`monitor/tests/key_export_test.rs` - 699 lines):
- **Framework**: Uses `serial_test` crate to run tests with `--test-threads=1` (prevents env var interference)
- **EnvGuard RAII pattern**: Saves/restores environment variables on drop for test isolation

- **Test Coverage** (13 tests total):

1. **Round-trip Tests** (FR-027, FR-028):
   - `roundtrip_generate_export_command_import_sign_verify` - Full round-trip: generate → save → export → load env → sign → verify
   - `roundtrip_export_command_signatures_are_identical` - Ed25519 determinism verification

2. **Output Format Tests** (FR-003):
   - `export_key_output_format_base64_with_single_newline` - Validates exact output format (base64 + \n)
   - `export_key_output_is_valid_base64_32_bytes` - Verifies base64 decodes to 32 bytes

3. **Diagnostic Output Tests** (FR-023):
   - `export_key_diagnostics_go_to_stderr` - Confirms stdout contains only base64 (no prose/labels)
   - `export_key_error_messages_go_to_stderr` - Error messages on stderr, stdout empty on failure

4. **Exit Code Tests** (FR-026):
   - `export_key_exit_code_success` - Returns 0 on success
   - `export_key_exit_code_missing_key_file` - Returns 1 for missing key.priv
   - `export_key_exit_code_nonexistent_path` - Returns 1 for non-existent directory

5. **Edge Case Tests**:
   - `export_key_handles_path_with_spaces` - Paths with spaces handled correctly
   - `export_key_suitable_for_piping` - No ANSI codes, no carriage returns
   - `export_key_reads_from_key_priv_file` - Reads from correct file with known seed

**Test Infrastructure**:
- Uses tempfile crate for isolated test directories
- Uses Command::new() to invoke vibetea-monitor binary
- Tests use get_monitor_binary_path() to find compiled binary
- All tests marked with `#[serial]` and `#[test]` attributes
- Tests verify both success and failure paths
- Base64 validation using base64 crate
- Ed25519 signature verification with ed25519_dalek::Verifier

**Requirements Addressed**:
- FR-003: Export-key command outputs base64 key with single newline
- FR-023: Diagnostics on stderr, key on stdout
- FR-026: Exit codes 0 (success), 1 (config error)
- FR-027: Exported key can be loaded via VIBETEA_PRIVATE_KEY
- FR-028: Round-trip verified with signature validation

## Phase 5 Additions

**GitHub Actions Workflow Example** (`.github/workflows/ci-with-monitor.yml` - 114 lines):
- **Purpose**: Demonstrates monitor deployment in GitHub Actions for tracking Claude Code events during CI
- **Binary Download**: Fetches pre-built monitor from releases with graceful fallback
- **Background Execution**: Starts monitor daemon before CI jobs run
- **Environment Setup**: Uses GitHub Actions secrets for VIBETEA_PRIVATE_KEY and VIBETEA_SERVER_URL
- **Source ID Format**: `github-{owner}/{repo}-{run_id}` for workflow traceability and filtering
- **Non-Blocking**: Network/monitor failures don't affect workflow success
- **Graceful Shutdown**: Sends SIGTERM at workflow completion with event flush window
- **Integration**: Works alongside existing Rust and TypeScript CI jobs

**Monitor Privacy Module** (`monitor/src/privacy.rs` - 1039 lines):
- **PrivacyConfig**: Configuration for privacy filtering with optional extension allowlist
- **PrivacyPipeline**: Core privacy processor that sanitizes event payloads before transmission
- **extract_basename()**: Utility function to reduce full paths to secure basenames
- **Sensitive tool detection**: Hardcoded list of tools requiring full context stripping (Bash, Grep, Glob, WebSearch, WebFetch)
- **Extension allowlist**: Optional filtering based on file extensions (configurable via `VIBETEA_BASENAME_ALLOWLIST`)
- **Summary stripping**: Session summary text replaced with neutral "Session ended" message
- **Comprehensive documentation**: Privacy guarantees, examples, and implementation details

**Privacy Test Suite** (`monitor/tests/privacy_test.rs` - 951 lines):
- 18+ comprehensive privacy compliance tests
- Validates Constitution I (Privacy by Design)
- Test categories:
  - Path sanitization (no full paths in output)
  - Sensitive tool context stripping (Bash, Grep, Glob, WebSearch, WebFetch)
  - File content/diff stripping
  - Code prompt/response stripping
  - Command argument removal
  - Summary text neutralization
  - Extension allowlist filtering
  - Sensitive pattern detection (credentials, paths, commands)

**Privacy Pipeline Integration Points** (`monitor/src/lib.rs`):
- Public exports: PrivacyConfig, PrivacyPipeline, extract_basename
- Module documentation: Privacy-first approach explained
- Ready for integration into main event loop

**Configuration**: VIBETEA_BASENAME_ALLOWLIST env var
- Format: Comma-separated extensions (e.g., `.rs,.ts,.md`)
- Handles missing dots: `rs,ts,md` auto-converted to `.rs,.ts,.md`
- Whitespace trimming: ` .rs , .ts ` normalized correctly
- Empty entries filtered: `.rs,,.ts,,,` results in `.rs`, `.ts`
- When not set: All extensions allowed (default privacy-preserving behavior)

## Phase 6 Additions

**GitHub Actions Composite Action** (`.github/actions/vibetea-monitor/action.yml` - 167 lines):
- **Type**: GitHub Actions composite action for reusable monitor integration
- **Binary Management**: Downloads monitor from releases with version control
- **Input Parameters**:
  - `server-url` (required): VibeTea server URL
  - `private-key` (required): Base64-encoded Ed25519 private key
  - `source-id` (optional): Custom source identifier (defaults to `github-<repo>-<run_id>`)
  - `version` (optional): Monitor version (default: `latest`)
  - `shutdown-timeout` (optional): Grace period for shutdown (default: `5` seconds)
- **Output Values**:
  - `monitor-pid`: Process ID of running monitor
  - `monitor-started`: Boolean indicating startup success
- **Features**:
  - Validates required environment variables before start
  - Graceful fallback on download failure
  - Process health check after startup
  - Documentation for manual cleanup step
  - Non-blocking: Warnings instead of errors on network issues

**Monitor Crypto Module** (`monitor/src/crypto.rs` - 438 lines):
- **Crypto struct**: Manages Ed25519 signing key and operations
- **Key generation**: `Crypto::generate()` using OS cryptographically secure RNG
- **Key persistence**: `save()` with file permissions (0600 private, 0644 public)
- **Key loading**: `load()` from directory with validation (32-byte seed check)
- **Public key export**: `public_key_base64()` for server registration
- **Message signing**: `sign()` returning base64-encoded Ed25519 signatures
- **CryptoError enum**: Comprehensive error handling (Io, InvalidKey, Base64, KeyExists)
- **File locations**: `~/.vibetea/key.priv` and `~/.vibetea/key.pub`

**Monitor Sender Module** (`monitor/src/sender.rs` - 544 lines):
- **Sender struct**: HTTP client with event buffering and retry logic
- **SenderConfig**: Configuration with server URL, source ID, buffer size
- **Event buffering**: VecDeque with FIFO eviction when full (1000 events default)
- **Connection pooling**: Reqwest Client with 10 max idle connections per host
- **Exponential backoff**: 1s → 60s with ±25% jitter (10 max attempts)
- **Rate limit handling**: Recognizes 429 status, respects Retry-After header
- **Batch sending**: `send_batch()` for efficient server transmission
- **Event queuing**: `queue()` for buffered operations
- **Flushing**: `flush()` to send all buffered events
- **Graceful shutdown**: `shutdown()` with timeout for final flush
- **SenderError enum**: Http, ServerError, AuthFailed, RateLimited, BufferOverflow, MaxRetriesExceeded, Json
- **Event signing**: Signs JSON payload with X-Signature header using Crypto

**Monitor CLI Module** (`monitor/src/main.rs` - 301-566 lines):
- **Command enum**: Init, Run, Help, Version variants (Phase 6: before clap)
- **ExportKey variant** (Phase 4): Subcommand for GitHub Actions integration
- **init command**: `vibetea-monitor init [--force]`
  - Generates new Ed25519 keypair
  - Saves to ~/.vibetea or VIBETEA_KEY_PATH
  - Displays public key for server registration
  - Prompts for confirmation if keys exist (unless --force)
- **run command**: `vibetea-monitor run`
  - Loads configuration from environment
  - Loads cryptographic keys from disk
  - Creates sender with buffering and retry
  - Waits for shutdown signal (SIGINT/SIGTERM)
  - Graceful shutdown with timeout
- **export-key command** (Phase 4): `vibetea-monitor export-key [--path]`
  - Outputs base64-encoded private key to stdout
  - All diagnostics to stderr
  - Exit code 0 on success, 1 on error
- **CLI parsing**: Manual argument parsing with support for flags (Phase 6), clap Subcommand (Phase 4)
- **Logging initialization**: Environment-based filtering via RUST_LOG
- **Signal handling**: Unix SIGTERM + SIGINT support (cross-platform)
- **Help/Version**: Built-in documentation

**Module Exports** (`monitor/src/lib.rs`):
- Public: Crypto, CryptoError, Sender, SenderConfig, SenderError
- Documentation updated with new modules (crypto, sender)

**Key Features of Phase 6**:
- Complete cryptographic pipeline for event authentication
- Buffered, resilient HTTP client for event transmission
- User-friendly CLI for key generation and monitor operation
- Graceful shutdown with event flushing
- Structured error handling throughout
- Constant-time signature operations via ed25519-dalek
- Reusable GitHub Actions composite action for simplified CI integration

## Phase 7 Additions

**Client WebSocket Hook** (`client/src/hooks/useWebSocket.ts` - 321 lines):
- **useWebSocket()**: Custom React hook for WebSocket management
- **Auto-reconnection**: Exponential backoff (1s initial, 60s max, ±25% jitter)
- **Connection state**: Tracks connecting, connected, reconnecting, disconnected states
- **Token management**: Reads authentication token from localStorage
- **Event dispatch**: Integrates with Zustand event store via `addEvent()`
- **Manual control**: Provides `connect()` and `disconnect()` methods
- **Message parsing**: Validates incoming messages as VibeteaEvent type
- **Error handling**: Logs connection errors, gracefully handles message failures
- **Cleanup**: Proper teardown on unmount with timeout clearing
- **Connection status**: Returns `isConnected` boolean for UI binding

**Connection Status Component** (`client/src/components/ConnectionStatus.tsx` - 106 lines):
- **Visual indicator**: Colored dot showing connection state
- **Status colors**: Green (connected), Yellow (connecting/reconnecting), Red (disconnected)
- **Optional label**: Shows status text ("Connected", "Connecting", "Reconnecting", "Disconnected")
- **Selective subscription**: Uses Zustand selector to prevent unnecessary re-renders
- **Accessibility**: ARIA roles and labels for screen readers
- **Configurable**: `showLabel` and `className` props for flexibility
- **Responsive**: Tailwind CSS utility classes for styling

**Token Form Component** (`client/src/components/TokenForm.tsx` - 201 lines):
- **Token input**: Password-protected input field for authentication token
- **Local storage**: Persists token to localStorage with `TOKEN_STORAGE_KEY`
- **Save/Clear buttons**: User can save new token or clear existing one
- **Status indicator**: Visual indicator showing "Token saved" or "No token saved"
- **Form validation**: Validates input before saving (non-empty trim)
- **Cross-window sync**: Detects token changes from other tabs via storage event
- **Callback hook**: Optional `onTokenChange` callback to trigger reconnection
- **Accessibility**: Labels, status roles, aria-live announcements
- **Styling**: Tailwind CSS with dark mode support, button states (disabled/hover)
- **Token masking**: Uses password input type to mask visible token value

**Client Type Enhancements** (`client/src/types/events.ts`):
- Complete type definitions already established in Phase 4-6
- Includes discriminated unions, type guards, and payload mapping
- Used by all client components for type-safe event handling

**Integration Points** (Phase 7):
- `useWebSocket()` hook reads token from TokenForm via localStorage
- ConnectionStatus displays real-time connection state from useEventStore
- TokenForm allows users to manage authentication before connecting
- All components use selective Zustand subscriptions for performance
- Proper TypeScript strict mode compliance throughout

## Phase 8 Additions

**Client Event Stream Component** (`client/src/components/EventStream.tsx` - 425 lines):
- **Virtual scrolling**: Uses `@tanstack/react-virtual` for efficient rendering of 1000+ events
- **Estimated row height**: 64 pixels per event row
- **Auto-scroll behavior**: Automatically scrolls to latest events unless user manually scrolls up
- **Auto-scroll threshold**: 50 pixels distance from bottom to disable auto-scroll
- **Jump to latest button**: Displays when auto-scroll is paused, shows count of new events
- **Event type icons**: Emoji mapping for session, activity, tool, agent, summary, error types
- **Color-coded badges**: Visual badges for each event type with Tailwind CSS colors
- **Event description extraction**: Concise event summaries showing key information
- **Timestamp formatting**: Displays RFC 3339 timestamps as HH:MM:SS format
- **Empty state**: Friendly message when no events are available
- **Sub-components**: EventRow (single event), JumpToLatestButton, EmptyState
- **Accessibility**: ARIA labels, roles, and live region for screen readers
- **Performance**: Selective subscriptions to prevent unnecessary re-renders
- **Responsive design**: Full-height scrollable container with flexible width

**Formatting Utilities Module** (`client/src/utils/formatting.ts` - 331 lines):
- **formatTimestamp()**: Formats RFC 3339 timestamps to HH:MM:SS (local timezone)
- **formatTimestampFull()**: Formats RFC 3339 timestamps to YYYY-MM-DD HH:MM:SS
- **formatRelativeTime()**: Formats timestamps as relative time ("5m ago", "yesterday", etc.)
- **formatDuration()**: Converts milliseconds to human-readable duration ("1h 30m", "5m 30s")
- **formatDurationShort()**: Converts milliseconds to compact format ("1:30:00", "5:30")
- **Helper functions**: parseTimestamp(), padZero(), isSameDay(), isYesterday()
- **Graceful error handling**: Returns fallback strings for invalid input
- **Pure functions**: No side effects, entirely deterministic
- **Time unit constants**: MS_PER_SECOND, MS_PER_MINUTE, MS_PER_HOUR, MS_PER_DAY, MS_PER_WEEK
- **Fallback strings**: Custom fallback values for each function type
- **Comprehensive documentation**: JSDoc comments with examples for each function

**Formatting Tests** (`client/src/__tests__/formatting.test.ts` - 229 lines):
- **33 comprehensive test cases**: Full coverage of all formatting functions
- **Test framework**: Vitest with descriptive test groups
- **formatTimestamp tests** (6 tests):
  - Valid RFC 3339 timestamps
  - Timezone offset handling
  - Empty string fallback
  - Invalid timestamp handling
  - Whitespace-only input
- **formatTimestampFull tests** (4 tests):
  - Valid full datetime formatting
  - Timezone offset handling
  - Empty string and invalid input fallbacks
- **formatRelativeTime tests** (8 tests):
  - "just now" for recent events (<1 minute, future timestamps)
  - Minutes ago ("1m", "5m", "59m")
  - Hours ago ("1h", "2h", "23h")
  - "yesterday" detection with timezone-aware testing
  - Days ago ("3d", "6d")
  - Weeks ago ("1w", "2w", "9w")
  - Invalid input handling
- **formatDuration tests** (10 tests):
  - Hours and minutes ("1h 30m", "2h 1m")
  - Minutes and seconds ("5m 30s", "1m 30s")
  - Seconds only ("30s", "59s")
  - Omits seconds when hours present
  - Zero and negative values
  - NaN handling
  - Large durations (48h, 100h)
- **formatDurationShort tests** (5 tests):
  - H:MM:SS format for durations >= 1 hour
  - M:SS format for durations < 1 hour
  - Leading zeros for seconds
  - Zero and negative value handling
  - NaN and large durations
- **Test coverage**: 100% of exported functions and key code paths

**Integration Points** (Phase 8):
- EventStream component displays events from Zustand store
- Formatting utilities used throughout client for consistent time display
- EventStream uses formatTimestamp() for event timestamps
- EventRow component uses event type for visual styling and icons
- Tests validate formatting across various time zones and edge cases

## Phase 9 Additions

**Client Activity Heatmap Component** (`client/src/components/Heatmap.tsx` - 590 lines):
- **CSS Grid Layout**: `grid-template-columns: auto repeat(24, minmax(0, 1fr))` for hours
- **Color Scale**: 5-level gradient from dark (#1a1a2e) to bright green (#5dad6f)
  - 0 events: #1a1a2e
  - 1-10 events: #2d4a3e
  - 11-25 events: #3d6b4f
  - 26-50 events: #4d8c5f
  - 51+ events: #5dad6f
- **View Toggle**: Switch between 7-day and 30-day views
- **Timezone-Aware Hour Alignment**: Uses `Date.getHours()` (local time)
- **Cell Click Filtering**: `onCellClick` callback with start/end Date objects
- **Accessibility**:
  - `role="grid"`, `role="row"`, `role="gridcell"` structure
  - `aria-label` on each cell with event count and datetime
  - Keyboard navigation (Enter/Space to click cells)
  - Focus indicators with ring styling
- **Tooltip on Hover**: Shows event count and formatted datetime
- **Empty State**: Calendar icon with helpful message
- **Legend**: Visual color scale indicator

**Sub-components**:
- `ViewToggle`: 7/30-day selector with `role="group"` and `aria-pressed`
- `HourHeader`: Hour labels (0, 6, 12, 18) for X-axis
- `CellTooltip`: Positioned tooltip showing cell details
- `HeatmapCellComponent`: Individual cell with hover/click handlers
- `EmptyState`: Calendar icon with guidance text

**Helper Functions**:
- `getHeatmapColor(count)`: Returns CSS color for event count
- `getBucketKey(timestamp)`: Creates "YYYY-MM-DD-HH" key from RFC 3339 timestamp
- `countEventsByHour(events)`: Aggregates events into hour buckets
- `generateHeatmapCells(days, counts)`: Creates cell data for grid rendering
- `formatCellDateTime(date, hour)`: Formats "Day, Mon DD at HH:00"

**Integration Points** (Phase 9):
- Heatmap subscribes to events from Zustand store
- Uses memoization (useMemo) for event counting and cell generation
- Provides onCellClick callback for parent to filter EventStream

## Phase 10 Additions

**Client Session Timeout Hook** (`client/src/hooks/useSessionTimeouts.ts` - 48 lines):
- **useSessionTimeouts()**: Custom React hook for session state management
- **Periodic checking**: Sets up interval using `SESSION_CHECK_INTERVAL_MS` (30 seconds)
- **State transitions**:
  - Active → Inactive: After 5 minutes without events (INACTIVE_THRESHOLD_MS = 300,000ms)
  - Inactive/Ended → Removed: After 30 minutes without events (REMOVAL_THRESHOLD_MS = 1,800,000ms)
- **Integration**: Calls `updateSessionStates()` from Zustand store
- **Cleanup**: Properly clears interval on unmount
- **App-level integration**: Should be called once at root level (App.tsx)
- **No parameters**: Hook manages its own interval lifecycle

**Session Overview Component** (`client/src/components/SessionOverview.tsx` - 484 lines):
- **Session Cards**: Displays active, idle, and ended sessions with rich information
- **Real-time Activity Indicators**: Pulsing dot showing activity level
  - 1-5 events in 60s: 1Hz pulse (slow)
  - 6-15 events in 60s: 2Hz pulse (medium)
  - 16+ events in 60s: 3Hz pulse (fast)
  - Inactive sessions: Gray dot, no pulse
- **Status Badges**: Color-coded session state
  - Active: Green badge with "Active" label
  - Inactive: Yellow badge with "Idle" label
  - Ended: Gray badge with "Ended" label
- **Session Information**:
  - Project name as title
  - Source identifier
  - Session duration (formatted with formatDuration)
  - Event count for active sessions
  - "Last active" timestamp for inactive sessions
- **Session Sorting**: Active sessions first, then by last event time descending
- **Recent Event Counting**: 60-second window for activity indicator calculation
- **Sub-components**:
  - `ActivityIndicator`: Pulsing dot with activity-based animation
  - `StatusBadge`: Color-coded status label
  - `SessionCard`: Individual session display
  - `EmptyState`: Helpful message when no sessions
- **Click Handlers**: Optional `onSessionClick` callback for filtering events by session
- **Keyboard Navigation**: Full accessibility support (Enter/Space to activate)
- **Accessibility**:
  - `role="region"` for container
  - `role="list"` and `role="listitem"` for session cards
  - `aria-label` for cards describing status, duration, and project
  - Keyboard focus support with visual indicators
- **Styling**: Dark mode Tailwind CSS with opacity changes for inactive sessions

**Zustand Store Enhancement** (`client/src/hooks/useEventStore.ts`):
- **New Constants**:
  - `INACTIVE_THRESHOLD_MS = 300,000` (5 minutes)
  - `REMOVAL_THRESHOLD_MS = 1,800,000` (30 minutes)
  - `SESSION_CHECK_INTERVAL_MS = 30,000` (30 seconds)
- **Session Interface Enhanced**:
  - `sessionId: string` - Unique session identifier
  - `source: string` - Monitor source ID
  - `project: string` - Project name
  - `startedAt: Date` - Session start time
  - `lastEventAt: Date` - Time of most recent event
  - `eventCount: number` - Total events in session
  - `status: SessionStatus` - 'active' | 'inactive' | 'ended'
- **New Action**: `updateSessionStates(): void`
  - Transitions sessions between states based on time thresholds
  - Removes sessions after 30 minutes of inactivity
  - Called periodically by useSessionTimeouts hook
  - Updates lastEventAt when new events arrive
  - Maintains session state machine invariants
- **Event Processing**:
  - `addEvent()` updates lastEventAt for corresponding session
  - Session created on first event with sessionId from payload
  - Session transitioned to 'ended' on summary event
  - Session status transitions to 'inactive' after inactivity timeout
- **Map-based Storage**: Sessions stored in Map<string, Session> keyed by sessionId

**Animation Constants** (`client/src/index.css`):
- **Pulse Animations** (already in Phase 9):
  - `pulse-slow`: 1Hz (1 second cycle) - opacity 1→0.6→1, scale 1→1.1→1
  - `pulse-medium`: 2Hz (0.5 second cycle) - same animation, faster
  - `pulse-fast`: 3Hz (0.33 second cycle) - same animation, fastest
- **Keyframes** (`@keyframes`):
  - Define opacity and scale transformation at 0%, 50%, 100% points
  - Used by ActivityIndicator for pulse effects based on event volume

**Integration Points** (Phase 10):
- SessionOverview component subscribes to sessions and events from Zustand store
- useSessionTimeouts hook manages periodic state transitions
- SessionOverview calculates recent event counts for activity indicators
- Pulse animations defined in index.css applied via ActivityIndicator component
- Session click handler allows filtering events by session (future feature)

## Not Yet Implemented

- Main event loop integration (watcher, parser, privacy, crypto, sender pipeline)
- Database/persistence layer
- Advanced state management patterns (beyond Context + Zustand)
- Session persistence beyond memory
- Request/response logging to external services
- Enhanced error tracking
- Per-user authentication tokens (beyond static bearer token)
- Token rotation and expiration
- Chunked event sending for high-volume sessions
- Background task spawning for async file watching and sending
- Session filtering/search in client UI
- Advanced event replay and history features
