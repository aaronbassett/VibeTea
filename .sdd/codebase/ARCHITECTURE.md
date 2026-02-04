# Architecture

> **Purpose**: Document system design, patterns, component relationships, and data flow.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

## Architecture Overview

VibeTea is a distributed event aggregation and broadcast system for AI coding assistants with three independent components:

- **Monitor** (Rust CLI) - Watches local Claude Code session files and forwards privacy-filtered events to the server, with enhanced tracking modules for agent spawns, skill invocations, token usage, activity patterns, model distribution, todo progress, file changes, project activity, and other metrics
- **Server** (Rust HTTP API) - Central hub that receives events from monitors and broadcasts them to subscribers via WebSocket
- **Client** (React SPA) - Real-time dashboard displaying aggregated event streams, session activity, usage heatmaps, and analytics

The system follows a **hub-and-spoke architecture** where the Server acts as a central event bus, decoupling multiple Monitor sources from multiple Client consumers. Events flow unidirectionally: Monitors → Server → Clients, with no persistent storage.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Central Server acts as event aggregation point, Monitors feed events, Clients consume |
| **Producer-Consumer** | Monitors are event producers, Clients are event consumers, Server mediates asynchronous delivery |
| **Privacy-First** | Events contain only structural metadata (timestamps, tool names, skill names, file basenames, project paths), never code or sensitive content |
| **Real-time Streaming** | WebSocket-based live event delivery with no message persistence (fire-and-forget) |
| **CI/CD Integration** | Environment variable key loading enables monitor deployment in containerized and GitHub Actions environments |
| **Composite Action** | GitHub Actions workflow integration via reusable composite action for simplified monitor setup |

## Core Components

### Monitor (Source)

- **Purpose**: Watches Claude Code session files in `~/.claude/projects/**/*.jsonl`, `~/.claude/history.jsonl`, `~/.claude/stats-cache.json`, `~/.claude/todos/`, and file history, emitting structured events
- **Location**: `monitor/src/`
- **Key Responsibilities**:
  - File watching via `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows)
  - Privacy-preserving JSONL and JSON parsing (extracts metadata only)
  - Cryptographic signing of events (Ed25519)
  - Event buffering and exponential backoff retry
  - Graceful shutdown with event flushing
  - Key export functionality for GitHub Actions integration
  - Dual-source key loading (environment variable with file fallback)
  - Project session activity tracking (Phase 11)
- **Dependencies**:
  - Monitors depend on **Server** (via HTTP POST to `/events`)
  - Monitors depend on local Claude Code installation (`~/.claude/`)
- **Dependents**: None (source component)
- **Deployment Context**: Can run locally, in Docker, or in GitHub Actions via environment variable key injection

### Server (Hub)

- **Purpose**: Central event aggregation point that validates, authenticates, and broadcasts events to all subscribers
- **Location**: `server/src/`
- **Key Responsibilities**:
  - Receiving and authenticating events from Monitors (Ed25519 signature verification)
  - Rate limiting (per-source basis, configurable)
  - Broadcasting events to WebSocket subscribers
  - Token-based authentication for WebSocket clients
  - Graceful shutdown with timeout for in-flight requests
- **Dependencies**:
  - Broadcasts to **Clients** (via WebSocket `/ws` endpoint)
  - Depends on Monitor-provided public keys for signature verification
- **Dependents**: Clients (consumers)

### Client (Consumer)

- **Purpose**: Real-time dashboard displaying aggregated event stream from the Server with analytics visualizations
- **Location**: `client/src/`
- **Key Responsibilities**:
  - WebSocket connection management with exponential backoff reconnection
  - Event buffering (1000 events max) with FIFO eviction
  - Session state management (Active, Inactive, Ended, Removed)
  - Event filtering (by session ID, time range)
  - Real-time visualization (event list, session overview, heatmap, analytics)
- **Dependencies**:
  - Depends on **Server** (via WebSocket connection to `/ws`)
  - No persistence layer (in-memory Zustand store)
- **Dependents**: None (consumer component)

### GitHub Actions Composite Action (CI/CD Integration - Phase 6)

- **Purpose**: Simplifies VibeTea monitor deployment in GitHub Actions workflows
- **Location**: `.github/actions/vibetea-monitor/action.yml`
- **Key Responsibilities**:
  - Download VibeTea monitor binary from GitHub releases
  - Configure environment variables (private key, server URL, source ID)
  - Start monitor in background before CI steps
  - Provide process ID and status outputs to workflow
  - Documentation for graceful shutdown pattern
- **Dependencies**:
  - Depends on VibeTea GitHub releases (binary artifacts)
  - Requires `curl` on runner for binary download
  - Uses `bash` shell for cross-platform compatibility
- **Inputs**:
  - `server-url` (required): VibeTea server URL
  - `private-key` (required): Base64-encoded Ed25519 private key
  - `source-id` (optional): Custom event source identifier
  - `version` (optional): Monitor version (default: latest)
  - `shutdown-timeout` (optional): Graceful shutdown wait time (default: 5 seconds)
- **Outputs**:
  - `monitor-pid`: Process ID of running monitor
  - `monitor-started`: Whether monitor started successfully
- **Environment Variables Set**:
  - `VIBETEA_PRIVATE_KEY`: From action input
  - `VIBETEA_SERVER_URL`: From action input
  - `VIBETEA_SOURCE_ID`: From action input or auto-generated as `github-{repo}-{run_id}`
  - `VIBETEA_MONITOR_PID`: Process ID saved for cleanup
  - `VIBETEA_SHUTDOWN_TIMEOUT`: For manual cleanup reference

## Data Flow

### Primary Monitor-to-Client Flow

Claude Code → Monitor → Server → Client:
1. JSONL line written to `~/.claude/projects/<uuid>.jsonl` or skill invocation to `~/.claude/history.jsonl` or todo change to `~/.claude/todos/<uuid>-agent-<uuid>.json` or stats update to `~/.claude/stats-cache.json` or session file change in projects directory
2. File watcher detects change via inotify/FSEvents
3. Parser extracts event metadata (no code/prompts)
4. Enhanced trackers extract specialized event types (agent spawns, skill invocations, token usage, activity patterns, model distribution, session metrics, todo progress, file changes, project activity)
5. Privacy pipeline sanitizes payload
6. Sender signs with Ed25519, buffers, and retries on failure
7. POST /events sent to Server with X-Source-ID and X-Signature headers
8. Server verifies signature and rate limit
9. Broadcaster sends event to all WebSocket subscribers
10. Client receives via useWebSocket hook
11. Zustand store adds event (FIFO eviction at 1000 limit)
12. React renders updated event list, session overview, heatmap

### Enhanced Tracking Flow (Phase 4-11)

Six parallel tracking pipelines within Monitor:

**Pipeline 1: Session JSONL Parser (WatchEvent → ParsedEvent)**
```
Session JSONL → FileWatcher
  ↓
