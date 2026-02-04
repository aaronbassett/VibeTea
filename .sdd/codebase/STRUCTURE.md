# Project Structure

> **Purpose**: Document directory layout, module boundaries, and where to add new code.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Directory Layout

```
VibeTea/
├── monitor/                    # Rust CLI for watching Claude Code sessions
│   ├── src/
│   │   ├── main.rs            # Entry point, CLI commands (init, run)
│   │   ├── lib.rs             # Module exports
│   │   ├── config.rs          # Environment configuration loading
│   │   ├── watcher.rs         # File system watcher (inotify/FSEvents/ReadDirectoryChangesW)
│   │   ├── parser.rs          # Claude Code JSONL parsing
│   │   ├── privacy.rs         # Event payload sanitization
│   │   ├── crypto.rs          # Ed25519 keypair generation/management
│   │   ├── sender.rs          # HTTP client with retry and buffering
│   │   ├── types.rs           # Event type definitions
│   │   ├── error.rs           # Error types
│   │   ├── trackers/          # Enhanced tracking modules (Phase 4-8)
│   │   │   ├── mod.rs         # Tracker module exports
│   │   │   ├── agent_tracker.rs     # Task tool agent spawn detection
│   │   │   ├── skill_tracker.rs     # Skill/slash command invocation tracking (Phase 5)
│   │   │   ├── stats_tracker.rs     # Token usage and session metrics (Phase 8)
│   │   │   ├── todo_tracker.rs      # Todo list progress tracking (Phase 6)
│   │   │   └── file_history_tracker.rs # File edit tracking (Phase 8+)
│   │   └── utils/             # Shared utilities (debouncing, tokenization, etc.)
│   │       ├── mod.rs         # Utilities exports
│   │       ├── debounce.rs    # Event debouncing for coalescing rapid changes
│   │       ├── session_filename.rs # Utility for parsing session filenames
│   │       └── tokenize.rs    # Skill name extraction (Phase 5)
│   ├── tests/
│   │   ├── privacy_test.rs    # Privacy filtering tests
│   │   └── sender_recovery_test.rs  # Retry logic tests
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
│   │   └── error.rs           # Error types
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
| `main.rs` | CLI entry (init/run commands), signal handling, tracker initialization | `Cli`, `Command` |
| `config.rs` | Load from env vars: `VIBETEA_*` | `Config` |
| `watcher.rs` | inotify/FSEvents for `~/.claude/projects/**/*.jsonl` and `~/.claude/history.jsonl` | `FileWatcher`, `WatchEvent` |
| `parser.rs` | Parse JSONL, extract Session/Activity/Tool/Agent events | `SessionParser`, `ParsedEvent`, `ParsedEventKind` |
| `privacy.rs` | Remove code, prompts, sensitive data | `PrivacyPipeline`, `PrivacyConfig` |
| `crypto.rs` | Ed25519 keypair (generate, load, save) | `Crypto` |
| `sender.rs` | HTTP POST to server with retry/buffering | `Sender`, `SenderConfig`, `RetryPolicy` |
| `types.rs` | Event schema (shared with server) | `Event`, `EventPayload`, `EventType`, `AgentSpawnEvent`, `SkillInvocationEvent`, `TokenUsageEvent`, `SessionMetricsEvent`, `TodoProgressEvent` |
| `error.rs` | Error types | `MonitorError`, custom errors |
| `trackers/mod.rs` | Tracker module organization | Exports `agent_tracker`, `skill_tracker`, `stats_tracker`, `todo_tracker`, `file_history_tracker` |
| `trackers/agent_tracker.rs` | Task tool agent spawn parsing | `TaskToolInput`, `parse_task_tool_use()`, `try_extract_agent_spawn()` |
| `trackers/skill_tracker.rs` (Phase 5) | Skill/slash command parsing from history.jsonl | `SkillTracker`, `parse_history_entry()`, `create_skill_invocation_event()` |
| `trackers/stats_tracker.rs` (Phase 8) | Token usage and session metrics from stats-cache.json | `StatsTracker`, `StatsCache`, `ModelTokens`, `StatsEvent`, `TokenUsageEvent`, `SessionMetricsEvent` |
| `trackers/todo_tracker.rs` (Phase 6) | Todo list progress and abandonment detection | `TodoTracker`, `TodoEntry`, `TodoStatus`, `TodoProgressEvent`, `parse_todo_file()`, `count_todo_statuses()`, `is_abandoned()` |
| `trackers/file_history_tracker.rs` (Phase 8+) | File edit history tracking | `FileHistoryTracker`, `FileChangeEvent` |
| `utils/mod.rs` | Utility module exports | Exports `debounce`, `tokenize`, `session_filename` |
| `utils/debounce.rs` | Event debouncing to coalesce rapid changes | `Debouncer`, `DebouncerError` |
| `utils/session_filename.rs` | Parsing session identifiers from filenames | `parse_todo_filename()`, `parse_project_filename()` |
| `utils/tokenize.rs` (Phase 5) | Skill name extraction | `extract_skill_name()` |

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

## Module Boundaries

### Monitor Module

Self-contained CLI with these responsibilities:
1. **Watch** files via `FileWatcher` (both session JSONL and history.jsonl and todos/ directory and stats-cache.json)
2. **Parse** JSONL via `SessionParser`
3. **Extract enhanced events** via `trackers` (agent spawns, skill invocations, token usage, session metrics, todo progress, file changes)
4. **Filter** events via `PrivacyPipeline`
5. **Sign** events via `Crypto`
6. **Send** to server via `Sender`

No cross-dependencies with Server or Client.

```
monitor/src/main.rs
├── config.rs (load env)
├── watcher.rs → sender.rs
│   ├─ session JSONL files
│   ├─ history.jsonl (Phase 5)
│   ├─ stats-cache.json (Phase 8)
│   ├─ todos/ directory (Phase 6)
│   └─ project files (Phase 8+)
│   ↓
├── parser.rs → trackers/
│   ├→ agent_tracker.rs (detect Task tool_use)
│   └→ privacy.rs
│   ↓
├── trackers/skill_tracker.rs (Phase 5)
│   ├→ parse_history_entry()
│   ├→ extract_skill_name() (from utils/tokenize.rs)
│   └→ create_skill_invocation_event()
│   ↓
├── trackers/stats_tracker.rs (Phase 8)
│   ├→ read_stats_with_retry()
│   ├→ emit_stats_events() [emits SessionMetricsEvent + TokenUsageEvent for each model]
│   └→ uses utils/debounce.rs for 200ms debouncing
│   ↓
├── trackers/todo_tracker.rs (Phase 6)
│   ├→ parse_todo_file()
│   ├→ count_todo_statuses()
│   ├→ is_abandoned()
│   └→ mark_session_ended() (integration with session summary events)
│   ↓
├── trackers/file_history_tracker.rs (Phase 8+)
│   ├→ compute_line_deltas()
│   └→ hash_file_path() (privacy)
│   ↓
├── sender.rs (HTTP, retry, buffering)
│   ├── crypto.rs (sign events)
│   └── types.rs (Event schema)
```

### Server Module

Central hub with these responsibilities:
1. **Route** HTTP requests to handlers
2. **Authenticate** monitors (verify signatures)
3. **Validate** tokens for WebSocket clients
4. **Broadcast** events to subscribers
5. **Rate limit** per-source

No direct dependencies on Monitor or Client implementation.

```
server/src/main.rs
├── config.rs (load env)
├── routes.rs (HTTP handlers)
│   ├── auth.rs (verify signatures, validate tokens)
│   ├── broadcast.rs (WebSocket distribution)
│   └── rate_limit.rs (per-source rate limiting)
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

