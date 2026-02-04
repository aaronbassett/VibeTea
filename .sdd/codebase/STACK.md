# Technology Stack

**Status**: Phase 11 - Project activity tracking for multi-project monitoring
**Last Updated**: 2026-02-04

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, JSONL parsing, privacy filtering, event signing, HTTP transmission, CLI with export-key, project activity tracking |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization |
| GitHub Actions | YAML/Bash | - | Composite action for monitor integration and workflow orchestration |

## Frameworks & Core Libraries

### Rust (Monitor & Server)

| Package            | Version | Purpose | Used By |
|--------------------|---------|---------|----------|
| tokio              | 1.43    | Async runtime with full features (threads, signals, timers) | Server, Monitor |
| axum               | 0.8     | HTTP/WebSocket server framework with routing and middleware | Server |
| tower              | 0.5     | Composable middleware and service abstractions | Server |
| tower-http         | 0.6     | HTTP utilities (CORS, tracing, compression) | Server |
| reqwest            | 0.12    | HTTP client library with connection pooling and timeouts | Monitor sender, Server tests |
| serde              | 1.0     | Serialization/deserialization with derive macros | All |
| serde_json         | 1.0     | JSON format handling and streaming | All |
| ed25519-dalek      | 2.1     | Ed25519 cryptographic signing and verification | Server auth, Monitor crypto |
| uuid               | 1.11    | Unique identifiers (v4, v5) for events and sessions | Server, Monitor |
| chrono             | 0.4     | Timestamp handling with serde support | Server, Monitor |
| thiserror          | 2.0     | Derive macros for error types | Server, Monitor |
| anyhow             | 1.0     | Flexible error handling and context | Server, Monitor |
| tracing            | 0.1     | Structured logging framework | Server, Monitor |
| tracing-subscriber | 0.3     | Logging implementation (JSON, env-filter) | Server, Monitor |
| notify             | 8.0     | File system watching (inotify/FSEvents/ReadDirectoryChangesW) | Monitor (sessions, skills, todos, stats, projects) |
| base64             | 0.22    | Base64 encoding/decoding | Server, Monitor |
| rand               | 0.9     | Random number generation | Server, Monitor |
| directories        | 6.0     | Standard directory paths | Monitor |
| gethostname        | 1.0     | System hostname retrieval | Monitor |
| subtle             | 2.6     | Constant-time comparison for cryptography | Server (auth) |
| zeroize            | 1.8     | Secure memory wiping for cryptographic material | Monitor (Phase 3) |
| futures-util       | 0.3     | WebSocket stream utilities | Server |
| futures            | 0.3     | Futures trait and utilities | Monitor (async coordination) |
| clap               | 4.5     | CLI argument parsing with derive macros | Monitor (clap Subcommand/Parser for export-key, Phase 4) |
| lru                | 0.12    | LRU cache for session state tracking (Phase 11) | Monitor project tracker |
| similar            | 2.6     | Line diffing for file history tracking (Phase 11) | Monitor stats tracker |

### TypeScript/JavaScript (Client)

| Package                    | Version  | Purpose |
|---------------------------|----------|---------|
| React                      | ^19.2.4  | UI framework for component-based architecture |
| React DOM                  | ^19.2.4  | DOM rendering and lifecycle management |
| TypeScript                 | ^5.9.3   | Static type checking and transpilation |
| Vite                       | ^7.3.1   | Build tool and dev server with HMR |
| Tailwind CSS               | ^4.1.18  | Utility-first CSS framework for styling |
| Zustand                    | ^5.0.11  | Lightweight state management without boilerplate |
| @tanstack/react-virtual    | ^3.13.18 | Virtual scrolling for efficient rendering of 1000+ events |
| @vitejs/plugin-react       | ^5.1.3   | React Fast Refresh for HMR in Vite |
| @tailwindcss/vite          | ^4.1.18  | Tailwind CSS Vite plugin for CSS compilation |
| vite-plugin-compression2   | ^2.4.0   | Brotli compression for optimized production builds |

## Build Tools & Package Managers

| Tool     | Version  | Purpose |
|----------|----------|---------|
| cargo    | -        | Rust package manager and build system with workspaces |
| pnpm     | -        | Node.js package manager with monorepo support |
| rustfmt  | -        | Rust code formatter enforcing consistent style |
| clippy   | -        | Rust linter for code quality |
| prettier | ^3.8.1   | Code formatter for TypeScript and CSS |
| ESLint   | ^9.39.2  | Linter for JavaScript/TypeScript code quality |

## Testing Infrastructure