WatchEvent (LinesAdded)
  ↓
SessionParser.parse_line()
  ├→ SessionParser (traditional parsing)
  ├→ agent_tracker (detect Task tool_use)
  └→ ParsedEventKind enum variant (including AgentSpawned)
```

**Pipeline 2: Skill Tracker (history.jsonl → SkillInvocationEvent)**
```
history.jsonl → FileWatcher (notify crate)
  ↓
WatchEvent (Create/Modify)
  ↓
SkillTracker.process_file_changes()
  ├→ parse_history_entry() (JSON deserialization)
  ├→ extract_skill_name() (tokenization from display field)
  └→ SkillInvocationEvent
  ↓
Main event loop: sender.queue() + sender.flush()
```

**Pipeline 3: Stats Tracker (stats-cache.json → SessionMetricsEvent + ActivityPatternEvent + ModelDistributionEvent + TokenUsageEvent) (Phase 8-10)**
```
stats-cache.json → StatsTracker
  ↓
WatchEvent (Create/Modify via notify crate)
  ↓
StatsTracker.process_file_changes() [debounced 200ms]
  ├→ read_stats_with_retry() (JSON deserialization with 3 retries)
  ├→ emit_stats_events() [emits ALL event types]
  │  ├→ SessionMetricsEvent (once per file read: total_sessions, total_messages, total_tool_usage, longest_session)
  │  ├→ ActivityPatternEvent (once per file read: hour_counts distribution, Phase 9)
  │  ├→ ModelDistributionEvent (once per file read: model_usage summary, Phase 10)
  │  └→ TokenUsageEvent (once per model: model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens)
  ↓
Main event loop: sender.queue()
```

**Pipeline 4: Todo Tracker (todos/ directory → TodoProgressEvent)**
```
~/.claude/todos/<uuid>-agent-<uuid>.json → FileWatcher (notify crate)
  ↓
WatchEvent (Create/Modify)
  ↓
