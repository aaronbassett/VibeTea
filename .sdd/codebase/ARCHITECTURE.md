# Architecture

> **Purpose**: Document system design, patterns, component relationships, and data flow.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

## Architecture Overview

VibeTea is a distributed event aggregation, broadcast, and persistence system for AI coding assistants with four integrated components:

- **Monitor** (Rust CLI) - Watches local Claude Code session files, forwards privacy-filtered events to the server, and optionally batches them to Supabase via edge functions
- **Server** (Rust HTTP API) - Central hub that receives events from monitors and broadcasts them to subscribers via WebSocket (in-memory, no persistence)
- **Client** (React SPA) - Real-time dashboard displaying aggregated event streams, session activity, and heatmaps (real-time + optional historic data from Supabase)
- **Supabase** (PostgreSQL + Edge Functions) - Optional persistence layer storing events and providing hourly aggregates for historic heatmap visualization

The system follows a **hub-and-spoke architecture** where the Server acts as a central event bus for real-time delivery, decoupling Monitor sources from Client consumers. Events flow unidirectionally: Monitors → Server → Clients (in-memory broadcast). Independently, if persistence is enabled, Monitors also batch events → Supabase edge functions → PostgreSQL database, and Clients query edge functions ← Supabase for historic data.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Central Server acts as real-time event aggregation point; Monitors feed events; Clients consume |
| **Producer-Consumer** | Monitors are event producers; Clients are event consumers; Server mediates asynchronous real-time delivery |
| **Optional Persistence** | Supabase layer (PostgreSQL + edge functions) provides optional durability for historic analysis without impacting real-time path |
| **Privacy-First** | Events contain only structural metadata (timestamps, tool names, file basenames), never code or sensitive content |
| **Real-time Streaming** | WebSocket-based live event delivery with no message persistence at Server (fire-and-forget) |
| **Serverless Processing** | Edge functions handle all Supabase database access via VibeTea authentication (not direct client access) |
| **Event-Driven** | Deno edge functions respond to HTTP requests for ingestion and querying; RPC functions handle database operations |
| **Async Channel-Based Batching** | **NEW Phase 4**: Persistence uses mpsc channels for decoupled event batching in background tasks |
| **Stale-While-Revalidate Caching** | **NEW Phase 5**: Client caches historic data with 5-minute staleness window, refetches in background |
| **SWR Hook Pattern** | **NEW Phase 5**: `useHistoricData` hook provides stale-while-revalidate data fetching with Bearer token auth |

## Core Components

### Monitor (Source)