## Where to Add New Code

| If you're adding... | Put it in... | Example |
|---------------------|--------------|---------|
| **New Monitor command** | `monitor/src/main.rs` (add to `Command` enum) | `Command::Status` |
| **New Monitor feature** | `monitor/src/<feature>.rs` (new module) | `monitor/src/compression.rs` |
| **New tracker module** | `monitor/src/trackers/<tracker>.rs` (new module) + export in `trackers/mod.rs` | `monitor/src/trackers/project_tracker.rs` for project tracking |
| **New event extraction** | `monitor/src/trackers/` or enhance `monitor/src/parser.rs` | Add new `ParsedEventKind` variant |
| **New utility function** | `monitor/src/utils/<category>.rs` for Rust utilities | `monitor/src/utils/hashing.rs` |
| **New Server endpoint** | `server/src/routes.rs` (add route handler) | `POST /events/:id/ack` |
| **New Server middleware** | `server/src/routes.rs` or `server/src/` (new module) | `server/src/middleware.rs` |
| **New event type** | `server/src/types.rs` + `monitor/src/types.rs` (sync both) | New `EventPayload` variant |
| **New Client component** | `client/src/components/` | `client/src/components/EventDetail.tsx` |
| **New Client hook** | `client/src/hooks/` | `client/src/hooks/useFilters.ts` |
| **New Client page** | `client/src/pages/` (if routing added) | `client/src/pages/Analytics.tsx` |
| **Shared utilities** | Monitor: `monitor/src/utils/` (if created), Server: `server/src/utils/`, Client: `client/src/utils/` | `format_`, `validate_` |
| **Tests** | Colocate with source: `file.rs` → internal `#[cfg(test)] mod tests`, Server/Monitor integration tests in `tests/` | — |