TodoTracker.process_file_changes()
  ├→ parse_todo_file() (JSON array deserialization)
  ├→ count_todo_statuses() (aggregate task states)
  ├→ is_abandoned() (detect incomplete tasks on session end)
  └→ TodoProgressEvent
  ↓
Main event loop: sender.queue() + sender.flush()
```

**Pipeline 5: File History Tracker (project files → FileChangeEvent)**
```
Project files → FileWatcher (notify crate)
  ↓
WatchEvent (Create/Modify/Delete)
  ↓
FileHistoryTracker.process_file_changes()
  ├→ compute_line_deltas()
  ├→ hash_file_path() (privacy)
  └→ FileChangeEvent
  ↓
Main event loop: sender.queue()
```

**Pipeline 6: Project Tracker (~/.claude/projects/ → ProjectActivityEvent) (Phase 11)**
```
~/.claude/projects/<slug>/<uuid>.jsonl → ProjectTracker (notify crate)
  ↓
WatchEvent (Create/Modify via notify crate, recursive)
  ↓
ProjectTracker.process_file_changes()
  ├→ extract_session_id() (UUID validation from filename)
  ├→ parse_project_slug() (path reconstruction from directory name)
  ├→ check_session_active() (detect summary event in JSONL)
  └→ ProjectActivityEvent (project_path, session_id, is_active)
  ↓
