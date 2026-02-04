# Architecture

> **Purpose**: Document system design, patterns, component relationships, and data flow.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

## Architecture Overview

VibeTea is a distributed event aggregation and broadcast system for AI coding assistants with three independent components:

- **Monitor** (Rust CLI) - Watches local Claude Code session files and forwards privacy-filtered events to the server, with enhanced tracking modules for agent spawns, skill invocations, token usage, and other metrics
- **Server** (Rust HTTP API) - Central hub that receives events from monitors and broadcasts them to subscribers via WebSocket
- **Client** (React SPA) - Real-time dashboard displaying aggregated event streams, session activity, and usage heatmaps

The system follows a **hub-and-spoke architecture** where the Server acts as a central event bus, decoupling multiple Monitor sources from multiple Client consumers. Events flow unidirectionally: Monitors → Server → Clients, with no persistent storage.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Central Server acts as event aggregation point, Monitors feed events, Clients consume |
| **Producer-Consumer** | Monitors are event producers, Clients are event consumers, Server mediates asynchronous delivery |
| **Privacy-First** | Events contain only structural metadata (timestamps, tool names, skill names, file basenames), never code or sensitive content |
| **Real-time Streaming** | WebSocket-based live event delivery with no message persistence (fire-and-forget) |
| **Modular Tracking** | Enhanced tracking subsystem with pluggable tracker modules for different data sources |

## Core Components

### Monitor (Source)

- **Purpose**: Watches Claude Code session files in `~/.claude/projects/**/*.jsonl` and `~/.claude/history.jsonl`, emitting structured events
- **Location**: `monitor/src/`
- **Key Responsibilities**:
  - File watching via `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows)
  - Privacy-preserving JSONL parsing (extracts metadata only)
  - Cryptographic signing of events (Ed25519)
  - Event buffering and exponential backoff retry
  - Graceful shutdown with event flushing
  - Enhanced tracking of specialized events (agent spawns, skill invocations, token usage, stats)
- **Dependencies**:
  - Monitors depend on **Server** (via HTTP POST to `/events`)
  - Monitors depend on local Claude Code installation (`~/.claude/`)
- **Dependents**: None (source component)

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

- **Purpose**: Real-time dashboard displaying aggregated event stream from the Server
- **Location**: `client/src/`
- **Key Responsibilities**:
  - WebSocket connection management with exponential backoff reconnection
  - Event buffering (1000 events max) with FIFO eviction
  - Session state management (Active, Inactive, Ended, Removed)
  - Event filtering (by session ID, time range)
  - Real-time visualization (event list, session overview, heatmap)
- **Dependencies**:
  - Depends on **Server** (via WebSocket connection to `/ws`)
  - No persistence layer (in-memory Zustand store)
- **Dependents**: None (consumer component)

## Data Flow

### Primary Monitor-to-Client Flow

Claude Code → Monitor → Server → Client:
1. JSONL line written to `~/.claude/projects/<uuid>.jsonl` or skill invocation to `~/.claude/history.jsonl`
2. File watcher detects change via inotify/FSEvents
3. Parser extracts event metadata (no code/prompts)
4. Enhanced trackers extract specialized event types (agent spawns, skill invocations, token usage)
5. Privacy pipeline sanitizes payload
6. Sender signs with Ed25519, buffers, and retries on failure
7. POST /events sent to Server with X-Source-ID and X-Signature headers
8. Server verifies signature and rate limit
9. Broadcaster sends event to all WebSocket subscribers
10. Client receives via useWebSocket hook
11. Zustand store adds event (FIFO eviction at 1000 limit)
12. React renders updated event list, session overview, heatmap

### Enhanced Tracking Flow (Phase 4-5)

Three parallel tracking pipelines within Monitor:

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
history.jsonl → FileWatcher
  ↓
WatchEvent (LinesAdded)
  ↓
SkillTracker.process_line()
  ├→ parse_history_entry()
  ├→ extract_skill_name() (tokenization)
  └→ SkillInvocationEvent
```

**Pipeline 3: Stats Tracker (stats.jsonl → TokenUsageEvent)**
```
stats.jsonl → StatsTracker
  ↓
TokenUsageEvent (token counts by model)
  ↓
Sender queue
```

All pipelines feed into the same `sender.queue()` for batching and transmission.

### Detailed Request/Response Cycle

1. **Event Creation** (Monitor/Parser):
   - JSONL line parsed from `~/.claude/projects/<uuid>.jsonl`
   - `SessionParser` extracts timestamp, tool name, action
   - `agent_tracker` detects Task tool_use and extracts agent metadata
   - `SkillTracker` detects history.jsonl changes and parses skill invocations
   - `PrivacyPipeline` removes sensitive fields (code, prompts)
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
| **Monitor** | Observe local activity, preserve privacy, authenticate, track metrics | FileSystem, HTTP client, Crypto, Stats files, history.jsonl | Server internals, other Monitors, network stack details |
| **Server** | Route, authenticate, broadcast, rate limit | All Monitors' public keys, event config, rate limiter state | File system, external APIs, Client implementation details |
| **Client** | Display, interact, filter, manage WebSocket | Server WebSocket, local storage (token), state store | Server internals, other Clients' state, file system |

## Dependency Rules