## Import Paths & Module Organization

### Monitor/Server (Rust)

**Convention**: Use fully qualified names from crate root via `use` statements.

```rust
// In monitor/src/main.rs
use vibetea_monitor::config::Config;
use vibetea_monitor::watcher::FileWatcher;
use vibetea_monitor::sender::Sender;
use vibetea_monitor::types::Event;
use vibetea_monitor::trackers::agent_tracker;
use vibetea_monitor::trackers::skill_tracker::SkillTracker;  // Phase 5
use vibetea_monitor::trackers::stats_tracker::{StatsTracker, StatsEvent};  // Phase 8
use vibetea_monitor::trackers::todo_tracker::TodoTracker;    // Phase 6
use vibetea_monitor::trackers::file_history_tracker::FileHistoryTracker;  // Phase 8+
use vibetea_monitor::utils::tokenize::extract_skill_name;    // Phase 5
use vibetea_monitor::utils::debounce::Debouncer;             // Phase 6

// In server/src/routes.rs
use vibetea_server::auth::verify_signature;
use vibetea_server::broadcast::EventBroadcaster;
use vibetea_server::config::Config;
use vibetea_server::types::Event;
```

**Modules**:
- `monitor/src/lib.rs` re-exports public API
- `server/src/lib.rs` re-exports public API
- `monitor/src/trackers/mod.rs` exports tracker submodules
- `monitor/src/utils/mod.rs` exports utility modules
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

## Entry Points

| Component | File | Launch Command |
|-----------|------|-----------------|
| **Monitor** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor -- run` |
| **Server** | `server/src/main.rs` | `cargo run -p vibetea-server` |
| **Client** | `client/src/main.tsx` | `npm run dev` (from `client/`) |

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
| Module names | `snake_case` | `parser.rs`, `privacy.rs`, `agent_tracker.rs`, `skill_tracker.rs`, `stats_tracker.rs`, `todo_tracker.rs`, `file_history_tracker.rs` |
| Type names | `PascalCase` | `Event`, `ParsedEvent`, `EventPayload`, `AgentSpawnEvent`, `SkillInvocationEvent`, `TokenUsageEvent`, `SessionMetricsEvent`, `TodoProgressEvent`, `FileChangeEvent` |
| Function names | `snake_case` | `verify_signature()`, `calculate_backoff()`, `parse_task_tool_use()`, `extract_skill_name()`, `count_todo_statuses()`, `emit_stats_events()` |
| Constant names | `UPPER_SNAKE_CASE` | `MAX_BODY_SIZE`, `EVENT_ID_PREFIX`, `DEFAULT_CHANNEL_CAPACITY`, `DEFAULT_DEBOUNCE_MS`, `STATS_DEBOUNCE_MS` |
| Test functions | `#[test]` or `_test.rs` suffix | `privacy_test.rs`, `agent_tracker` has internal test module, `todo_tracker` has extensive test module, `stats_tracker` has comprehensive test module |

### TypeScript Components and Functions

| Category | Pattern | Example |
|----------|---------|---------|
| Component files | `PascalCase.tsx` | `EventStream.tsx`, `TokenForm.tsx` |
| Hook files | `camelCase.ts` | `useWebSocket.ts`, `useEventStore.ts` |
| Utility files | `camelCase.ts` | `formatting.ts` |
| Type files | `camelCase.ts` | `events.ts` |
| Constants | `UPPER_SNAKE_CASE` | `TOKEN_STORAGE_KEY`, `MAX_BACKOFF_MS` |
| Test files | `__tests__/{name}.test.ts` | `__tests__/formatting.test.ts` |

## Dependency Boundaries (Import Rules)

### Monitor

```
✓ CAN import:     types, config, crypto, watcher, parser, privacy, sender, trackers, utils, error
✓ CAN import:     std, tokio, serde, ed25519-dalek, notify, reqwest, chrono, directories
✗ CANNOT import:  server modules, client code
```

### Server

```
✓ CAN import:     types, config, auth, broadcast, rate_limit, error, routes
✓ CAN import:     std, tokio, axum, serde, ed25519-dalek, subtle
✗ CANNOT import:  monitor modules, client code
```

### Client

```
✓ CAN import:     components, hooks, types, utils, React, Zustand, third-party UI libs
✗ CANNOT import:  monitor code, server code (except via HTTP/WebSocket)
```

## Enhanced Tracking Subsystem (Phase 4-8)

### Adding a New Tracker Module

To add a new tracker (e.g., `project_tracker` for project tracking):