Main event loop: sender.queue() via channel
```

All pipelines feed into the same `sender.queue()` for batching and transmission.

### GitHub Actions Integration Flow (Phase 5 & 6)

GitHub Actions workflow → Monitor (via composite action or manual setup) → Server → Client:
1. **Composite Action Setup (Phase 6)**: `.github/actions/vibetea-monitor` downloads binary and configures environment
   - Action downloads monitor binary from releases (or skips gracefully on network failure)
   - Environment variables configured: `VIBETEA_PRIVATE_KEY`, `VIBETEA_SERVER_URL`, `VIBETEA_SOURCE_ID`
   - Monitor started in background, PID captured and saved to `$GITHUB_ENV`
   - Action returns `monitor-started` and `monitor-pid` outputs for workflow use

2. **Manual Setup (Phase 5)**: Workflow manually downloads and starts monitor
   - `curl` downloads binary from releases
   - Permissions set with `chmod +x`
   - Monitor started in background with `./vibetea-monitor run &`
   - Process ID saved for cleanup

3. **Common Flow**: Both setup methods use same environment variable key loading
   - Monitor loads private key from `VIBETEA_PRIVATE_KEY` environment variable
   - During CI/CD steps (tests, builds, Claude Code operations), monitor captures events
   - Events signed and buffered using env var key
   - Events transmitted to server via HTTP with retry logic
   - Server authenticates using pre-registered public key
   - Clients receive events in real-time dashboard
   - Workflow terminates, monitor receives SIGTERM and flushes remaining events

### Detailed Request/Response Cycle

1. **Event Creation** (Monitor/Parser):
   - JSONL line parsed from `~/.claude/projects/<uuid>.jsonl`
   - `SessionParser` extracts timestamp, tool name, action
   - `agent_tracker` detects Task tool_use and extracts agent metadata
   - `SkillTracker` detects history.jsonl changes and parses skill invocations
   - `StatsTracker` detects stats-cache.json changes and parses session metrics, activity patterns, model distribution, and token usage
   - `TodoTracker` detects todos/ changes and parses todo progress
   - `FileHistoryTracker` detects file changes and tracks line deltas
   - `ProjectTracker` detects projects/ changes and tracks session activity status
   - `PrivacyPipeline` removes sensitive fields (code, prompts, task content)
   - `Event` struct created with unique ID (`evt_` prefix + 20-char suffix)

2. **Event Signing** (Monitor/Sender):
   - Event payload serialized to JSON
   - Ed25519 signature computed over message body
   - Event queued in local buffer (max 1000, FIFO eviction)

3. **Event Transmission** (Monitor/Sender):
   - `POST /events` with headers: `X-Source-ID`, `X-Signature`
   - On 429 (rate limit): parse `Retry-After` header
   - On network failure: exponential backoff (1s → 60s, ±25% jitter)
   - On success: continue flushing buffered events

4. **Event Ingestion** (Server):
   - Extract `X-Source-ID` and `X-Signature` headers
   - Load Monitor's public key from config (`VIBETEA_PUBLIC_KEYS`)
   - Verify Ed25519 signature using `subtle::ConstantTimeEq` (timing-safe)
   - Rate limit check per source_id
   - Broadcast event to all WebSocket subscribers via `tokio::broadcast`

5. **Event Delivery** (Server → Client):
   - WebSocket subscriber receives `BroadcastEvent`
   - Client's `useWebSocket` hook calls `addEvent()` action
   - Zustand store updates event buffer (evicts oldest if > 1000)
   - Session state updated (Active/Inactive/Ended/Removed)
   - React re-renders only affected components (via Zustand selectors)

6. **Visualization** (Client):
   - `EventStream` component renders with virtualized scrolling
   - `SessionOverview` shows active sessions with metadata
   - `Heatmap` displays activity over time bins
   - `ConnectionStatus` shows server connectivity

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|------------|---------------|
| **Monitor** | Observe local activity, preserve privacy, authenticate, track metrics | FileSystem, HTTP client, Crypto, Stats files, history.jsonl, todos/, file history, projects directory | Server internals, other Monitors, network stack details |
| **Server** | Route, authenticate, broadcast, rate limit | All Monitors' public keys, event config, rate limiter state | File system, external APIs, Client implementation details |
| **Client** | Display, interact, filter, manage WebSocket | Server WebSocket, local storage (token), state store | Server internals, other Clients' state, file system |
| **GitHub Actions Composite Action** | Download, configure, launch monitor | GitHub releases, environment variables, bash shell | Monitor source code, Server config, Client code |

## Dependency Rules

- **Monitor → Server**: Only on startup (load server URL from config), then periodically via HTTP POST
- **Server → Monitor**: None (server doesn't initiate contact)
- **Server ↔ Client**: Bidirectional (Client initiates WebSocket, Server sends events, Client sends nothing back)
- **Cross-Monitor**: No inter-Monitor communication (all go through Server)
- **Composite Action → Monitor**: Downloads binary and configures environment (read-only)
- **Persistence**: None at any layer (no database, no cache persistence)

## Key Interfaces & Contracts

| Interface | Purpose | Implementations |
|-----------|---------|-----------------|
| `Event` | Core event struct with type + payload | JSON serialization via `serde` |
| `EventPayload` | Tagged union of event variants | Session, Activity, Tool, Agent, Summary, Error, AgentSpawn, SkillInvocation, TokenUsage, ActivityPattern, ModelDistribution, SessionMetrics, TodoProgress, FileChange, ProjectActivity, etc. |
| `EventType` | Enum discriminator | 15+ variants (Session, Activity, Tool, Agent, Summary, Error, AgentSpawn, SkillInvocation, TokenUsage, SessionMetrics, ActivityPattern, ModelDistribution, TodoProgress, FileChange, ProjectActivity) |
| `ParsedEventKind` | Parser output enum | SessionStarted, Activity, ToolStarted, ToolCompleted, Summary, AgentSpawned |
| `AgentSpawnEvent` | Task tool agent spawn metadata | session_id, agent_type, description, timestamp |
| `SkillInvocationEvent` | Skill/slash command invocation metadata | session_id, skill_name, project, timestamp |
| `TokenUsageEvent` | Per-model token consumption | model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens |
| `SessionMetricsEvent` | Global session statistics (Phase 8) | total_sessions, total_messages, total_tool_usage, longest_session |
| `ActivityPatternEvent` | Hourly activity distribution (Phase 9) | hour_counts: HashMap<String, u64> |
| `ModelDistributionEvent` | Model usage distribution (Phase 10) | model_usage: HashMap<String, TokenUsageSummary> |
| `StatsEvent` | Union of stats tracker events (Phase 8-10) | TokenUsage, SessionMetrics, ActivityPattern, ModelDistribution |
| `TodoProgressEvent` | Todo list progress per session | session_id, completed, in_progress, pending, abandoned |
| `FileChangeEvent` | File edit line deltas | session_id, file_hash, version, lines_added, lines_removed, lines_modified, timestamp |
| `ProjectActivityEvent` | Project session activity status (Phase 11) | project_path, session_id, is_active |
| `TaskToolInput` | Parsed Task tool input | subagent_type, description (prompt excluded for privacy) |
| `AuthError` | Auth failure codes | InvalidSignature, UnknownSource, InvalidToken |
| `RateLimitResult` | Rate limit outcome | Allowed, Blocked (with retry delay) |
| `SubscriberFilter` | Optional event filtering | by_event_type, by_project, by_source |
| `KeySource` | Tracks key origin for logging | EnvironmentVariable, File(PathBuf) |
| `Crypto` | Ed25519 key operations | generate(), load(), save(), sign(), public_key_fingerprint(), seed_base64() |

## Authentication & Authorization

### Monitor Authentication (Source)

- **Mechanism**: Ed25519 signature verification
- **Flow**:
  1. Monitor generates keypair: `vibetea-monitor init`
  2. Public key registered with Server: `VIBETEA_PUBLIC_KEYS=monitor1:base64pubkey`
  3. On `POST /events`, Monitor signs message body with private key
  4. Server verifies signature against pre-registered public key
  5. Invalid signatures rejected with 401 Unauthorized
- **Security**: Uses `ed25519_dalek::VerifyingKey::verify_strict()` (RFC 8032 compliant)
- **Timing Attack Prevention**: `subtle::ConstantTimeEq` for signature comparison
- **Key Tracking**: `KeySource` enum tracks whether keys loaded from file or environment variable (logged at startup for verification)

### Client Authentication (Consumer)

- **Mechanism**: Bearer token (HTTP header)
- **Flow**:
  1. Client obtains token (out-of-band, server-configured)
  2. Token sent in WebSocket upgrade request: `?token=secret`
  3. Server calls `validate_token()` to check token
  4. Invalid/missing tokens rejected with 401 Unauthorized
- **Storage**: Client stores token in localStorage under `vibetea_token` key

## State Management

### Server State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Configuration** | `Arc<Config>` in `AppState` | Immutable, shared across requests |
| **Broadcast Channel** | `EventBroadcaster` (tokio::broadcast) | Multi-producer, multi-consumer, lossy if slow subscribers |
| **Rate Limiter** | `RateLimiter` (Arc<Mutex<HashMap>>) | Per-source tracking with TTL-based cleanup |
| **Uptime** | `Instant` in `AppState` | Initialized at startup for health checks |

### Client State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Connection Status** | Zustand store | Driven by WebSocket events (connecting, connected, disconnected, reconnecting) |
| **Event Buffer** | Zustand store (Vec, max 1000) | FIFO eviction when full, newest first |
| **Sessions** | Zustand store (Map<sessionId, Session>) | Keyed by UUID, state machines (Active → Inactive → Ended → Removed) |
| **Filters** | Zustand store | Session ID filter, time range filter |
| **Authentication Token** | localStorage | Persisted across page reloads |

### Monitor State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Event Buffer** | `VecDeque<Event>` (max configurable) | FIFO eviction, flushed on graceful shutdown |
| **Session Parsers** | `HashMap<PathBuf, SessionParser>` | Keyed by file path, created on first write, removed on file delete |
| **Skill Tracker State** | `SkillTracker` instance | Maintains byte offset for history.jsonl, processes immediately (no debounce) |
| **Stats Tracker State** | `StatsTracker` instance | Watches stats-cache.json with 200ms debounce, emits all four event types (SessionMetrics, ActivityPattern, ModelDistribution, TokenUsage) |
| **Todo Tracker State** | `TodoTracker` instance | Maintains ended sessions set, debounces file changes, processes immediately (Phase 6) |
| **File History Tracker State** | `FileHistoryTracker` instance | Tracks file versions and computes line deltas |
| **Project Tracker State** | `ProjectTracker` instance | Watches projects directory recursively, detects session activity (active/completed), no debouncing needed (Phase 11) |
| **Retry State** | `Sender` internal | Tracks backoff attempt count per send operation |
| **Crypto Keys** | Loaded once at startup | KeySource tracked for logging, indicates origin (env var or file) |

## Cryptographic Key Management

### Key Loading Strategy

The Monitor implements a flexible key loading mechanism with precedence rules:

1. **Environment Variable (`VIBETEA_PRIVATE_KEY`)**
   - Takes absolute precedence when set
   - Must contain base64-encoded 32-byte Ed25519 seed (RFC 4648 standard base64)
   - Whitespace trimmed before decoding (handles newlines, CRLF, spaces)
   - Used via `Crypto::load_from_env()` for direct loading
   - Used via `Crypto::load_with_fallback()` as primary source (no fallback on error)

2. **File-based Key (`~/.vibetea/key.priv`)**
   - Used as fallback when env var not set
   - Contains raw 32-byte seed (binary format)
   - Loaded via `Crypto::load()` or `Crypto::load_with_fallback()`
   - File permissions enforced: `0600` (owner read/write only)

3. **Key Precedence Rules**
   - If `VIBETEA_PRIVATE_KEY` is set: use it (even if file exists and file would error)
   - If `VIBETEA_PRIVATE_KEY` is set but invalid: error immediately (no fallback to file)
   - If `VIBETEA_PRIVATE_KEY` not set: use file at `VIBETEA_KEY_PATH/key.priv`

4. **Runtime Behavior in `run_monitor()`**
   - Calls `Crypto::load_with_fallback(&config.key_path)`
   - Returns tuple `(Crypto, KeySource)` indicating origin
   - Logs key fingerprint and source at INFO level
   - Logs warning if file key exists but env var takes precedence

### Key Export for CI/CD Integration (Phase 4)

The Monitor now supports exporting private keys for GitHub Actions and other CI systems:

- **Command**: `vibetea-monitor export-key [--path <DIR>]`
- **Output**: Outputs base64-encoded seed to stdout, only (no diagnostics)
- **Diagnostics**: All messages (errors, logs) go to stderr
- **Use Case**: `vibetea-monitor export-key | gh secret set VIBETEA_PRIVATE_KEY`
- **Exit Codes**:
  - 0: Success (key exported to stdout)
  - 1: Configuration error (missing key, invalid directory)
  - 2: Runtime error (I/O failures)
- **Security Considerations**:
  - Command reads from filesystem only (no environment variable fallback)
  - Suitable for piping to clipboard or secret management tools
  - No ANSI codes or extra formatting in stdout
  - Clean output enables direct integration with CI/CD systems

### Composite Action Key Loading (Phase 6)

The GitHub Actions composite action simplifies key management:

- **Input Handling**: Action accepts `private-key` input (base64-encoded Ed25519 seed)
- **Environment Variable Setting**: Action sets `VIBETEA_PRIVATE_KEY` env var before starting monitor
- **Precedence**: Monitor uses environment variable key loading (no file fallback needed in CI)
- **Error Handling**: Action gracefully skips if download fails (non-blocking to workflow)
- **Whitespace Handling**: Environment variable setting handles edge cases automatically

### Memory Safety & Zeroization

The crypto module implements memory-safe key handling:

- **Intermediate Buffers**: All decoded/processed key material is zeroed after use via `zeroize` crate
- **Seed Array Zeroization**: `seed: [u8; SEED_LENGTH]` zeroed immediately after creating `SigningKey`
- **Error Path Safety**: Decoded buffers zeroed even on error paths (e.g., invalid length)
- **Key Derivation**: Signing key created from seed, then seed immediately zeroed
- **Comment Tags**: All zeroization points marked with FR-020 (security feature reference)

### Key Exposure Logging

The Monitor logs key origin at startup to help users verify key loading:

```rust
// When loaded from environment variable
info!(
    source = "environment",
    fingerprint = %crypto.public_key_fingerprint(),
    "Cryptographic key loaded"
);

