# Project Structure

> **Purpose**: Document directory layout, module boundaries, and where to add new code.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

## Directory Layout

```
VibeTea/
├── monitor/                    # Rust CLI for watching Claude Code sessions
│   ├── src/
│   │   ├── main.rs            # Entry point, CLI commands (default: TUI, init, export-key, run)
│   │   ├── lib.rs             # Module exports
│   │   ├── config.rs          # Environment configuration loading
│   │   ├── watcher.rs         # File system watcher (inotify/FSEvents/ReadDirectoryChangesW)
│   │   ├── parser.rs          # Claude Code JSONL parsing
│   │   ├── privacy.rs         # Event payload sanitization
│   │   ├── crypto.rs          # Ed25519 keypair generation/management with key loading strategies and export
│   │   ├── sender.rs          # HTTP client with retry and buffering
│   │   ├── types.rs           # Event type definitions
│   │   ├── error.rs           # Error types
│   │   └── tui/               # Terminal user interface (default entry point)
│   │       ├── mod.rs         # TUI module orchestration and lifecycle
│   │       └── widgets/       # Reusable TUI widget components
│   │           ├── mod.rs     # Widget catalog and re-exports
│   │           ├── logo.rs    # ASCII art logo with gradient animation
│   │           ├── setup_form.rs # Server URL and registration code input form
│   │           ├── header.rs  # Connection status bar with server info
│   │           ├── event_stream.rs # Scrollable list of session events
│   │           ├── credentials.rs # Device credentials display
│   │           ├── stats_footer.rs # Session statistics and keybinding hints
│   │           └── size_warning.rs # Terminal size warning widget (Phase 11)
│   ├── tests/
│   │   ├── privacy_test.rs    # Privacy filtering tests
│   │   ├── sender_recovery_test.rs  # Retry logic tests
│   │   ├── env_key_test.rs    # Environment variable key loading tests
│   │   └── key_export_test.rs # export-key command integration tests (Phase 4)
│   └── Cargo.toml
│
├── server/                     # Rust HTTP server (event hub)
│   ├── src/
│   │   ├── main.rs            # Entry point, logging, graceful shutdown
│   │   ├── lib.rs             # Module exports
│   │   ├── routes.rs          # HTTP route handlers (POST /events, GET /ws, GET /health)
│   │   ├── auth.rs            # Ed25519 signature verification + token validation
│   │   ├── broadcast.rs       # Event distribution to WebSocket subscribers
│   │   ├── rate_limit.rs      # Per-source rate limiting
│   │   ├── config.rs          # Environment configuration loading
│   │   ├── types.rs           # Event type definitions (shared with monitor)
│   │   ├── error.rs           # Error types
│   │   ├── session.rs         # Session token store for user authentication (Phase 2)
│   │   └── supabase.rs        # Supabase client for JWT validation + key distribution (Phase 2)
│   ├── tests/
│   │   └── unsafe_mode_test.rs # Auth bypass mode tests
│   └── Cargo.toml
│
├── client/                     # React SPA (dashboard)
│   ├── src/
│   │   ├── main.tsx           # ReactDOM entry point
│   │   ├── App.tsx            # Root component (layout + page state)
│   │   ├── components/
│   │   │   ├── ConnectionStatus.tsx   # WebSocket connection indicator
│   │   │   ├── TokenForm.tsx          # Authentication token input
│   │   │   ├── EventStream.tsx        # Virtualized event list
│   │   │   ├── SessionOverview.tsx    # Active sessions table
│   │   │   └── Heatmap.tsx           # Activity over time visualization
│   │   ├── hooks/
│   │   │   ├── useWebSocket.ts       # WebSocket connection management
│   │   │   ├── useEventStore.ts      # Zustand store (state + selectors)
│   │   │   └── useSessionTimeouts.ts # Session state machine (Active → Inactive → Ended)
│   │   ├── types/
│   │   │   └── events.ts             # TypeScript event interfaces
│   │   ├── utils/
│   │   │   └── formatting.ts         # Timestamp, event type formatting
│   │   ├── __tests__/
│   │   │   ├── App.test.tsx          # Integration tests
│   │   │   ├── events.test.ts        # Event parsing/filtering tests
│   │   │   └── formatting.test.ts    # Formatting utility tests
│   │   └── index.css
│   ├── public/
│   ├── vite.config.ts
│   ├── package.json
│   └── tsconfig.json
│
├── supabase/                   # Supabase backend infrastructure (Phase 2)
│   ├── functions/
│   │   └── public-keys/        # Edge function for public key distribution
│   │       └── index.ts        # Deno function: GET /functions/v1/public-keys
│   ├── migrations/
│   │   └── 001_public_keys.sql # Database migration: monitor_public_keys table
│   └── (supabase.json config would be here)
│
├── .github/
│   ├── actions/
│   │   └── vibetea-monitor/           # Composite GitHub Action (Phase 6)
│   │       └── action.yml             # Action definition for simplified monitor setup
│   ├── workflows/
│   │   ├── ci.yml                    # Primary CI workflow (tests, linting, build)
│   │   └── ci-with-monitor.yml       # Example workflow with VibeTea monitoring (Phase 5)
│   ├── ISSUE_TEMPLATE/
│   ├── CODEOWNERS
│   └── PULL_REQUEST_TEMPLATE.md
│
├── discovery/                  # AI assistant discovery module (future expansion)
│   └── src/
│
├── specs/                      # API specifications (future OpenAPI)
│
├── .sdd/
│   └── codebase/               # This documentation
│
├── Cargo.toml                  # Workspace root (members: monitor, server)
├── Cargo.lock
├── PRD.md                      # Product requirements
├── README.md
├── CLAUDE.md                   # Project guidelines & learnings
└── lefthook.yml               # Pre-commit hooks
```