### Rust Testing
| Package      | Version | Purpose |
|--------------|---------|---------|
| tokio-test   | 0.4     | Tokio testing utilities for async tests |
| tempfile     | 3.15    | Temporary file/directory management for tests |
| serial_test  | 3.2     | Serial test execution for environment variable tests |
| wiremock     | 0.6     | HTTP mocking for integration tests |

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit and component testing framework |
| @testing-library/react | ^16.3.2  | React component testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM assertion helpers |
| jsdom                  | ^28.0.0  | Full DOM implementation for tests |
| happy-dom              | ^20.5.0  | Lightweight DOM for faster tests |

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
| File System Monitoring | Rust notify crate (inotify/FSEvents/ReadDirectoryChangesW) for multi-file tracking: JSONL sessions, history, todos, stats, projects |
| CLI Support | clap Subcommand enum for command parsing (init, run, export-key via clap derive macros, Phase 4) |
| CI/CD Integration | GitHub Actions workflow with monitor deployment and background execution (Phase 5), composite action wrapper (Phase 6) |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature with X-Signature header |
| Server → Client | WebSocket | JSON | Bearer token |
| Client → Server | WebSocket | JSON | Bearer token |
| Monitor → File System | Native | JSONL (sessions/history), JSON (todos/stats), detect (projects) | N/A (local file access) |
| GitHub Actions | SSH/HTTPS | Binary download + env var injection | GitHub Actions secrets |

## Data Serialization

| Component | Serialization | Notes |
|-----------|---------------|-------|
| Server/Monitor | serde (Rust) | JSON with snake_case for serde rename |
| Client | TypeScript/JSON | camelCase for API contracts |
| Events | serde_json | Standardized event schema across all components |
| Claude Code Sessions | JSONL (JSON Lines) | Privacy-first parsing extracting metadata only |
| History File | JSONL (JSON Lines) | One JSON object per line, append-only file |
| Todo Files | JSON Array | Array of todo entries with status fields |
| Stats Cache | JSON Object | Claude Code stats-cache.json with model usage data and hour counts |
| Project Activity | File system events | Directory structure scanning, summary event detection |
| Cryptographic Keys | Base64 + raw bytes | Public keys base64-encoded, private keys raw 32-byte seeds |

## Build Output

| Component | Output | Format | Deployment |
|-----------|--------|--------|-----------|
| Server | Binary | ELF (Linux) | Docker container on Fly.io |
| Monitor | Binary | ELF/Mach-O/PE | Standalone executable for users or GitHub Actions |
| Client | Static files | JS + CSS (Brotli compressed) | CDN (Netlify/Vercel/Cloudflare) |

## Module Organization

### Client (`client/src`)
- `components/` - React components
  - `ConnectionStatus.tsx` - Visual WebSocket connection indicator
  - `TokenForm.tsx` - Token management with localStorage persistence
  - `EventStream.tsx` - Virtual scrolling for 1000+ events with auto-scroll (Phase 10: activity_pattern and model_distribution event handlers)
  - `Heatmap.tsx` - Activity heatmap with 7/30-day views and color scale
  - `SessionOverview.tsx` - Session cards with real-time activity indicators
- `hooks/` - Custom React hooks
  - `useEventStore.ts` - Zustand store for event state with session timeout management
  - `useWebSocket.ts` - WebSocket management with auto-reconnect
  - `useSessionTimeouts.ts` - Periodic session state transitions
- `types/events.ts` - Discriminated union event types matching server schema (Phase 10: added ActivityPatternPayload, ModelDistributionPayload with type guards)
- `utils/formatting.ts` - Timestamp and duration formatting (5 functions)
- `__tests__/` - Test suite with 33+ test cases

### Server (`server/src`)
- `config.rs` - Configuration from environment (ports, keys, tokens)
- `auth.rs` - Ed25519 signature verification with constant-time comparison
- `broadcast.rs` - Event broadcasting via tokio channels with filtering
- `rate_limit.rs` - Per-source token bucket rate limiting (100 events/sec)
- `routes.rs` - HTTP endpoints (POST /events, GET /ws, GET /health)
- `error.rs` - Comprehensive error types and handling
- `types.rs` - Event types and data models (Phase 10: ActivityPatternEvent, ModelDistributionEvent; Phase 11: ProjectActivityEvent)
- `main.rs` - Server binary entry point