// When loaded from file
info!(
    source = "file",
    path = %path.display(),
    fingerprint = %crypto.public_key_fingerprint(),
    "Cryptographic key loaded"
);

// Warning if both sources exist
info!(
    ignored_path = %config.key_path.display(),
    "File key exists but VIBETEA_PRIVATE_KEY takes precedence"
);
```

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Logging** | Structured JSON (tracing crate) | Server: `main.rs`, Monitor: `main.rs`, Client: error boundaries in components |
| **Error Handling** | Custom error enums + thiserror | `server/src/error.rs`, `monitor/src/error.rs` |
| **Rate Limiting** | Per-source counter with TTL | `server/src/rate_limit.rs` |
| **Privacy** | Event payload sanitization + tracker opt-out | `monitor/src/privacy.rs` (removes sensitive fields), tracker modules follow privacy-first design |
| **Graceful Shutdown** | Signal handlers + timeout | `server/src/main.rs`, `monitor/src/main.rs` |
| **Retry Logic** | Exponential backoff with jitter | `monitor/src/sender.rs`, `client/src/hooks/useWebSocket.ts` |
| **Key Management** | Ed25519 key generation, storage, signing, export | `monitor/src/crypto.rs` (with KeySource tracking, memory zeroing, and export support) |
| **GitHub Actions Integration** | Binary download, env var config, background launch | `.github/actions/vibetea-monitor/action.yml` |

## Enhanced Tracking Subsystem (Phase 4-11)

New modular tracking architecture in `monitor/src/trackers/`:

### agent_tracker Module

- **Purpose**: Extract Task tool agent spawn events from JSONL
- **Location**: `monitor/src/trackers/agent_tracker.rs`
- **Key Functions**:
  - `parse_task_tool_use()`: Detects Task tool_use blocks and extracts metadata
  - `create_agent_spawn_event()`: Constructs `AgentSpawnEvent` from parsed Task input
  - `try_extract_agent_spawn()`: Convenience wrapper for integration in parser
- **Privacy**: Extracts only `subagent_type` and `description`; ignores `prompt` field
- **Integration**: Called from `SessionParser::parse_line()` when processing tool_use content blocks
- **Output**: `ParsedEventKind::AgentSpawned` variant → `EventPayload::AgentSpawn` → `EventType::AgentSpawn`

### skill_tracker Module (Phase 5)

- **Purpose**: Monitor `~/.claude/history.jsonl` for skill/slash command invocations
- **Location**: `monitor/src/trackers/skill_tracker.rs`
- **Key Components**:
  - `SkillTracker`: Main tracker instance managing file watch and byte offset
  - `parse_history_entry()`: Parses JSON entry from history.jsonl
  - `create_skill_invocation_event()`: Constructs `SkillInvocationEvent` from parsed entry
  - `extract_skill_name()`: Tokenizes display string to extract skill name from `/command`
- **Privacy**: Extracts only skill name (parsed from `display` field); ignores command arguments
- **Integration**: Spawned as independent async task in `main.rs`; results queued via `sender.queue()`
- **Output**: `SkillInvocationEvent` → queued by main event loop
- **Architecture**: Uses `notify` crate for file changes; maintains byte offset for efficient tailing; processes immediately without debounce
- **Data Source**: `~/.claude/history.jsonl` (append-only format with JSON entries)

### stats_tracker Module (Phase 8-10)

- **Purpose**: Monitor token usage, session metrics, activity patterns, and model distribution from stats-cache.json
- **Location**: `monitor/src/trackers/stats_tracker.rs`
- **Key Components**:
  - `StatsTracker`: Main tracker instance managing file watch and debouncing
  - `StatsCache`: Parsed contents of stats-cache.json with aggregated metrics
  - `ModelTokens`: Per-model token usage (input, output, cache_read, cache_creation)
  - `StatsEvent`: Enum with four variants (TokenUsage, SessionMetrics, ActivityPattern, ModelDistribution)
  - `emit_stats_events()`: Reads file and emits all four event types in sequence
- **Data Source**: `~/.claude/stats-cache.json` with structure: totalSessions, totalMessages, totalToolUsage, longestSession, hourCounts, modelUsage
- **Event Emission** (Phase 8-10):
  - Emits one `SessionMetricsEvent` per file read
  - Emits one `ActivityPatternEvent` per file read (if hourCounts non-empty) - Phase 9
  - Emits one `ModelDistributionEvent` per file read (if modelUsage non-empty) - Phase 10
  - Emits one `TokenUsageEvent` per model
- **Integration**: Spawned as independent async task in `main.rs`; results queued via `sender.queue()`
- **Architecture**:
  - Uses `notify` crate for file changes (non-recursive directory watch)
  - Debounces file changes (200ms) to coalesce rapid updates
  - Includes retry logic for parse failures (up to 3 retries with 100ms delays)
  - Performs initial read on startup if file exists
  - Provides manual `refresh()` method for forcing a read
- **Privacy**: Extracts only aggregated metrics; never transmits code or personal data
- **Output**:
  - `StatsEvent::SessionMetrics` → converted to Event with `EventType::SessionMetrics` and `EventPayload::SessionMetrics`
  - `StatsEvent::ActivityPattern` → converted to Event with `EventType::ActivityPattern` and `EventPayload::ActivityPattern` (Phase 9)
  - `StatsEvent::ModelDistribution` → converted to Event with `EventType::ModelDistribution` and `EventPayload::ModelDistribution` (Phase 10)
  - `StatsEvent::TokenUsage` → converted to Event with `EventType::TokenUsage` and `EventPayload::TokenUsage`
- **Testing**: Comprehensive test suite covering JSON parsing, event emission, debouncing, retries, and tracker lifecycle

### todo_tracker Module (Phase 6)

- **Purpose**: Monitor `~/.claude/todos/` directory for todo list progress and abandonment detection
- **Location**: `monitor/src/trackers/todo_tracker.rs`
- **Data Source**: `~/.claude/todos/<session-uuid>-agent-<session-uuid>.json` files (JSON arrays of todo entries)
- **Key Components**:
  - `TodoTracker`: Main tracker instance managing directory watch and session state
  - `TodoEntry`: Individual todo item with `content`, `status`, and optional `activeForm`
  - `TodoStatus`: Enum with three states (Completed, InProgress, Pending)
  - `parse_todo_file()`: Strict JSON array parsing with validation
  - `parse_todo_file_lenient()`: Lenient parsing for partially-written files
  - `count_todo_statuses()`: Aggregate task counts by status
  - `is_abandoned()`: Detect incomplete tasks when session ends
  - `create_todo_progress_event()`: Construct `TodoProgressEvent`
- **Privacy**: Extracts only status counts and metadata; never transmits task content
- **Integration**: Spawned as independent async task in `main.rs`; receives `mark_session_ended()` calls for abandonment detection
- **Output**: `TodoProgressEvent` → queued by main event loop
- **Architecture**:
  - Uses `notify` crate for directory monitoring (non-recursive)
  - Debounces file changes (default 100ms) to coalesce rapid status updates
  - Maintains `RwLock<HashSet<String>>` of ended sessions for abandonment detection
  - Async task processes debounced changes and emits events
- **Debouncing**: Per research.md, 100ms is optimal to capture multiple status updates as single event
- **Session Lifecycle**:
  1. Session starts: TodoTracker begins watching
  2. Todo file created/modified: Events emitted with `abandoned=false`
  3. Session ends (summary event received): `mark_session_ended()` called
  4. Subsequent todo changes: Events emitted with `abandoned=true` if incomplete tasks remain

### file_history_tracker Module (Phase 8+)

- **Purpose**: Monitor file edit history and track line changes
- **Location**: `monitor/src/trackers/file_history_tracker.rs`
- **Data Source**: Project files watched during active Claude Code sessions
- **Integration**: Spawned as independent async task in `main.rs`
- **Output**: `FileChangeEvent` → queued by main event loop
- **Architecture**: Computes line deltas without exposing code content

### project_tracker Module (Phase 11)

- **Purpose**: Monitor `~/.claude/projects/` to identify active and completed sessions per project
- **Location**: `monitor/src/trackers/project_tracker.rs`
- **Directory Structure**: Watches `~/.claude/projects/<project-slug>/<session-uuid>.jsonl` files
- **Project Slug Format**: Directory names use path-to-slug conversion (forward slashes → dashes)
- **Key Components**:
  - `ProjectTracker`: Main tracker instance managing directory watch and initial scan
  - `ProjectTrackerConfig`: Configuration with `scan_on_init` flag
  - `parse_project_slug()`: Converts slug format back to original absolute path
  - `has_summary_event()`: Detects if session is completed (JSONL contains summary event)
  - `extract_session_id()`: Validates UUID format in filename
  - `create_project_activity_event()`: Constructs `ProjectActivityEvent`
- **Session Activity Detection**:
  - **Active**: Session JSONL does NOT contain a summary event (session ongoing)
  - **Completed**: Session JSONL contains a summary event (session finished)
- **Privacy**: Extracts only project paths and session IDs; never transmits code or file contents
- **Integration**: Spawned as independent async task in `main.rs`; results queued via channel to `sender.queue()`
- **Output**: `ProjectActivityEvent` → queued by main event loop
- **Architecture**:
  - Uses `notify` crate for recursive directory watching (handles project creation and session file changes)
  - No debouncing (per research.md: 0ms needed for project files)
  - Performs optional initial scan on startup (configurable via `ProjectTrackerConfig`)
  - Supports manual `scan_projects()` method for refreshing all project state
  - Channel-based communication (mpsc) for async event delivery
- **Initialization**:
  - Creates tracker with `ProjectTracker::new(tx)` - uses default config with scan_on_init=true
  - Or `ProjectTracker::with_config()` for custom configuration
  - Or `ProjectTracker::with_path()` to watch custom directory instead of ~/.claude/projects
- **File Event Processing**:
  - Only processes `.jsonl` files with valid UUID filenames
  - Extracts project slug from parent directory name
  - Verifies file is directly under project directory (not nested subdirs)
  - Reads file and checks for summary event to determine activity status
- **Testing**: Comprehensive unit + integration tests for path parsing, activity detection, file watching, and tracker lifecycle (39 test cases)

---

*This document describes HOW the system is organized. Consult STRUCTURE.md for WHERE code lives.*