## Key Directories

### `monitor/src/` - Monitor Component

| File | Purpose | Key Types |
|------|---------|-----------|
| `main.rs` | CLI entry (default: TUI, init, run, export-key commands), signal handling | `Cli`, `Command` |
| `config.rs` | Load from env vars: `VIBETEA_*` | `Config` |
| `watcher.rs` | inotify/FSEvents for `~/.claude/projects/**/*.jsonl` | `FileWatcher`, `WatchEvent` |
| `parser.rs` | Parse JSONL, extract Session/Activity/Tool events | `SessionParser`, `ParsedEvent`, `ParsedEventKind` |
| `privacy.rs` | Remove code, prompts, sensitive data | `PrivacyPipeline`, `PrivacyConfig` |
| `crypto.rs` | Ed25519 keypair with dual loading strategy (env var + file fallback) and key export | `Crypto`, `KeySource`, `CryptoError` |
| `sender.rs` | HTTP POST to server with retry/buffering | `Sender`, `SenderConfig`, `RetryPolicy` |
| `types.rs` | Event schema (shared with server) | `Event`, `EventPayload`, `EventType` |
| `error.rs` | Error types | `MonitorError`, custom errors |

### Monitor CLI Commands

The Monitor now supports four modes:

| Command | File Location | Purpose | Arguments |
|---------|---|---------|-----------|
| `(no args)` or `tui` | `monitor/src/main.rs:137-149` | Launch interactive TUI (default) | (none, optional --config) |
| `init` | `monitor/src/main.rs:109-118` | Generate Ed25519 keypair | `--force` (optional) |
| `export-key` | `monitor/src/main.rs:119-127` | Export private key for CI/CD | `--path <DIR>` (optional, defaults to `~/.vibetea`) |
| `run` | `monitor/src/main.rs:128-135` | Start headless monitor daemon | (none, requires `VIBETEA_SERVER_URL`) |

**Phase 11 Addition: TUI as Default Entry Point**
- Lines 137-142: Default command handling (TUI is default when no subcommand)
- Lines 145: `Command::Tui => run_tui()` dispatches to TUI mode
- TUI provides interactive configuration and real-time event monitoring
- Headless `run` command remains available for scripting and background monitoring

### `monitor/src/tui/` - Terminal User Interface

The TUI module provides an interactive terminal interface for configuring and monitoring Claude Code sessions.

| File | Purpose | Key Types |
|------|---------|-----------|
| `mod.rs` | TUI lifecycle, event loop, state management | `Tui`, `App`, `AppState` |
| `widgets/mod.rs` | Widget catalog and re-exports | — |

### `monitor/src/tui/widgets/` - TUI Widgets

Reusable widget components built on ratatui:

| File | Purpose | Key Types | Added In |
|------|---------|-----------|----------|
| `logo.rs` | ASCII art logo with gradient animation | `LogoWidget`, `LogoVariant` | Phase 10 |
| `setup_form.rs` | Server URL and registration code input form | `SetupFormWidget` | Phase 10 |
| `header.rs` | Connection status bar with server info and indicators | `HeaderWidget`, `ConnectionStatusWidget` | Phase 10 |
| `event_stream.rs` | Scrollable list of session events with filtering | `EventStreamWidget` | Phase 10 |
| `credentials.rs` | Device credentials display (public key, device name) | `CredentialsWidget` | Phase 10 |
| `stats_footer.rs` | Session statistics and keybinding hints | `StatsFooterWidget` | Phase 10 |
| `size_warning.rs` | Terminal size warning when dimensions are too small | `SizeWarningWidget`, `TerminalSizeStatus` | Phase 11 |