1. **Create the module**: `monitor/src/trackers/project_tracker.rs`
2. **Export it**: Add `pub mod project_tracker;` to `monitor/src/trackers/mod.rs`
3. **Define extraction functions**: `parse_<source>()`, `create_<event>()`
4. **Integrate with main loop**: Spawn async task in `main.rs` or call from parser
5. **Add event type**: New `EventPayload` and `EventType` variants in `monitor/src/types.rs` and `server/src/types.rs`
6. **Write tests**: Include comprehensive test module in the tracker
7. **Update documentation**: Add section to ARCHITECTURE.md and STRUCTURE.md

Example structure:
```rust
// monitor/src/trackers/project_tracker.rs
use crate::types::ProjectActivityEvent;

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectEntry { ... }

pub fn parse_project_entry(line: &str) -> Result<ProjectEntry, ParseError> { ... }

pub fn create_project_event(...) -> ProjectActivityEvent { ... }

#[cfg(test)]
mod tests { ... }
```

### stats_tracker Module (Phase 8) - Concrete Example

The stats_tracker demonstrates the full pattern for a new tracker with system-wide metrics:

1. **Location**: `monitor/src/trackers/stats_tracker.rs`
2. **Data Source**: `~/.claude/stats-cache.json` (JSON file with global metrics)
3. **File Watching**: Uses `notify` crate for file monitoring with directory watch
4. **Entry Point**: Spawned as async task in `main.rs` via `StatsTracker::new()`
5. **Event Types**: Emits two variants via `StatsEvent` enum:
   - `SessionMetricsEvent`: Global counters (total_sessions, total_messages, total_tool_usage, longest_session)
   - `TokenUsageEvent`: Per-model token usage (model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens)
6. **Helper Types**: Uses `StatsCache` and `ModelTokens` for JSON deserialization
7. **Privacy**: Extracts only aggregated metrics; no code or personal data
8. **Processing**:
   - Debounced at 200ms to coalesce rapid file updates
   - Emits one `SessionMetricsEvent` per file read (global metrics)
   - Emits one `TokenUsageEvent` per model in modelUsage section
9. **Error Handling**: Includes retry logic (up to 3 retries with 100ms delays) for file read failures

Key architectural patterns:
- **File watching**: Non-recursive directory watch, single file matching, filtering
- **Debouncing**: Coalesces rapid writes using `Debouncer<PathBuf, FileChangeEvent>`
- **Dual event emission**: Single `emit_stats_events()` call produces two event types sequentially
- **Retry logic**: Handles file mid-write scenarios with exponential delays
- **Async processing**: Spawned as background task, events sent via channel
- **Privacy preservation**: No code or personal information exposed
- **Testing**: Extensive unit tests covering parsing, event emission, debouncing, retries, and integration scenarios

### todo_tracker Module (Phase 6) - Concrete Example

The todo_tracker demonstrates the full pattern for a new tracker with session lifecycle integration:

1. **Location**: `monitor/src/trackers/todo_tracker.rs`
2. **Data Source**: `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json` (JSON array files)
3. **File Watching**: Uses `notify` crate for directory monitoring
4. **Entry Point**: Spawned as async task in `main.rs` via `TodoTracker::new()`
5. **Event Type**: `TodoProgressEvent` (session_id, completed, in_progress, pending, abandoned)
6. **Helper Utilities**: Uses `parse_todo_filename()` from `utils/session_filename.rs`
7. **Privacy**: Extracts only status counts; task content never transmitted
8. **Processing**:
   - Debounced at 100ms to coalesce rapid status updates
   - Session lifecycle tracking via `mark_session_ended()` for abandonment detection
9. **Session Integration**: Requires coordination with main event loop to detect session summaries

Key architectural patterns:
- **File watching**: Non-recursive directory watch, JSON file detection, filtering
- **Debouncing**: Coalesces rapid writes using `Debouncer<PathBuf, PathBuf>`
- **State management**: Maintains `Arc<RwLock<HashSet<String>>>` of ended sessions
- **Async processing**: Spawned as background task, events sent via channel
- **Privacy preservation**: Lenient parsing for corrupted/partial files, never exposing task content
- **Testing**: Extensive unit tests covering parsing, counting, abandonment, integration scenarios

### Key Design Principles for Trackers

- **Privacy-first**: Extract only metadata, never code or sensitive content
- **Modular**: Each tracker is independent and can be disabled
- **Testable**: Include comprehensive unit tests with realistic file examples
- **Integrated**: Feed results into the same `sender.queue()` for consistency
- **Documented**: Include module doc comments with Format, Privacy, Architecture, Example sections
- **Independent Data Source**: Each tracker watches its own file(s)
- **Async Task Pattern**: Trackers run as concurrent async tasks, not inline in parser
- **Session Aware**: When appropriate, trackers should integrate with session lifecycle events

---

*This document shows WHERE code lives. Consult ARCHITECTURE.md for HOW the system is organized.*
