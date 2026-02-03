# Architecture

> **Purpose**: Document system design, patterns, component relationships, and data flow.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

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

## Core Components

### Monitor (Source)

- **Purpose**: Watches Claude Code session files in `~/.claude/projects/**/*.jsonl` and emits structured events; optionally batches events to Supabase
- **Location**: `monitor/src/`
- **Key Responsibilities**:
  - File watching via `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows)
  - Privacy-preserving JSONL parsing (extracts metadata only)
  - Cryptographic signing of events (Ed25519)
  - Event buffering and exponential backoff retry for real-time server
  - **NEW Phase 3**: Optional batch buffering and submission to Supabase edge functions (if `VIBETEA_SUPABASE_URL` configured)
  - **NEW Phase 3**: Batch retry logic with exponential backoff (1s → 2s → 4s, max 3 attempts, then drop and log)
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
  - **NEW Phase 3**: Optional historic data fetching from Supabase query edge function (if `VITE_SUPABASE_URL` configured)
  - **NEW Phase 3**: Heatmap visualization combining real-time event counts with historic hourly aggregates
  - **NEW Phase 3**: Deduplication logic: real-time event counts for current hour override historic aggregates for that hour
- **Dependencies**:
  - Depends on **Server** (via WebSocket connection to `/ws` for real-time)
  - Optionally depends on **Supabase** (via HTTP GET to query edge function for historic data)
  - No persistence layer (in-memory Zustand store for real-time events)
- **Dependents**: None (consumer component)

### Supabase (Persistence) - Phase 3 Implementation

- **Purpose**: Optional persistence layer providing event durability and historic data for heatmap visualization
- **Location**: `supabase/` (migrations and edge functions)
- **Key Responsibilities**:
  - **Database**: PostgreSQL events table with RLS (Row Level Security) policies denying all direct access
  - **Migrations**: `supabase/migrations/20260203000000_create_events_table.sql` - Creates events table with indexes
  - **Migrations**: `supabase/migrations/20260203000001_create_functions.sql` - Creates bulk insert and aggregation functions
  - **Ingest Edge Function** (Phase 3 - IMPLEMENTED): `supabase/functions/ingest/index.ts` - Validates Ed25519 signatures, inserts events via `bulk_insert_events()` function
  - **Query Edge Function** (Phase 3 - IMPLEMENTED): `supabase/functions/query/index.ts` - Validates bearer tokens, returns hourly aggregates via `get_hourly_aggregates()` function
  - **Shared Auth** (Phase 3 - IMPLEMENTED): `supabase/functions/_shared/auth.ts` - Ed25519 signature verification and bearer token validation
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

### Phase 3: Persistence Flow (Monitor to Supabase Edge Function)

Monitor → Supabase ingest edge function → PostgreSQL (batched, asynchronous to real-time):
1. Monitor accumulates events in batch buffer
2. **When batch interval elapses (default 60s) OR batch size reaches 1000, whichever first**:
   - Sender signs batch with Ed25519, creates JSON array
   - POST to `/functions/v1/ingest` with X-Source-ID and X-Signature headers
3. Edge function (`supabase/functions/ingest/index.ts`) validates signature (using shared auth utility)
4. Edge function validates event schema (id, source, timestamp, eventType, payload)
5. Edge function calls `bulk_insert_events(JSONB)` PL/pgSQL function with validated events
6. Function inserts events with `ON CONFLICT DO NOTHING` (idempotency)
7. Returns inserted count (separately tracks duplicates)
8. Monitor updates batch state and continues
9. On failure: exponential backoff (1s, 2s, 4s), max 3 retries, then drop and log

### Phase 3: Historic Data Flow (Supabase Edge Function to Client)

Client → Supabase query edge function → PostgreSQL (periodic fetches):
1. Client initializes with `VITE_SUPABASE_URL` configured
2. On component mount or "refresh" click, fetches historic heatmap data
3. GET `/functions/v1/query` with Authorization: Bearer token header
4. Edge function (`supabase/functions/query/index.ts`) validates bearer token (constant-time comparison)
5. Edge function parses query parameters: `days` (7 or 30, default 7), optional `source` filter
6. Edge function calls `get_hourly_aggregates(days_back, source_filter)` RPC
7. Function returns `{source, date, hour, event_count}` rows grouped by hour, ordered by date DESC, hour DESC
8. Client receives hourly aggregates and stores in Zustand
9. Heatmap renders: real-time counts for current hour override historic counts for that hour
10. On fetch failure/timeout (5 seconds): shows "Unable to load historic data" with retry button

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

#### 6. Event Batching for Persistence (Monitor/Sender):
- Event added to batch buffer (separate from real-time buffer)
- Timestamp recorded for batch interval tracking
- **When 60 seconds elapses OR 1000 events accumulated**:
  - Batch serialized as JSONB array
  - Ed25519 signature computed over JSON batch
  - POST to `/functions/v1/ingest` with X-Source-ID and X-Signature

#### 7. Event Persistence (Phase 3 - Supabase Edge Function):
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

#### 8. Historic Data Fetch (Phase 3 - Client via Edge Function):
- On component mount (Heatmap) or manual refresh
- GET `/functions/v1/query` with:
  - Authorization: Bearer token header
  - Query params: `days=7` or `days=30` (optional), `source=...` (optional)
- Edge function (`supabase/functions/query/index.ts`) validates bearer token
- Parses and validates query parameters
- Call `get_hourly_aggregates(days_back, source_filter)` RPC
- Returns hourly event counts `{source, date, hour, event_count}` with metadata (totalCount, daysRequested, fetchedAt)
- Client merges with real-time data and renders heatmap

#### 9. Visualization (Client):
- `EventStream` component renders with virtualized scrolling (real-time events)
- `SessionOverview` shows active sessions with metadata
- `Heatmap` displays activity over time bins (real-time + historic aggregates)
- `ConnectionStatus` shows server connectivity

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|------------|---------------|
| **Monitor** | Observe local activity, preserve privacy, authenticate, batch for persistence | FileSystem, HTTP client, Crypto | Server internals, other Monitors, Supabase direct (only via edge functions) |
| **Server** | Route, authenticate, broadcast, rate limit (real-time only) | All Monitors' public keys, event config, rate limiter state, broadcast channel | File system, Supabase database, Client implementation, persistence state |
| **Client** | Display, interact, filter, manage WebSocket, fetch historic data | Server WebSocket, Supabase edge functions, localStorage (token), Zustand state | Server internals, other Clients' state, file system, Supabase database direct |
| **Supabase Edge Functions** | Validate auth, transform request, call database RPCs | Environment secrets (VIBETEA_PUBLIC_KEYS, VIBETEA_SUBSCRIBER_TOKEN), Supabase client | Monitor/Client implementation, direct database access (via RPC only) |
| **PostgreSQL** | Persist events, aggregate historic data, enforce RLS | RPC function calls from edge functions | Direct queries (RLS denies all) |

## Dependency Rules

- **Monitor → Server**: Continuous via HTTP POST (real-time path)
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
| **Phase 3**: `HourlyAggregate` | Historic data format | {source, date, hour, event_count} |
| **Phase 3**: `AuthResult` | Edge function auth result | {isValid, error?, sourceId?} |
| **Phase 3**: `IngestResponse` | Edge function ingest success | {inserted: number, message: string} |
| **Phase 3**: `QueryResponse` | Edge function query success | {aggregates: HourlyAggregate[], meta: QueryMeta} |

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

### Monitor Authentication (Source) - Persistence Path (Phase 3)

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

### Client Authentication (Consumer) - Persistence Path (Phase 3)

- **Mechanism**: Same bearer token as real-time, via HTTP Authorization header
- **Flow**:
  1. Client reads token from localStorage (`vibetea_token`)
  2. GET `/functions/v1/query` with `Authorization: Bearer <token>` header
  3. Edge function validates token using `validateBearerToken()` from `_shared/auth.ts`
  4. Invalid/missing tokens rejected with 401 Unauthorized
- **Timing Attack Prevention**: Constant-time comparison in edge function using check for token equality before slicing

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
| **Phase 3**: **Historic Heatmap Data** | Zustand store (HourlyAggregate[]) | Fetched from Supabase edge function, merged with real-time counts |
| **Phase 3**: **Heatmap Loading State** | Zustand store | Loading, loaded, error (non-blocking) |

### Monitor State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Event Buffer** | `VecDeque<Event>` (max configurable) | FIFO eviction, flushed on graceful shutdown (real-time) |
| **Phase 3**: **Batch Buffer** | `VecDeque<Event>` (max 1000) | Separate from real-time buffer, batched on interval or size limit |
| **Phase 3**: **Batch State** | Sender internal | Tracks last batch send time, retry count per batch |
| **Session Parsers** | `HashMap<PathBuf, SessionParser>` | Keyed by file path, created on first write, removed on file delete |
| **Retry State** | `Sender` internal | Tracks backoff attempt count per send operation (real-time) |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Logging** | Structured JSON (tracing crate) | Server: `main.rs`, Monitor: `main.rs`, Client: error boundaries in components |
| **Error Handling** | Custom error enums + thiserror | `server/src/error.rs`, `monitor/src/error.rs` |
| **Rate Limiting** | Per-source counter with TTL | `server/src/rate_limit.rs` |
| **Privacy** | Event payload sanitization | `monitor/src/privacy.rs` (removes sensitive fields) |
| **Graceful Shutdown** | Signal handlers + timeout | `server/src/main.rs`, `monitor/src/main.rs` |
| **Retry Logic** | Exponential backoff with jitter | `monitor/src/sender.rs` (real-time), batch retry (Phase 3) |
| **Phase 3**: **Persistence** | Optional, async to real-time | Monitor: batch buffering and submission, Client: independent query fetches |
| **Phase 3**: **Batch Retry** | Exponential backoff (1s, 2s, 4s), max 3 attempts | Monitor sender module (persistence path) |
| **Phase 3**: **Edge Function Auth** | Ed25519 + bearer tokens | `supabase/functions/_shared/auth.ts` |

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