**Widget Details:**
- **size_warning.rs** (Phase 11): Displays warning when terminal is below 80x24 minimum
  - Functions: `check_terminal_size()`, `get_terminal_size_status()`
  - Constants: `MIN_TERMINAL_WIDTH`, `MIN_TERMINAL_HEIGHT`
  - Handles gracefully for zero-size areas and edge cases
  - Comprehensive tests for all size scenarios and boundary conditions

### Crypto Module Details (`monitor/src/crypto.rs`)

The crypto module provides Ed25519 key management with flexible loading strategies and export support:

| Method | Purpose | Returns |
|--------|---------|---------|
| `Crypto::generate()` | Generate new Ed25519 keypair using OS RNG | `Crypto` instance |
| `Crypto::load_from_env()` | Load 32-byte seed from `VIBETEA_PRIVATE_KEY` env var | `(Crypto, KeySource::EnvironmentVariable)` |
| `Crypto::load_with_fallback(dir)` | Try env var first, fallback to file if not set | `(Crypto, KeySource)` |
| `Crypto::load(dir)` | Load from file only (`dir/key.priv`) | `Crypto` instance |
| `Crypto::save(dir)` | Save keypair to files (mode 0600/0644) | `Result<()>` |
| `Crypto::public_key_base64()` | Get public key as base64 (RFC 4648) | `String` |
| `Crypto::public_key_fingerprint()` | Get first 8 chars of public key (for logging) | `String` |
| `Crypto::seed_base64()` | Export seed as base64 (for `VIBETEA_PRIVATE_KEY` and `export-key`) | `String` |
| `Crypto::sign(message)` | Sign message, return base64 signature | `String` |
| `Crypto::sign_raw(message)` | Sign message, return raw 64-byte signature | `[u8; 64]` |

**Key Loading Behavior:**
- `load_with_fallback()` used in `monitor/src/main.rs` at startup (see lines 222-226)
- `load()` used in `run_export_key()` for filesystem-only export (see lines 189-193)
- Environment variable `VIBETEA_PRIVATE_KEY` contains base64-encoded 32-byte Ed25519 seed
- Whitespace trimming applied before base64 decoding
- If env var set but invalid: error immediately (no fallback to file)
- If env var not set: load from `VIBETEA_KEY_PATH/key.priv` (default `~/.vibetea/key.priv`)
- Returns `KeySource` enum indicating origin (for logging)

**Memory Safety:**
- All intermediate key buffers zeroed via `zeroize` crate
- Seed arrays zeroed immediately after `SigningKey` creation
- Error paths also zero buffers before returning errors
- Marked with FR-020 comments for security audit

**Phase 4 Addition: export-key Command**
- Lines 101-109: CLI definition (ExportKey struct)
- Lines 180-202: Implementation (run_export_key function)
- Supports `--path` argument for custom key directory
- Falls back to `VIBETEA_KEY_PATH` or `~/.vibetea` if not specified
- Outputs only base64 seed to stdout (no diagnostics)
- All errors go to stderr
- Exit code 0 on success, 1 on configuration error

### `server/src/` - Server Component

| File | Purpose | Key Types |
|------|---------|-----------|
| `main.rs` | Startup, logging, graceful shutdown, signal handling | — |
| `routes.rs` | HTTP handlers + middleware, `AppState` | `AppState`, route handlers |
| `auth.rs` | Ed25519 sig verification, token validation | `AuthError`, `verify_signature()`, `validate_token()` |
| `broadcast.rs` | Event distribution to WebSocket subscribers | `EventBroadcaster`, `SubscriberFilter` |
| `rate_limit.rs` | Per-source rate limiting with TTL cleanup | `RateLimiter`, `RateLimitResult` |
| `config.rs` | Load from env: `VIBETEA_PUBLIC_KEYS`, `VIBETEA_SUBSCRIBER_TOKEN` | `Config` |
| `types.rs` | Event schema (shared with monitor) | `Event`, `EventPayload`, `EventType` |
| `error.rs` | Server error types | `ServerError`, `ApiError` |
| `session.rs` (Phase 2) | Session token store for user authentication | `SessionStore`, `Session`, `SessionStoreConfig`, `SessionError` |
| `supabase.rs` (Phase 2) | Supabase client for JWT validation + key distribution | `SupabaseClient`, `SupabaseError`, `PublicKey` |

### Phase 2 Server Components

#### `server/src/session.rs`

In-memory session token store for managing authenticated user sessions:

| Type | Purpose | Key Methods |
|------|---------|------------|
| `SessionStore` | Token store with TTL management | `new()`, `create_session()`, `validate_session()`, `extend_session()` |
| `SessionStoreConfig` | Configuration (capacity, TTL, grace period) | default(), custom fields |
| `Session` | Session metadata (user_id, email, expires_at) | struct fields |
| `SessionError` | Error types for session operations | `AtCapacity`, `NotFound`, `InvalidToken` |

