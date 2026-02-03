# Technology Stack

**Status**: Phase 4 Implementation - Agent spawning and token usage tracking
**Last Updated**: 2026-02-03

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
| lru                | 0.12    | LRU cache for session tracking | Monitor (Phase 4) |
| clap               | 4.5     | CLI argument parsing | Monitor |
| serial_test        | 3.2     | Serial test execution for env var isolation | Monitor, Server (tests) |

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
| wiremock     | 0.6     | HTTP mocking for integration tests |

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit/component testing framework |
| @testing-library/react | ^16.3.2  | React testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM matchers for testing |
| jsdom                  | ^28.0.0  | DOM implementation for Node.js |
| happy-dom              | ^20.5.0  | Lightweight DOM implementation |

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
- `parser.rs` - Claude Code JSONL parser (privacy-first metadata extraction, Phase 4 agent spawn parsing)
- `watcher.rs` - File system watcher for `.claude/projects/**/*.jsonl` files with position tracking
- `privacy.rs` - **Phase 5**: Privacy pipeline for event sanitization before transmission
- `crypto.rs` - **Phase 6**: Ed25519 keypair generation, loading, saving, and event signing
- `sender.rs` - **Phase 6**: HTTP client with event buffering, exponential backoff retry, and rate limit handling
- `main.rs` - **Phase 6**: CLI entry point with init and run commands
- `trackers/` - **Phase 4**: Enhanced tracking modules
  - `agent_tracker.rs` - Task tool agent spawn detection and parsing
  - `stats_tracker.rs` - Token usage and session statistics accumulation
  - `mod.rs` - Trackers module exports
- `lib.rs` - Public interface

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary, minimal base image |
| Client | CDN | Static files | Optimized builds with compression |
| Monitor | Local | Native binary | Users download and run locally |

## Phase 4 Additions

**Monitor Agent Tracker Module** (`monitor/src/trackers/agent_tracker.rs` - 716 lines):
- **AgentSpawnEvent**: Event type for Task tool agent spawns with session_id, agent_type, description, timestamp
- **TaskToolInput**: Parsed Task tool input with subagent_type and description fields
- **parse_task_tool_use()**: Extracts Task tool metadata from Claude Code events, ignores prompt field (privacy-first)
- **create_agent_spawn_event()**: Constructs AgentSpawnEvent from parsed input and session context
- **try_extract_agent_spawn()**: Convenience function combining parsing and event creation
- **Privacy-first approach**: Only subagent_type and description extracted; prompt field never transmitted
- **40+ comprehensive test cases**: Validates Task tool parsing, edge cases, malformed input handling
- **ParsedEventKind::AgentSpawned**: New variant in parser for agent spawn events
- **Integration in parser.rs**: Task tool invocations emit both ToolStarted and AgentSpawned events

**Monitor Stats Tracker Module** (`monitor/src/trackers/stats_tracker.rs` - 265 lines):
- **StatsTracker**: Accumulates token usage and session statistics from events
- **Session-level metrics**: Input/output tokens, cache read/write tokens per model
- **Global metrics**: Total sessions, messages, tool usage across all sessions
- **Token usage aggregation**: Per-model token consumption tracking with cache hit rates
- **Activity pattern tracking**: Hourly distribution of events across 24-hour windows
- **Model distribution**: Token usage breakdown by model with detailed summaries
- **Test suite**: Validates token accumulation, multiple models, cache metrics

**Enhanced Parser** (`monitor/src/parser.rs`):
- **AgentSpawned event kind**: New ParsedEventKind variant for Task tool spawns
- **Task tool handling**: Detects Task tool invocations, extracts subagent_type and description
- **Dual event emission**: Task tools emit both ToolStarted (for Task tool itself) and AgentSpawned (for spawned agent)
- **Agent type mapping**: Maps subagent_type to agent_type field in AgentSpawnEvent
- **Integration with agent_tracker**: Uses parse_task_tool_use() for metadata extraction

**Enhanced Main Event Loop** (`monitor/src/main.rs`):
- **AgentSpawnEvent handling**: Emits Event::AgentSpawn payload for spawned agents
- **StatsTracker integration**: Accumulates token usage from events
- **TokenUsageEvent emission**: Emits token metrics to server for tracking
- **Session-level token tracking**: Maintains per-session token consumption statistics

**New Dependencies**:
- `lru` 0.12 - LRU cache for session tracking in stats tracker
- `clap` 4.5 - Structured CLI argument parsing with derive macros
- `serial_test` 3.2 - Serial test execution for environment variable isolation
- `wiremock` 0.6 - HTTP mocking for integration tests

**Enhanced Module Exports** (`monitor/src/lib.rs`):
- Public exports: `trackers` module with `agent_tracker`, `stats_tracker`
- Public exports: `AgentSpawnEvent`, `TaskToolInput`, `parse_task_tool_use`, `try_extract_agent_spawn`
- Documentation updated with tracker module descriptions

**Event Type Enhancements** (`monitor/src/types.rs`):
- **AgentSpawnEvent**: New event type with session_id, agent_type, description, timestamp fields
- **EventType::AgentSpawn**: New enum variant for agent spawn events
- **EventPayload::AgentSpawn**: Payload variant wrapping AgentSpawnEvent
- **TokenUsageEvent**: Tracks per-model token consumption (input, output, cache_read, cache_creation)
- **EventType::TokenUsage**: New enum variant for token tracking events
- **Multiple new event types**: SessionMetrics, ActivityPattern, ModelDistribution, TodoProgress, FileChange, ProjectActivity
- **Enhanced testing**: 30+ tests validating new event types and serialization

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
- **formatTimestamp tests** (6 tests): Valid/invalid timestamps, timezone handling, empty strings
- **formatTimestampFull tests** (4 tests): Full datetime formatting, timezone handling
- **formatRelativeTime tests** (8 tests): Relative time displays, timezone-aware testing
- **formatDuration tests** (10 tests): Duration formatting with various time ranges
- **formatDurationShort tests** (5 tests): Compact duration formatting
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

- Main event loop integration with all modules (watcher, parser, privacy, crypto, sender pipeline)
- Database/persistence layer for event storage and replay
- Advanced state management patterns (beyond Context + Zustand)
- Session persistence beyond memory
- Request/response logging to external services
- Enhanced error tracking and recovery
- Per-user authentication tokens (beyond static bearer token)
- Token rotation and expiration
- Chunked event sending for high-volume sessions
- Background task spawning for async file watching and sending
- Session filtering/search in client UI
- Advanced event replay and history features