- **Monitor → Server**: Only on startup (load server URL from config), then periodically via HTTP POST
- **Server → Monitor**: None (server doesn't initiate contact)
- **Server ↔ Client**: Bidirectional (Client initiates WebSocket, Server sends events, Client sends nothing back)
- **Cross-Monitor**: No inter-Monitor communication (all go through Server)
- **Persistence**: None at any layer (no database, no cache persistence)

## Key Interfaces & Contracts

| Interface | Purpose | Implementations |
|-----------|---------|-----------------|
| `Event` | Core event struct with type + payload | JSON serialization via `serde` |
| `EventPayload` | Tagged union of event variants | Session, Activity, Tool, Agent, Summary, Error, AgentSpawn, SkillInvocation, TokenUsage, SessionMetrics, etc. |
| `EventType` | Enum discriminator | 10+ variants (Session, Activity, Tool, Agent, Summary, Error, AgentSpawn, SkillInvocation, TokenUsage, etc.) |
| `ParsedEventKind` | Parser output enum | SessionStarted, Activity, ToolStarted, ToolCompleted, Summary, AgentSpawned |
| `AgentSpawnEvent` | Task tool agent spawn metadata | session_id, agent_type, description, timestamp |
| `SkillInvocationEvent` | Skill/slash command invocation metadata | session_id, skill_name, project, timestamp |
| `TaskToolInput` | Parsed Task tool input | subagent_type, description (prompt excluded for privacy) |
| `AuthError` | Auth failure codes | InvalidSignature, UnknownSource, InvalidToken |
| `RateLimitResult` | Rate limit outcome | Allowed, Blocked (with retry delay) |
| `SubscriberFilter` | Optional event filtering | by_event_type, by_project, by_source |

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
| **Retry State** | `Sender` internal | Tracks backoff attempt count per send operation |
| **Stats Tracker** | `StatsTracker` instance | Watches stats.jsonl for token usage metrics |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Logging** | Structured JSON (tracing crate) | Server: `main.rs`, Monitor: `main.rs`, Client: error boundaries in components |
| **Error Handling** | Custom error enums + thiserror | `server/src/error.rs`, `monitor/src/error.rs` |
| **Rate Limiting** | Per-source counter with TTL | `server/src/rate_limit.rs` |
| **Privacy** | Event payload sanitization + tracker opt-out | `monitor/src/privacy.rs` (removes sensitive fields), tracker modules follow privacy-first design |
| **Graceful Shutdown** | Signal handlers + timeout | `server/src/main.rs`, `monitor/src/main.rs` |
| **Retry Logic** | Exponential backoff with jitter | `monitor/src/sender.rs`, `client/src/hooks/useWebSocket.ts` |

## Enhanced Tracking Subsystem (Phase 4-5)

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

### stats_tracker Module

- **Purpose**: Monitor token usage and session metrics from stats.jsonl
- **Location**: `monitor/src/trackers/stats_tracker.rs`
- **Integration**: Spawns async task to watch `~/.claude/stats.jsonl`
- **Output**: `TokenUsageEvent` → queued by main event loop

### Future Tracker Modules (Planned)

- `todo_tracker`: Todo list progress and abandonment detection
- `file_history_tracker`: File edit line change tracking
- `project_tracker`: Active project session tracking

## Design Decisions

### Why Hub-and-Spoke?

- **Decouples sources from sinks**: Multiple Monitor instances can run independently
- **Centralized authentication**: Server is the only point needing cryptographic keys
- **Easy horizontal scaling**: Monitors and Clients scale independently
- **No inter-Monitor coupling**: Monitors don't need to know about each other

### Why No Persistence?

- **Simplifies deployment**: No database to manage
- **Supports distributed monitoring**: Each Client sees latest events only
- **Privacy-first**: Events never written to disk (except logs)
- **Real-time focus**: System optimized for live streams, not historical analysis

### Why Ed25519?

- **Widely supported**: NIST-standardized modern elliptic curve
- **Signature verification only**: Public key crypto prevents Monitors impersonating each other
- **Timing-safe implementation**: `subtle::ConstantTimeEq` prevents timing attacks
- **Small key size**: ~32 bytes per key, easy to share via env vars

### Why WebSocket?

- **Bi-directional low-latency**: Better than HTTP polling for real-time updates
- **Connection persistence**: Single connection replaces request/response overhead
- **Native browser support**: No additional libraries needed for basic connectivity
- **Standard protocol**: Works with existing proxies and load balancers

### Why Modular Trackers?

- **Extensibility**: New event types can be added without touching parser core
- **Separation of concerns**: Each tracker focuses on one data source
- **Privacy control**: Trackers can be disabled/enabled per configuration
- **Parallel extraction**: Multiple trackers can process different JSONL files independently
- **Independent lifecycle**: Trackers can have different update frequencies and processing patterns

### Why Separate Skill Tracker?

- **Different data source**: history.jsonl is separate from session JSONL files
- **Append-only pattern**: Allows efficient tailing with byte offsets
- **Immediate processing**: No debounce needed (user actions are already serialized)
- **Independent lifecycle**: Can be enabled/disabled without affecting session tracking

---

*This document describes HOW the system is organized. Consult STRUCTURE.md for WHERE code lives.*