**Configuration**:
- `max_capacity`: 10,000 sessions (configurable, FR-022)
- `default_ttl`: 5 minutes (FR-008)
- `grace_period`: 30 seconds (FR-024)

**Token Format**:
- 32 bytes random (FR-021)
- Base64-url encoded without padding (43 characters, FR-021)
- 5-minute expiration (FR-008)

#### `server/src/supabase.rs`

HTTP client for Supabase services (JWT validation + public key distribution):

| Type | Purpose | Key Methods |
|------|---------|------------|
| `SupabaseClient` | HTTP client for Supabase APIs | `new()`, `validate_jwt()`, `fetch_public_keys()` |
| `SupabaseError` | Error types for Supabase operations | `Unauthorized`, `Timeout`, `Unavailable`, `InvalidResponse`, `InvalidConfig` |
| `PublicKey` | Database public key record | `source_id: String`, `public_key: String` |

**Configuration**:
- `REQUEST_TIMEOUT`: 5 seconds (FR-010)
- `MAX_RETRY_ATTEMPTS`: 5 (with exponential backoff)
- `BASE_BACKOFF_MS`: 100ms
- `MAX_BACKOFF_MS`: 10 seconds
- `MAX_JITTER_MS`: ±100ms random jitter

**Key Methods**:
- `validate_jwt()`: Remote validation via `GET /auth/v1/user` (FR-002)
- `fetch_public_keys()`: Get keys from edge function (30-second refresh, FR-016)

### `client/src/` - Client Component

| File | Purpose | Key Types |
|------|---------|-----------|
| `App.tsx` | Root layout, token form, conditional rendering | `App` component |
| `main.tsx` | ReactDOM.createRoot() | — |
| `components/ConnectionStatus.tsx` | Status badge (connecting/connected/disconnected) | `ConnectionStatus` component |
| `components/TokenForm.tsx` | Input for auth token, localStorage persistence | `TokenForm` component |
| `components/EventStream.tsx` | Virtualized list of events with filtering | `EventStream` component |
| `components/SessionOverview.tsx` | Table of active sessions with stats | `SessionOverview` component |
| `components/Heatmap.tsx` | Activity heatmap binned by time | `Heatmap` component |
| `hooks/useWebSocket.ts` | WebSocket lifecycle, reconnection with backoff | `useWebSocket()` hook |
| `hooks/useEventStore.ts` | Zustand store, event buffer, session state, filters | `useEventStore()` hook |
| `hooks/useSessionTimeouts.ts` | Session state machine (Active → Inactive → Ended) | `useSessionTimeouts()` hook |
| `types/events.ts` | TypeScript interfaces (VibeteaEvent, Session, etc.) | `VibeteaEvent`, `Session` |
| `utils/formatting.ts` | Date/time/event type formatting | `formatTimestamp()`, `formatEventType()` |
| `__tests__/` | Vitest unit + integration tests | — |

### `supabase/` - Supabase Backend Infrastructure (Phase 2)

| Location | Purpose | Key Content |
|----------|---------|-----------|
| `supabase/functions/public-keys/index.ts` | Edge function for public key distribution | Deno function, GET endpoint, JSON response |
| `supabase/migrations/001_public_keys.sql` | Database schema for monitor public keys | `monitor_public_keys` table, constraints, triggers, permissions |

#### Supabase Functions

**`supabase/functions/public-keys/index.ts`**:
- **Endpoint**: `GET /functions/v1/public-keys`
- **Language**: Deno/TypeScript
- **Purpose**: Returns all monitor public keys from database
- **Response Format**: JSON with `{ keys: [{ source_id, public_key }] }`
- **Features**:
  - CORS support for cross-origin requests
  - 10-second cache control header (reduces database load)
  - No authentication required (FR-015)
  - Error handling for database failures
- **Integration**: Called by server every 30 seconds (FR-016)

#### Supabase Migrations

**`supabase/migrations/001_public_keys.sql`**:
- **Table Name**: `monitor_public_keys`
- **Columns**:
  - `source_id` (TEXT PRIMARY KEY): Unique monitor identifier
  - `public_key` (TEXT NOT NULL): Base64-encoded Ed25519 public key (44 characters)
  - `description` (TEXT): Optional monitor description
  - `created_at` (TIMESTAMPTZ): Record creation timestamp
  - `updated_at` (TIMESTAMPTZ): Last update timestamp (auto-maintained)
- **Constraints**: Non-empty source_id and public_key
- **Index**: On source_id for fast lookups
- **Permissions**: SELECT granted to anon role (for edge function access)
- **Trigger**: Auto-updates `updated_at` on modifications