### Monitor (`monitor/src`)
- `config.rs` - Configuration from environment variables (server URL, source ID, key path, buffer size)
- `error.rs` - Error types
- `types.rs` - Event types (Phase 11: ProjectActivityEvent struct)
- `parser.rs` - Claude Code JSONL parser (privacy-first metadata extraction)
- `watcher.rs` - File system watcher for `.claude/projects/**/*.jsonl` files with position tracking
- `privacy.rs` - **Phase 5**: Privacy pipeline for event sanitization before transmission
- `crypto.rs` - **Phase 3-6**: Ed25519 keypair generation, loading, saving, and event signing with memory safety
- `sender.rs` - **Phase 6**: HTTP client with event buffering, exponential backoff retry, and rate limit handling
- `trackers/` - **Phase 11**: Project and session activity tracking
  - `agent_tracker.rs` - Task tool agent spawn tracking
  - `skill_tracker.rs` - Skill invocation tracking from history.jsonl
  - `stats_tracker.rs` - Token usage and session metrics from stats-cache.json (Phase 10)
  - `todo_tracker.rs` - Todo progress tracking from todos/*.json
  - `project_tracker.rs` - **Phase 11**: Project activity detection via `~/.claude/projects/` directory scanning
- `utils/` - Utility modules for tokenization, debouncing, session filename parsing
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

## Phase 10 - Enhanced Activity Pattern and Model Distribution Tracking (Complete)

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

2. **ActivityPatternEvent**:
   - Source: `hourCounts` field from stats-cache.json
   - Field: `hour_counts: HashMap<String, u64>`
   - Keys: String hours "0" through "23" for JSON reliability
   - Values: Activity count per hour
   - Emitted: Once per stats-cache.json read (before token events)
   - Purpose: Real-time hourly distribution visualization

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

4. **Event Emission Order**:
   - SessionMetricsEvent (global stats)
   - ActivityPatternEvent (hourly breakdown)
   - TokenUsageEvent for each model (individual metrics)
   - ModelDistributionEvent (aggregated by model)

### Server Type Additions (`server/src/types.rs`):
- `EventType::ActivityPattern` - New enum variant
- `EventType::ModelDistribution` - New enum variant
- `ActivityPatternEvent` - Struct with `hour_counts: HashMap<String, u64>`
- `ModelDistributionEvent` - Struct with `model_usage: HashMap<String, TokenUsageSummary>`
- `TokenUsageSummary` - New struct for per-model token breakdown

### Client Type Additions (`client/src/types/events.ts`):
- `'activity_pattern'` - New EventType variant
- `'model_distribution'` - New EventType variant
- `ActivityPatternPayload` - Interface with `hourCounts: Record<string, number>`
- `ModelDistributionPayload` - Interface with `modelUsage: Record<string, TokenUsageSummary>`
- `isActivityPatternEvent()` - Type guard function
- `isModelDistributionEvent()` - Type guard function
- Event mapping entries in `EventPayloadMap`

### Client Event Display (`client/src/components/EventStream.tsx`):
- **Icon**: activity_pattern uses 📈 emoji, model_distribution uses 🤖 emoji
- **Colors**: activity_pattern uses teal-600 styling, model_distribution uses orange-600 styling
- **Event Descriptions**:
  - activity_pattern: "Activity pattern: {N} hours tracked"
  - model_distribution: "Model distribution: {N} model(s) used"
- **Case handlers** in `getEventDescription()` for both new event types

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

### Architecture
- Distributed event system: Monitor → Server → Client
- Privacy-by-design throughout pipeline
- Cryptographic authentication for event integrity
- Efficient file watching with position tracking and debouncing
- Virtual scrolling for high-volume event display
- Multi-source project activity detection

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

### Reliability
- Exponential backoff retry with jitter
- Rate limiting protection
- Graceful shutdown with event flushing
- Structured error handling throughout
- Constant-time signature operations via ed25519-dalek
- Reusable GitHub Actions composite action for simplified CI integration

### Security
- Ed25519 signatures on all events
- Constant-time signature verification
- Bearer token authentication for clients
- File permission enforcement (0600 private keys)
- Privacy pipeline stripping sensitive data

## Phase 11 - Project Activity Tracking (In Progress)

**Project Tracker Module** (`monitor/src/trackers/project_tracker.rs` - 500+ lines):
- **ProjectActivityEvent** - New event type for project-level activity tracking
  - Fields: `project_path: String`, `session_id: String`, `is_active: bool`
  - Tracks both active and completed sessions per project
  - Serialized with camelCase for API compatibility

- **ProjectTracker** - File system watcher for `~/.claude/projects/` directory
  - Uses `notify` crate (already in stack) for cross-platform directory monitoring
  - Recursive watching of project subdirectories
  - Session activity detection via summary event presence
  - `parse_project_slug()` - Converts slug format back to absolute paths
  - `has_summary_event()` - Detects session completion state
  - `create_project_activity_event()` - Factory function for event construction

- **ProjectTrackerConfig** - Configuration for tracker behavior
  - `scan_on_init: bool` - Initial full scan option
  - Default: true (scan all projects on startup)

- **ProjectTrackerError** - Comprehensive error handling
  - WatcherInit: File system watcher setup failures
  - Io: File read/write errors
  - ClaudeDirectoryNotFound: Missing project directory
  - ChannelClosed: Event sender channel unavailable

- **Features**:
  - No debouncing needed (project/* files change infrequently)
  - Async/await compatible with tokio runtime
  - Thread-safe via mpsc channels
  - Graceful error handling and logging
  - Privacy-first: Only paths and session IDs tracked

- **Use Cases**:
  - Track which projects have active sessions
  - Detect when sessions are completed
  - Multi-project monitoring dashboards
  - Correlate activity across projects

**Monitor Type Additions** (`monitor/src/types.rs` - Phase 11):
- `EventPayload::ProjectActivity(ProjectActivityEvent)` - New event variant
- `ProjectActivityEvent` struct with camelCase serialization
- Full enum support for discriminated union event types

---

*This document captures production technologies, frameworks, and dependencies. Version specifications reflect compatibility constraints.*