- **Purpose**: Watches Claude Code session files in `~/.claude/projects/**/*.jsonl` and emits structured events; optionally batches events to Supabase
- **Location**: `monitor/src/`
- **Key Responsibilities**:
  - File watching via `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows)
  - Privacy-preserving JSONL parsing (extracts metadata only)
  - Cryptographic signing of events (Ed25519)
  - Event buffering and exponential backoff retry for real-time server
  - Optional batch buffering and submission to Supabase edge functions (if `VIBETEA_SUPABASE_URL` configured)
  - Batch retry logic with exponential backoff (1s → 2s → 4s, max 3 attempts, then drop and log)
  - **NEW Phase 4**: PersistenceManager runs as async background task, receives events via mpsc channel
  - **NEW Phase 4**: EventBatcher handles buffering, flushes on interval or capacity, manages retry state
  - Graceful shutdown with event flushing
- **Dependencies**:
  - Monitors depend on **Server** (via HTTP POST to `/events` for real-time)
  - Monitors optionally depend on **Supabase** (via HTTP POST to ingest edge function for persistence)
  - Monitors depend on local Claude Code installation (`~/.claude/`)
- **Dependents**: None (source component)

### Server (Hub)

- **Purpose**: Central event aggregation point that validates, authenticates, and broadcasts events to all subscribers in real-time
- **Location**: `server/src/`
- **Key Responsibilities**:
  - Receiving and authenticating events from Monitors (Ed25519 signature verification)
  - Rate limiting (per-source basis, configurable)
  - Broadcasting events to WebSocket subscribers (in-memory, no persistence)
  - Token-based authentication for WebSocket clients
  - Graceful shutdown with timeout for in-flight requests
- **Dependencies**:
  - Broadcasts to **Clients** (via WebSocket `/ws` endpoint)
  - Depends on Monitor-provided public keys for signature verification
- **Dependents**: Clients (consumers)
- **Note**: Server does NOT interact with Supabase; persistence is handled separately by Monitor → edge functions and Client ← edge functions

### Client (Consumer)

- **Purpose**: Real-time dashboard displaying aggregated event stream from the Server and optional historic data from Supabase
- **Location**: `client/src/`
- **Key Responsibilities**:
  - WebSocket connection management with exponential backoff reconnection
  - Event buffering (1000 events max) with FIFO eviction for real-time events
  - Session state management (Active, Inactive, Ended, Removed)
  - Event filtering (by session ID, time range)
  - Real-time visualization (event list, session overview)
  - Optional historic data fetching from Supabase query edge function (if `VITE_SUPABASE_URL` configured)
  - Heatmap visualization combining real-time event counts with historic hourly aggregates
  - Deduplication logic: real-time event counts for current hour override historic aggregates for that hour
  - **NEW Phase 5**: Stale-while-revalidate caching of historic data with 5-minute staleness window
  - **NEW Phase 5**: `useHistoricData` hook for reactive data fetching with Bearer token authentication
- **Dependencies**:
  - Depends on **Server** (via WebSocket connection to `/ws` for real-time)
  - Optionally depends on **Supabase** (via HTTP GET to query edge function for historic data)
  - No persistence layer (in-memory Zustand store for real-time events)
- **Dependents**: None (consumer component)

### Supabase (Persistence)

- **Purpose**: Optional persistence layer providing event durability and historic data for heatmap visualization
- **Location**: `supabase/` (migrations and edge functions)
- **Key Responsibilities**:
  - **Database**: PostgreSQL events table with RLS (Row Level Security) policies denying all direct access
  - **Migrations**: `supabase/migrations/20260203000000_create_events_table.sql` - Creates events table with indexes
  - **Migrations**: `supabase/migrations/20260203000001_create_functions.sql` - Creates bulk insert and aggregation functions
  - **Ingest Edge Function**: `supabase/functions/ingest/index.ts` - Validates Ed25519 signatures, inserts events via `bulk_insert_events()` function
  - **Query Edge Function**: `supabase/functions/query/index.ts` - Validates bearer tokens, returns hourly aggregates via `get_hourly_aggregates()` function
  - **Shared Auth**: `supabase/functions/_shared/auth.ts` - Ed25519 signature verification and bearer token validation
- **Dependencies**:
  - Receives events from **Monitor** (via HTTP POST to ingest edge function)
  - Receives queries from **Client** (via HTTP GET to query edge function)
- **Dependents**: Monitor (for batch submission), Client (for historic data queries)
- **Note**: Database uses service role only access; all client requests must go through authenticated edge functions

## Data Flow

### Primary Real-Time Monitor-to-Client Flow

Claude Code → Monitor → Server → Client (in-memory broadcast):
1. JSONL line written to `~/.claude/projects/<uuid>.jsonl`
2. File watcher detects change via inotify/FSEvents
3. Parser extracts event metadata (no code/prompts)
4. Privacy pipeline sanitizes payload
5. Sender signs with Ed25519, buffers, and retries on failure
6. POST /events sent to Server with X-Source-ID and X-Signature headers
7. Server verifies signature and rate limit
8. Broadcaster sends event to all WebSocket subscribers
9. Client receives via useWebSocket hook
10. Zustand store adds event (FIFO eviction at 1000 limit)
11. React renders updated event list, session overview, real-time heatmap

### Persistence Flow (Monitor to Supabase Edge Function)

Monitor → Supabase ingest edge function → PostgreSQL (batched, asynchronous to real-time):
1. **NEW Phase 4**: Monitor sends events to `PersistenceManager` via mpsc channel (non-blocking try_send)
2. **NEW Phase 4**: PersistenceManager runs in background task, receives events from channel
3. **NEW Phase 4**: EventBatcher accumulates events in buffer (max 1000)
4. **When batch interval elapses (default 60s) OR batch size reaches 1000, whichever first**:
   - EventBatcher serializes events to JSON array, signs with Ed25519
   - POST to `/functions/v1/ingest` with X-Source-ID and X-Signature headers
5. Edge function (`supabase/functions/ingest/index.ts`) validates signature (using shared auth utility)
6. Edge function validates event schema (id, source, timestamp, eventType, payload)
7. Edge function calls `bulk_insert_events(JSONB)` PL/pgSQL function with validated events
8. Function inserts events with `ON CONFLICT DO NOTHING` (idempotency)
9. Returns inserted count (separately tracks duplicates)
10. EventBatcher updates batch state and continues
11. On failure: exponential backoff (1s, 2s, 4s), max 3 retries, then drop and log
12. **NEW Phase 4**: On shutdown (channel closed), PersistenceManager flushes remaining events

### Historic Data Flow (Supabase Edge Function to Client)

Client → Supabase query edge function → PostgreSQL (periodic fetches with stale-while-revalidate caching):

**NEW Phase 5 - Stale-While-Revalidate Flow:**
1. Client calls `useHistoricData(days)` hook on component mount
2. Hook checks Zustand store for cached data and staleness timestamp
3. If cached data exists AND fresh (< 5 minutes old), immediately return cached data
4. If no cache OR data is stale (> 5 minutes old), trigger `fetchHistoricData(days)` action
5. While refetch is in-flight, return stale cached data for responsive UX (stale-while-revalidate)
6. Store updates `historicDataStatus` to 'loading' during fetch
7. GET `/functions/v1/query` with:
   - Authorization: Bearer token header
   - Query params: `days=7` or `days=30` (from hook parameter)
8. Edge function (`supabase/functions/query/index.ts`) validates bearer token (constant-time comparison)
9. Edge function parses and validates query parameters
10. Edge function calls `get_hourly_aggregates(days_back, source_filter)` RPC
11. Returns hourly event counts `{source, date, hour, event_count}` rows grouped by hour
12. Client receives response, updates `historicData` in Zustand, sets `historicDataFetchedAt` timestamp
13. Store updates `historicDataStatus` to 'success' or 'error'
14. `useHistoricData` hook returns updated result object with data, status, error, fetchedAt, refetch
15. Heatmap component re-renders with merged real-time + historic data
16. Manual refetch available via `refetch()` function (bypasses staleness check)

### Detailed Request/Response Cycle

#### 1. Event Creation (Monitor/Parser):
- JSONL line parsed from `~/.claude/projects/<uuid>.jsonl`
- `SessionParser` extracts timestamp, tool name, action
- `PrivacyPipeline` removes sensitive fields (code, prompts)
- `Event` struct created with unique ID (`evt_` prefix + 20-char suffix)

#### 2. Event Signing (Monitor/Sender):
- Event payload serialized to JSON
- Ed25519 signature computed over message body
- Event queued in local buffer (max 1000 for real-time, separate batch buffer for persistence)

#### 3. Event Transmission to Server (Monitor/Sender):
- `POST /events` with headers: `X-Source-ID`, `X-Signature`
- On 429 (rate limit): parse `Retry-After` header
- On network failure: exponential backoff (1s → 60s, ±25% jitter)
- On success: continue flushing buffered events

#### 4. Event Ingestion (Server):
- Extract `X-Source-ID` and `X-Signature` headers
- Load Monitor's public key from config (`VIBETEA_PUBLIC_KEYS`)
- Verify Ed25519 signature using `subtle::ConstantTimeEq` (timing-safe)
- Rate limit check per source_id
- Broadcast event to all WebSocket subscribers via `tokio::broadcast`

#### 5. Event Delivery (Server → Client):
- WebSocket subscriber receives `BroadcastEvent`
- Client's `useWebSocket` hook calls `addEvent()` action
- Zustand store updates event buffer (evicts oldest if > 1000)
- Session state updated (Active/Inactive/Ended/Removed)
- React re-renders only affected components (via Zustand selectors)

#### 6. Event Routing to Persistence (Monitor/Main Loop):
- **NEW Phase 4**: Event successfully sent to real-time server
- **NEW Phase 4**: Also queued to PersistenceManager via `persistence_tx.try_send(event)`
- **NEW Phase 4**: If channel full (backpressure), logs debug message and drops event
- **NEW Phase 4**: Real-time event delivery is unaffected by persistence channel state

#### 7. Async Persistence Batching (PersistenceManager):
- **NEW Phase 4**: PersistenceManager::run() continuously selects between:
  - Receiving events from mpsc channel
  - Periodic timer tick (configurable interval, default 60s)
- **NEW Phase 4**: When event received, queue in EventBatcher buffer
- **NEW Phase 4**: When buffer full (1000 events), trigger immediate flush
- **NEW Phase 4**: When timer ticks and buffer not empty, trigger periodic flush

#### 8. Event Batch Flush (EventBatcher):
- Serializes buffered events to JSON array
- Computes Ed25519 signature over JSON body
- POST to `/functions/v1/ingest` with X-Source-ID and X-Signature headers
- On success: clears buffer and resets retry counter
- On transient failure: exponential backoff retry (with configurable limit)
- On permanent failure (auth error): logs and skips retry
- On max retries exceeded: drops batch, resets retry counter, logs warning

#### 9. Event Persistence (Supabase Edge Function):
- Edge function (`supabase/functions/ingest/index.ts`) handles POST request
- Extract X-Source-ID and X-Signature from request headers
- Load public key for source from env (`VIBETEA_PUBLIC_KEYS`)
- Verify Ed25519 signature (using `verifyAsync()` from @noble/ed25519 in `_shared/auth.ts`)
- Parse and validate JSONB array from request body
- Validate each event: id pattern (evt_XXXXX), timestamp (RFC 3339), eventType (one of 6 types), source match
- Call `bulk_insert_events(JSONB)` with validated array
- Function inserts each event with ON CONFLICT DO NOTHING
- Return inserted_count separately from duplicates skipped
- Edge function returns success with counts or 400/401/422 error

#### 10. Historic Data Fetch (Client via Edge Function - Phase 5):
- Component mounts and calls `useHistoricData(7)` hook
- Hook checks Zustand for cached data and `historicDataFetchedAt` timestamp
- If fresh (< 5 min old), immediately return cached data without fetch
- If stale or missing, dispatch `fetchHistoricData(days)` action
- **Phase 5**: Set status to 'loading' and trigger background fetch
- GET `/functions/v1/query` with:
  - Authorization: Bearer token header (from localStorage `vibetea_token`)
  - Query params: `days=7` or `days=30`, optional `source=...`
- Edge function (`supabase/functions/query/index.ts`) validates bearer token
- Parses and validates query parameters
- Call `get_hourly_aggregates(days_back, source_filter)` RPC
- Returns hourly event counts `{source, date, hour, event_count}` with metadata (totalCount, daysRequested, fetchedAt)
- **Phase 5**: Client merges response with Zustand store, updates `historicData`, `historicDataFetchedAt`, status
- Heatmap re-renders with merged real-time + historic data

#### 11. Visualization (Client):
- `EventStream` component renders with virtualized scrolling (real-time events)
- `SessionOverview` shows active sessions with metadata
- `Heatmap` displays activity over time bins (real-time + historic aggregates, with deduplication for current hour)
- `ConnectionStatus` shows server connectivity
- **NEW Phase 5**: Historic data loading state shown with 'loading' or error messages

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|------------|---------------|
| **Monitor** | Observe local activity, preserve privacy, authenticate, batch for persistence | FileSystem, HTTP client, Crypto | Server internals, other Monitors, Supabase direct (only via edge functions) |
| **Server** | Route, authenticate, broadcast, rate limit (real-time only) | All Monitors' public keys, event config, rate limiter state, broadcast channel | File system, Supabase database, Client implementation, persistence state |
| **Client** | Display, interact, filter, manage WebSocket, fetch historic data with Bearer auth | Server WebSocket, Supabase edge functions, localStorage (token), Zustand state, React hooks | Server internals, other Clients' state, file system, Supabase database direct |
| **Supabase Edge Functions** | Validate auth, transform request, call database RPCs | Environment secrets (VIBETEA_PUBLIC_KEYS, VIBETEA_SUBSCRIBER_TOKEN), Supabase client | Monitor/Client implementation, direct database access (via RPC only) |
| **PostgreSQL** | Persist events, aggregate historic data, enforce RLS | RPC function calls from edge functions | Direct queries (RLS denies all) |

## Dependency Rules

- **Monitor → Server**: Continuous via HTTP POST (real-time path)
- **Monitor → PersistenceManager**: Via mpsc channel (async, non-blocking)
- **Monitor → Supabase**: Periodic via HTTP POST to edge functions (persistence path, async to real-time)
- **Server → Monitor**: None (server doesn't initiate contact)
- **Server → Supabase**: None (Server has no persistence concern)
- **Client → Server**: Initiated WebSocket, bidirectional (Client sends WebSocket close, Server sends events)
- **Client → Supabase**: Periodic HTTP GET to edge functions for historic data (independent of Server)
- **Supabase Edge Functions → Monitor/Client**: None (pull only via edge functions)
- **Supabase Edge Functions → Database**: Via RPC functions (SECURITY DEFINER for service role escalation)

## Key Interfaces & Contracts

| Interface | Purpose | Implementations |
|-----------|---------|-----------------|
| `Event` | Core event struct with type + payload | JSON serialization via `serde` (Rust), TypeScript interfaces (Client) |
| `EventPayload` | Tagged union of event variants | Session, Activity, Tool, Agent, Summary, Error |
| `EventType` | Enum discriminator | 6 variants (Session, Activity, Tool, Agent, Summary, Error) |
| `AuthError` | Auth failure codes | InvalidSignature, UnknownSource, InvalidToken |
| `RateLimitResult` | Rate limit outcome | Allowed, Blocked (with retry delay) |
| `SubscriberFilter` | Optional event filtering | by_event_type, by_project, by_source |
| `HourlyAggregate` | Historic data format | {source, date, hour, event_count} |
| `AuthResult` | Edge function auth result | {isValid, error?, sourceId?} |
| `IngestResponse` | Edge function ingest success | {inserted: number, message: string} |
| `QueryResponse` | Edge function query success | {aggregates: HourlyAggregate[], meta: QueryMeta} |
| **NEW Phase 4**: `PersistenceManager` | Async background task for event batching | Wraps `EventBatcher`, runs `run()` async function |
| **NEW Phase 4**: `EventBatcher` | Buffer and flush events with retry logic | `queue()`, `flush()`, `needs_flush()`, `is_empty()` |
| **NEW Phase 4**: `PersistenceError` | Persistence-specific errors | Http, Serialization, AuthFailed, ServerError, MaxRetriesExceeded |
| **NEW Phase 5**: `UseHistoricDataResult` | Hook result for historic data | {data, status, error, fetchedAt, refetch} |
| **NEW Phase 5**: `HistoricDataStatus` | Status of historic data fetch | 'idle' \| 'loading' \| 'error' \| 'success' |
| **NEW Phase 5**: `HistoricDataSnapshot` | Store state subset for historic data | {data, status, fetchedAt, error} |

## Authentication & Authorization

### Monitor Authentication (Source) - Real-Time Path

- **Mechanism**: Ed25519 signature verification
- **Flow**:
  1. Monitor generates keypair: `vibetea-monitor init`
  2. Public key registered with Server: `VIBETEA_PUBLIC_KEYS=monitor1:base64pubkey`
  3. On `POST /events`, Monitor signs message body with private key
  4. Server verifies signature against pre-registered public key
  5. Invalid signatures rejected with 401 Unauthorized
- **Security**: Uses `ed25519_dalek::VerifyingKey::verify_strict()` (RFC 8032 compliant)
- **Timing Attack Prevention**: `subtle::ConstantTimeEq` for signature comparison

### Monitor Authentication (Source) - Persistence Path

- **Mechanism**: Same Ed25519 signature verification used for real-time
- **Flow**:
  1. Monitor reads private key (initialized via `vibetea-monitor init`)
  2. On batch ready, signs batch JSON with private key
  3. POST to `/functions/v1/ingest` with X-Source-ID and X-Signature headers
  4. Edge function uses public key lookup and verification from `_shared/auth.ts`
- **Security**: Uses `@noble/ed25519` for RFC 8032 compliant verification in `verifyAsync()`
- **Timing Attack Prevention**: Constant-time comparison in edge function (uses `verifyAsync()` from noble library)

### Client Authentication (Consumer) - Real-Time Path

- **Mechanism**: Bearer token (HTTP header)
- **Flow**:
  1. Client obtains token (out-of-band, server-configured)
  2. Token sent in WebSocket upgrade request: `?token=secret`
  3. Server calls `validate_token()` to check token
  4. Invalid/missing tokens rejected with 401 Unauthorized
- **Storage**: Client stores token in localStorage under `vibetea_token` key

### Client Authentication (Consumer) - Persistence Path (NEW Phase 5)

- **Mechanism**: Same bearer token as real-time, via HTTP Authorization header
- **Flow**:
  1. Client reads token from localStorage (`vibetea_token`)
  2. GET `/functions/v1/query` with `Authorization: Bearer <token>` header
  3. Edge function validates token using `validateBearerToken()` from `_shared/auth.ts`
  4. Invalid/missing tokens rejected with 401 Unauthorized
- **Timing Attack Prevention**: Constant-time comparison in edge function using token equality check before slicing
- **Status Codes**: 401 for invalid/missing auth, 400 for invalid parameters, 200 with data on success

## State Management

### Server State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Configuration** | `Arc<Config>` in `AppState` | Immutable, shared across requests |
| **Broadcast Channel** | `EventBroadcaster` (tokio::broadcast) | Multi-producer, multi-consumer, lossy if slow subscribers |
| **Rate Limiter** | `RateLimiter` (Arc<Mutex<HashMap>>) | Per-source tracking with TTL-based cleanup |
| **Uptime** | `Instant` in `AppState` | Initialized at startup for health checks |

### Client State (NEW Phase 5 - Historic Data)
| State Type | Location | Pattern |
|------------|----------|---------|
| **Connection Status** | Zustand store | Driven by WebSocket events (connecting, connected, disconnected, reconnecting) |
| **Event Buffer** | Zustand store (Vec, max 1000) | FIFO eviction when full, newest first |
| **Sessions** | Zustand store (Map<sessionId, Session>) | Keyed by UUID, state machines (Active → Inactive → Ended → Removed) |
| **Filters** | Zustand store | Session ID filter, time range filter |
| **Authentication Token** | localStorage | Persisted across page reloads |
| **Historic Heatmap Data** | Zustand store (HourlyAggregate[]) | Fetched from Supabase edge function, merged with real-time counts |
| **Heatmap Loading State** | Zustand store | Loading, loaded, error (non-blocking) |
| **NEW Phase 5**: **Historic Data Cache** | Zustand store (readonly HourlyAggregate[]) | Cached with staleness timestamp, auto-refresh when > 5 min old |
| **NEW Phase 5**: **Historic Data Status** | Zustand store | 'idle' \| 'loading' \| 'success' \| 'error' (independent of real-time) |
| **NEW Phase 5**: **Historic Data Fetch Time** | Zustand store (timestamp) | Records when cache was last successfully fetched |
| **NEW Phase 5**: **Historic Data Error** | Zustand store (error message) | Captures fetch failures for UI feedback |

### Monitor State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Event Buffer** | `VecDeque<Event>` (max configurable) | FIFO eviction, flushed on graceful shutdown (real-time) |
| **Batch Buffer** | `Vec<Event>` in EventBatcher (max 1000) | Separate from real-time buffer, batched on interval or size limit |
| **Batch Timer** | `tokio::time::interval` in PersistenceManager | Periodic tick for batch flush trigger |
| **Batch Retry State** | `consecutive_failures` counter in EventBatcher | Tracks failures, resets on success, increments on transient errors |
| **Session Parsers** | `HashMap<PathBuf, SessionParser>` | Keyed by file path, created on first write, removed on file delete |
| **Retry State** | `Sender` internal | Tracks backoff attempt count per send operation (real-time) |
| **NEW Phase 4**: **Persistence Channel** | `mpsc::Sender<Event>` held in main loop | One per PersistenceManager instance, sent to background task |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Logging** | Structured JSON (tracing crate) | Server: `main.rs`, Monitor: `main.rs`, Client: error boundaries in components |
| **Error Handling** | Custom error enums + thiserror | `server/src/error.rs`, `monitor/src/error.rs`, `monitor/src/persistence.rs` |
| **Rate Limiting** | Per-source counter with TTL | `server/src/rate_limit.rs` |
| **Privacy** | Event payload sanitization | `monitor/src/privacy.rs` (removes sensitive fields) |
| **Graceful Shutdown** | Signal handlers + timeout | `server/src/main.rs`, `monitor/src/main.rs` |
| **Retry Logic** | Exponential backoff with jitter | `monitor/src/sender.rs` (real-time), `monitor/src/persistence.rs` (batch, Phase 4) |
| **Persistence** | Optional, async to real-time | Monitor: PersistenceManager + EventBatcher (Phase 4), Client: independent query fetches |
| **NEW Phase 4**: **Batch Retry** | Exponential backoff (1s, 2s, 4s), max configurable (1-10) | EventBatcher in `monitor/src/persistence.rs` |
| **NEW Phase 4**: **Channel Backpressure** | mpsc channel capacity (MAX_BATCH_SIZE * 2) | PersistenceManager receives from channel with non-blocking try_send in main loop |
| **NEW Phase 5**: **Historic Data Caching** | Stale-while-revalidate with 5-min threshold | `useHistoricData` hook and Zustand store |
| **NEW Phase 5**: **Bearer Token Auth** | Authorization header validation in edge functions | `supabase/functions/query/index.ts` and `_shared/auth.ts` |
| **Edge Function Auth** | Ed25519 + bearer tokens | `supabase/functions/_shared/auth.ts` |

## Design Decisions

### Why Hub-and-Spoke?

- **Decouples sources from sinks**: Multiple Monitor instances can run independently
- **Centralized authentication**: Server is the only point needing cryptographic keys
- **Easy horizontal scaling**: Monitors and Clients scale independently
- **No inter-Monitor coupling**: Monitors don't need to know about each other

### Why Optional Persistence via Supabase?

- **Decouples real-time from durability**: Persistence failures don't impact real-time broadcasts
- **Best-effort delivery**: System continues operating even if Supabase is temporarily unavailable
- **Async batching**: Monitor groups events to reduce request overhead
- **Privacy preserved**: Events already privacy-filtered by Monitor; database locked down with RLS

### Why Separate Batch Buffer?

- **Independent failure domains**: Persistence failures don't block real-time event delivery
- **Configurable batch interval**: Allows tuning latency vs. throughput (default 60s)
- **Idempotency**: Duplicate batch submissions handled via `ON CONFLICT DO NOTHING`

### Why PersistenceManager as Async Background Task (Phase 4)?

- **Non-blocking event routing**: Main event loop uses try_send, never blocks on persistence channel
- **Decoupled lifecycle**: Persistence manager runs independently, shutdown via channel close
- **Clear separation of concerns**: Batch management isolated in background task
- **Testable in isolation**: Can mock channel sender/receiver for unit tests
- **Future extensibility**: Easy to add multiple persistence targets (different channels)

### Why EventBatcher in Separate Module (Phase 4)?

- **Single Responsibility**: Handles only buffering, flushes, and retry logic
- **Reusable**: Can be used by multiple managers or in tests without tokio runtime
- **Clear interface**: Public methods for queue, flush, status checks
- **Flexible timing**: Timer and size-based triggers are independent

### Why Stale-While-Revalidate Caching (Phase 5)?

- **Responsive UX**: Immediate return of cached data without waiting for network
- **Background refresh**: Automatic staleness checks prevent stale data display
- **Graceful degradation**: If fetch fails, stale data still available
- **Network efficiency**: Reduces unnecessary requests for recently-fetched data
- **Standard pattern**: Well-understood caching strategy (used by HTTP and browsers)

### Why useHistoricData Hook (Phase 5)?

- **Composable**: Encapsulates fetching logic, state management, staleness checking
- **Declarative**: Component declares data needs, hook manages lifecycle
- **Reusable**: Multiple components can use same hook without duplicating logic
- **Testable**: Hook can be tested in isolation with mock Zustand state
- **Memoized**: Refetch function is memoized to prevent unnecessary re-renders

### Why Edge Functions for Database Access?

- **Centralized auth**: All database access enforces VibeTea authentication
- **RLS enforcement**: Database tables have implicit deny-all policies without edge function access
- **Reduces client library size**: Clients don't need Supabase SDK, just HTTP
- **Simplifies deployment**: Database credentials never exposed to Monitors or Clients
- **Deno runtime isolation**: Edge functions run in isolated Deno runtime, sandboxed from Rust components

### Why No Persistence in Server?

- **Simplifies deployment**: No database migration needed for Server
- **Supports distributed Servers**: Multiple Server instances don't need shared state
- **Real-time optimization**: No write latency to secondary storage
- **Privacy-first**: Events never written to disk except via optional Supabase layer

### Why Ed25519 for Both Paths?

- **Consistency**: Same key pair used for real-time Server auth and persistence edge function auth
- **Widely supported**: NIST-standardized modern elliptic curve
- **Signature verification only**: Public key crypto prevents Monitors impersonating each other
- **Timing-safe implementation**: `subtle::ConstantTimeEq` prevents timing attacks (Server), @noble/ed25519 for edge functions (RFC 8032 compliant)

### Why WebSocket for Real-Time?

- **Bi-directional low-latency**: Better than HTTP polling for real-time updates
- **Connection persistence**: Single connection replaces request/response overhead
- **Native browser support**: No additional libraries needed for basic connectivity
- **Standard protocol**: Works with existing proxies and load balancers

### Why Separate Ingest and Query Edge Functions?

- **Single Responsibility**: Ingest handles writes, Query handles reads
- **Independent scaling**: Can scale edge functions based on workload characteristics
- **Clear contracts**: Each function has focused input/output validation
- **Easier testing**: Smaller, more testable functions
- **Different auth mechanisms**: Ingest uses Ed25519 (like Monitor), Query uses bearer tokens (like Client)

---

*This document describes HOW the system is organized. Consult STRUCTURE.md for WHERE code lives.*