### `.github/actions/vibetea-monitor/` - GitHub Actions Composite Action (Phase 6)

| File | Purpose | Configuration |
|------|---------|----------------|
| `action.yml` | Action metadata, inputs, outputs, steps | Defines monitor download, env setup, startup logic |

**action.yml Details:**
- **Name**: "VibeTea Monitor"
- **Description**: Start VibeTea monitor to track Claude Code events during GitHub Actions workflows
- **Branding**: Icon "activity", color "green"
- **Inputs** (see lines 24-46):
  - `server-url` (required): VibeTea server URL
  - `private-key` (required): Base64-encoded Ed25519 private key
  - `source-id` (optional): Custom event source identifier
  - `version` (optional): Monitor version (default: "latest")
  - `shutdown-timeout` (optional): Graceful shutdown timeout (default: "5" seconds)
- **Outputs** (see lines 48-55):
  - `monitor-pid`: Process ID from `steps.start-monitor.outputs.pid`
  - `monitor-started`: Success flag from `steps.start-monitor.outputs.started`
- **Steps** (see lines 59-167):
  - Step 1 (lines 61-87): Download VibeTea Monitor binary via curl
  - Step 2 (lines 90-144): Start monitor in background with environment variables
  - Step 3 (lines 150-166): Save cleanup configuration and document graceful shutdown pattern

**Key Design Features:**
- Non-blocking: Gracefully skips if binary download fails (workflow continues)
- Composable: Single action can be used in multiple workflows
- Idempotent: Outputs indicate success/failure for conditional steps
- Self-documenting: Prints cleanup instructions for manual SIGTERM handling

### `.github/workflows/` - GitHub Actions Integration (Phase 5 & 6)

| File | Purpose | Usage | Added In |
|------|---------|-------|----------|
| `ci.yml` | Primary CI workflow | Runs on push/PR, tests + lint + build | Phase 1 |
| `ci-with-monitor.yml` | Example workflow with manual monitor setup | Template for tracking Claude Code events in CI | Phase 5 |

**ci.yml Details:**
- Lines 1-11: Workflow metadata and triggers
- Lines 14-51: Rust job (tests with `--test-threads=1` for env var safety)
- Lines 53-100: Client job (TypeScript tests, lint, format, build)

**ci-with-monitor.yml Details (Phase 5):**
- Lines 16-24: Workflow trigger (manual via `workflow_dispatch`)
- Lines 34-39: Environment variables (private key, server URL, source ID)
- Lines 46-57: Download VibeTea monitor binary from GitHub releases
- Lines 60-70: Start monitor in background before CI steps
- Lines 91-101: CI steps (formatting, linting, tests, build)
- Lines 105-113: Graceful shutdown with SIGTERM

**Using the Composite Action (Phase 6):**
Instead of manual binary download, workflows can use:
```yaml
- uses: aaronbassett/VibeTea/.github/actions/vibetea-monitor@main
  with:
    server-url: ${{ secrets.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    source-id: "github-${{ github.repository }}-${{ github.run_id }}"
    version: "latest"
```

## Module Boundaries

### Monitor Module

Self-contained CLI with these responsibilities:
1. **Launch** TUI by default (interactive mode)
2. **Watch** files via `FileWatcher` (headless `run` mode)
3. **Parse** JSONL via `SessionParser`
4. **Filter** events via `PrivacyPipeline`
5. **Sign** events via `Crypto` (with dual-source key loading and export)
6. **Send** to server via `Sender`
7. **Export** keys via `export-key` command

No cross-dependencies with Server or Client.

```
monitor/src/main.rs
├── Command::Tui → run_tui()
│   ├── tui/mod.rs (TUI lifecycle)
│   │   └── widgets/ (UI components)
├── Command::Init → run_init()
├── Command::ExportKey → run_export_key()
└── Command::Run → run_monitor()
    ├── config.rs (load env)
    ├── crypto.rs (load keys from env var OR file, track KeySource)
    ├── watcher.rs → sender.rs
    │   ↓
    ├── parser.rs → privacy.rs
    │   ↓
    └── sender.rs (HTTP, retry, buffering)
        ├── crypto.rs (sign events)
        └── types.rs (Event schema)
```

### Server Module

Central hub with these responsibilities:
1. **Route** HTTP requests to handlers
2. **Authenticate** monitors (verify signatures)
3. **Validate** tokens for WebSocket clients
4. **Broadcast** events to subscribers
5. **Rate limit** per-source
6. **(Phase 2) Manage** session tokens for authenticated clients
7. **(Phase 2) Integrate** with Supabase for JWT validation and key distribution

No direct dependencies on Monitor or Client implementation.

