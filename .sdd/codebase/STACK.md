# Technology Stack

**Status**: Phase 8 Implementation - Virtual scrolling EventStream component, formatting utilities
**Last Updated**: 2026-02-02

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, JSONL parsing, privacy filtering, event signing, HTTP transmission |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization |

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
| futures-util       | 0.3     | WebSocket stream utilities | Server |
| futures            | 0.3     | Futures trait and utilities | Monitor (async coordination) |

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
| CLI Support | Manual command parsing in monitor main.rs (init, run, help, version) |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature with X-Signature header |
| Server → Client | WebSocket | JSON | Bearer token |
| Client → Server | WebSocket | JSON | Bearer token |
| Monitor → File System | Native | JSONL | N/A (local file access) |

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
| Monitor | Binary | ELF/Mach-O/PE | Standalone executable for users |
| Client | Static files | JS + CSS (Brotli compressed) | CDN (Netlify/Vercel/Cloudflare) |

## Module Organization

### Client (`client/src`)
- `components/` - React components
  - `ConnectionStatus.tsx` - **Phase 7**: Visual WebSocket connection status indicator
  - `TokenForm.tsx` - **Phase 7**: Token management and persistence UI
  - `EventStream.tsx` - **Phase 8**: Virtual scrolling event stream with 1000+ event support
- `hooks/useEventStore.ts` - Zustand store for WebSocket event state with selective subscriptions
- `hooks/useWebSocket.ts` - **Phase 7**: WebSocket connection management with auto-reconnect
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
- `crypto.rs` - **Phase 6**: Ed25519 keypair generation, loading, saving, and event signing
- `sender.rs` - **Phase 6**: HTTP client with event buffering, exponential backoff retry, and rate limit handling
- `main.rs` - **Phase 6**: CLI entry point with init and run commands
- `lib.rs` - Public interface

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary, minimal base image |
| Client | CDN | Static files | Optimized builds with compression |
| Monitor | Local | Native binary | Users download and run locally |

## Phase 4 Additions

**Monitor Parser Module** (`monitor/src/parser.rs`):
- Claude Code JSONL parsing with privacy-first approach
- Extracts only metadata: tool names, timestamps, file basenames
- Never processes code content, prompts, or assistant responses
- Event mapping: assistant tool_use → ToolStarted, progress PostToolUse → ToolCompleted
- SessionParser state tracking for multi-line file processing
- ParsedEvent and ParsedEventKind types for normalized event representation
- Support for session detection from file paths (slugified project names)
- Comprehensive ParseError enum for error handling

**Monitor File Watcher Module** (`monitor/src/watcher.rs`):
- Watches `~/.claude/projects/**/*.jsonl` for changes using notify crate
- Position tracking map to efficiently tail files (no re-reading previous content)
- WatchEvent enum: FileCreated, LinesAdded, FileRemoved
- BufReader-based line reading with seek position management
- Automatic cleanup of removed files from position tracking
- WatcherError enum for I/O and initialization failures
- Thread-safe Arc<RwLock<>> position map for async operation

**New Dependencies**:
- `futures` 0.3 - Futures trait and utilities for async coordination
- `tempfile` 3.15 - Temporary file/directory management for testing

**Enhanced Module Exports** (`monitor/src/lib.rs`):
- Public exports: FileWatcher, WatchEvent, WatcherError
- Public exports: SessionParser, ParsedEvent, ParsedEventKind
- Documentation expanded with overview, privacy statement, and module descriptions

## Phase 5 Additions

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

**Monitor CLI Module** (`monitor/src/main.rs` - 301 lines):
- **Command enum**: Init, Run, Help, Version variants
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
- **CLI parsing**: Manual argument parsing with support for flags
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