```
server/src/main.rs
├── config.rs (load env)
├── routes.rs (HTTP handlers)
│   ├── auth.rs (verify signatures, validate tokens)
│   ├── broadcast.rs (WebSocket distribution)
│   ├── rate_limit.rs (per-source rate limiting)
│   ├── session.rs (Phase 2, session token management)
│   └── supabase.rs (Phase 2, JWT validation + key distribution)
└── types.rs (Event schema)
```

### Client Module

React SPA with these responsibilities:
1. **Connect** to server via WebSocket
2. **Manage** application state (Zustand)
3. **Display** events, sessions, heatmap
4. **Filter** by session/time range
5. **Persist** authentication token

No back-end dependencies (except server WebSocket).

```
client/src/App.tsx (root)
├── hooks/
│   ├── useWebSocket.ts (WebSocket, reconnect)
│   ├── useEventStore.ts (Zustand state)
│   └── useSessionTimeouts.ts (session state machine)
├── components/
│   ├── TokenForm.tsx (auth)
│   ├── ConnectionStatus.tsx (status badge)
│   ├── EventStream.tsx (virtualized list)
│   ├── SessionOverview.tsx (table)
│   └── Heatmap.tsx (visualization)
└── types/events.ts (TypeScript interfaces)
```

### Supabase Backend Module (Phase 2)

Provides backend services for authentication, session management, and key distribution:

```
supabase/
├── functions/
│   └── public-keys/
│       └── index.ts (Deno edge function)
│           ├── GET /functions/v1/public-keys
│           └── Query monitor_public_keys table
└── migrations/
    └── 001_public_keys.sql
        ├── CREATE TABLE monitor_public_keys
        ├── Constraints, indexes, triggers
        └── Grant SELECT to anon role
```

### GitHub Actions Composite Action Module (Phase 6)

Thin wrapper around monitor binary with these responsibilities:
1. **Download** monitor binary from GitHub releases
2. **Configure** environment variables
3. **Start** monitor in background
4. **Report** status and process ID

No dependencies on source code (binary-only).

```
.github/actions/vibetea-monitor/action.yml
├── Inputs: server-url, private-key, source-id, version, shutdown-timeout
├── Step 1: Download binary (curl)
├── Step 2: Start monitor with env vars
├── Step 3: Document cleanup pattern
└── Outputs: monitor-pid, monitor-started
```

## Where to Add New Code

| If you're adding... | Put it in... | Example |
|---------------------|--------------|---------|
| **New Monitor command** | `monitor/src/main.rs` (add to `Command` enum) | `Command::Status` |
| **New Monitor feature** | `monitor/src/<feature>.rs` (new module) | `monitor/src/compression.rs` |
| **New key loading method** | `monitor/src/crypto.rs` (add method to `Crypto`) | `Crypto::load_from_stdin()` |
| **New TUI widget** | `monitor/src/tui/widgets/<widget>.rs` (new module) | `monitor/src/tui/widgets/popup.rs` |
| **New TUI feature** | `monitor/src/tui/mod.rs` or new module | Event handlers, state management |
| **New Server endpoint** | `server/src/routes.rs` (add route handler) | `POST /events/:id/ack` |
| **New Server middleware** | `server/src/routes.rs` or `server/src/` (new module) | `server/src/middleware.rs` |
| **New Supabase function** (Phase 2) | `supabase/functions/<function-name>/` (new directory) | `supabase/functions/webhooks/index.ts` |
| **New database migration** (Phase 2) | `supabase/migrations/<number>_<name>.sql` (new file) | `supabase/migrations/002_sessions.sql` |
| **New event type** | `server/src/types.rs` + `monitor/src/types.rs` (sync both) | New `EventPayload` variant |
| **New Client component** | `client/src/components/` | `client/src/components/EventDetail.tsx` |
| **New Client hook** | `client/src/hooks/` | `client/src/hooks/useFilters.ts` |
| **New Client page** | `client/src/pages/` (if routing added) | `client/src/pages/Analytics.tsx` |
| **GitHub Actions workflow** | `.github/workflows/` (copy ci.yml as template) | `.github/workflows/release.yml` |
| **GitHub Actions composite action** | `.github/actions/<action-name>/` (new directory) | `.github/actions/notify-slack/action.yml` |
| **Shared utilities** | Monitor: `monitor/src/utils/` (if created), Server: `server/src/utils/`, Client: `client/src/utils/` | `format_`, `validate_` |
| **Tests** | Colocate with source: `file.rs` → `file_test.rs` (Rust), `file.ts` → `__tests__/file.test.ts` (TS) | — |

## Import Paths & Module Organization

### Monitor/Server (Rust)

**Convention**: Use fully qualified names from crate root via `use` statements.

```rust
// In monitor/src/main.rs
use vibetea_monitor::config::Config;
use vibetea_monitor::crypto::{Crypto, KeySource};
use vibetea_monitor::watcher::FileWatcher;
use vibetea_monitor::sender::Sender;
use vibetea_monitor::types::Event;
use vibetea_monitor::tui::Tui;

// In monitor/src/tui/mod.rs
use vibetea_monitor::tui::widgets::{SizeWarningWidget, check_terminal_size};

// In server/src/routes.rs
use vibetea_server::auth::verify_signature;
use vibetea_server::broadcast::EventBroadcaster;
use vibetea_server::config::Config;
use vibetea_server::types::Event;

// Phase 2: In server/src/routes.rs
use vibetea_server::session::SessionStore;
use vibetea_server::supabase::SupabaseClient;
```

**Modules**:
- `monitor/src/lib.rs` re-exports public API
- `server/src/lib.rs` re-exports public API
- Internal modules use relative `use` statements

### Client (TypeScript)

**Convention**: Absolute paths from `src/` root via `tsconfig.json` alias or relative imports.

```typescript
// In client/src/App.tsx
import { useWebSocket } from './hooks/useWebSocket';
import { useEventStore } from './hooks/useEventStore';
import type { VibeteaEvent } from './types/events';

// In client/src/components/EventStream.tsx
import { useEventStore } from '../hooks/useEventStore';
import type { Session } from '../types/events';
```

**Conventions**:
- Components: PascalCase (e.g., `EventStream.tsx`)
- Hooks: camelCase starting with `use` (e.g., `useWebSocket.ts`)
- Utils: camelCase (e.g., `formatting.ts`)
- Types: camelCase (e.g., `events.ts`)

### GitHub Actions (YAML)

**Convention**: Single file per action (action.yml in directory)

```yaml
# .github/actions/<action-name>/action.yml
name: 'Action Name'
description: 'Action description'
inputs:
  input-name:
    description: 'Input description'
    required: true
outputs:
  output-name:
    description: 'Output description'
    value: ${{ steps.<step-id>.outputs.<output-field> }}
runs:
  using: 'composite'
  steps:
    - run: echo "Hello"
      shell: bash
```

## Entry Points

| Component | File | Launch Command |
|-----------|------|-----------------|
| **Monitor (TUI, default)** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor` or `vibetea-monitor` |
| **Monitor (TUI, explicit)** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor -- tui` or `vibetea-monitor tui` |
| **Monitor (init)** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor -- init` |
| **Monitor (export-key)** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor -- export-key` |
| **Monitor (run, headless)** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor -- run` |
| **Server** | `server/src/main.rs` | `cargo run -p vibetea-server` |
| **Client** | `client/src/main.tsx` | `npm run dev` (from `client/`) |
| **GitHub Actions** | `.github/actions/vibetea-monitor/action.yml` | `uses: aaronbassett/VibeTea/.github/actions/vibetea-monitor@main` |

## Generated/Auto-Configured Files

Files that are auto-generated or should not be manually edited:

| Location | Generator | Regenerate Command |
|----------|-----------|-------------------|
| `Cargo.lock` | Cargo | `cargo lock` (auto-managed) |
| `target/` | Rust compiler | `cargo build` |
| `client/dist/` | Vite | `npm run build` |
| `client/node_modules/` | pnpm | `pnpm install` |

## Naming Conventions

### Rust Modules and Types

| Category | Pattern | Example |
|----------|---------|---------|
| Module names | `snake_case` | `parser.rs`, `privacy.rs`, `size_warning.rs`, `session.rs`, `supabase.rs` |
| Type names | `PascalCase` | `Event`, `ParsedEvent`, `EventPayload`, `SizeWarningWidget`, `SessionStore`, `SupabaseClient` |
| Function names | `snake_case` | `verify_signature()`, `calculate_backoff()`, `check_terminal_size()`, `validate_jwt()` |
| Constant names | `UPPER_SNAKE_CASE` | `MAX_BODY_SIZE`, `EVENT_ID_PREFIX`, `MIN_TERMINAL_WIDTH`, `REQUEST_TIMEOUT` |
| Test functions | `#[test]` or `_test.rs` suffix | `privacy_test.rs`, `env_key_test.rs` |
| Enum variants | `PascalCase` | `KeySource::EnvironmentVariable`, `KeySource::File`, `SessionError::AtCapacity` |

### TypeScript Components and Functions

| Category | Pattern | Example |
|----------|---------|---------|
| Component files | `PascalCase.tsx` | `EventStream.tsx`, `TokenForm.tsx` |
| Hook files | `camelCase.ts` | `useWebSocket.ts`, `useEventStore.ts` |
| Utility files | `camelCase.ts` | `formatting.ts` |
| Type files | `camelCase.ts` | `events.ts` |
| Constants | `UPPER_SNAKE_CASE` | `TOKEN_STORAGE_KEY`, `MAX_BACKOFF_MS` |
| Test files | `__tests__/{name}.test.ts` | `__tests__/formatting.test.ts` |

### YAML (GitHub Actions)

| Category | Pattern | Example |
|----------|---------|---------|
| Workflow names | `kebab-case` | `ci.yml`, `ci-with-monitor.yml` |
| Job names | `kebab-case` | `rust-tests`, `client-tests` |
| Action names | `PascalCase` | `Install Rust toolchain` |
| Step IDs | `kebab-case` | `download-binary`, `start-monitor` |
| Input names | `kebab-case` | `server-url`, `private-key` |
| Output names | `kebab-case` | `monitor-pid`, `monitor-started` |

### SQL (Supabase Migrations)

| Category | Pattern | Example |
|----------|---------|---------|
| Table names | `snake_case` | `monitor_public_keys` |
| Column names | `snake_case` | `source_id`, `public_key`, `created_at` |
| Function names | `snake_case` | `update_updated_at_column()` |
| Index names | `idx_{table}_{columns}` | `idx_monitor_public_keys_source_id` |
| Constraint names | `{table}_{constraint_type}` | `public_key_not_empty` |

## Dependency Boundaries (Import Rules)

### Monitor

```
✓ CAN import:     types, config, crypto, watcher, parser, privacy, sender, error, tui
✓ CAN import:     std, tokio, serde, ed25519-dalek, notify, reqwest, zeroize, ratatui
✗ CANNOT import:  server modules, client code
```

### Server

```
✓ CAN import:     types, config, auth, broadcast, rate_limit, error, routes, session, supabase
✓ CAN import:     std, tokio, axum, serde, ed25519-dalek, subtle, reqwest
✗ CANNOT import:  monitor modules, client code
```

### Client

```
✓ CAN import:     components, hooks, types, utils, React, Zustand, third-party UI libs
✗ CANNOT import:  monitor code, server code (except via HTTP/WebSocket)
```

### Supabase

```
✓ CAN import:     @supabase/supabase-js (in edge functions)
✓ CAN access:     PostgreSQL database, environment variables
✗ CANNOT import:  monitor code, server code, client code
```

### GitHub Actions Composite Action

```
✓ CAN use:       bash, curl, standard CLI tools, GitHub context variables
✗ CANNOT depend: source code, runtime binaries (except downloaded releases)
```

## Test Organization

### Monitor Tests

Located in `monitor/tests/` with `serial_test` crate for environment variable safety:

| File | Purpose | Key Pattern | Added In |
|------|---------|-------------|----------|
| `env_key_test.rs` | Environment variable key loading (FR-001 through FR-028) | `#[test] #[serial]` | Phase 3 |
| `key_export_test.rs` | export-key command integration tests (FR-003, FR-023, FR-026, FR-027, FR-028) | `#[test] #[serial]` | Phase 4 |
| `privacy_test.rs` | Privacy filtering validation | — | Phase 1 |
| `sender_recovery_test.rs` | Retry logic and buffering | — | Phase 1 |

**Test Coverage by Phase 4:**
- `roundtrip_generate_export_command_import_sign_verify()` - Full round-trip via export-key
- `roundtrip_export_command_signatures_are_identical()` - Ed25519 determinism verification
- `export_key_output_format_base64_with_single_newline()` - Output format validation
- `export_key_output_is_valid_base64_32_bytes()` - Base64 decoding verification
- `export_key_diagnostics_go_to_stderr()` - Separation of concerns (stdout/stderr)
- `export_key_error_messages_go_to_stderr()` - Error output routing
- `export_key_exit_code_success()` - Exit code 0 on success
- `export_key_exit_code_missing_key_file()` - Exit code 1 on missing key
- `export_key_exit_code_nonexistent_path()` - Exit code 1 on invalid path
- `export_key_handles_path_with_spaces()` - Edge case support
- `export_key_suitable_for_piping()` - CI/CD integration support
- `export_key_reads_from_key_priv_file()` - Correct file reading

**Important**: Tests modifying environment variables MUST use `#[serial]` from `serial_test` crate or run with `cargo test --workspace --test-threads=1` to prevent interference.

### Server Tests

Located in `server/tests/`:

| File | Purpose |
|------|---------|
| `unsafe_mode_test.rs` | Auth bypass mode validation |

### Client Tests

Located in `client/src/__tests__/`:

| File | Purpose |
|------|---------|
| `App.test.tsx` | Integration tests |
| `events.test.ts` | Event parsing/filtering |
| `formatting.test.ts` | Utility function tests |

---

*This document shows WHERE code lives. Consult ARCHITECTURE.md for HOW the system is organized.*
