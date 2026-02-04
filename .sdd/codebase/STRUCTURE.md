# Project Structure

> **Purpose**: Document directory layout, module boundaries, and where to add new code.
> **Generated**: 2026-02-03
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
│   │   ├── sender.rs          # HTTP client with retry and buffering (real-time + persistence)
│   │   ├── persistence.rs     # NEW Phase 4: Event batching and Supabase persistence (PersistenceManager + EventBatcher)
│   │   ├── types.rs           # Event type definitions
│   │   └── error.rs           # Error types
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
│   │   │   └── Heatmap.tsx           # Activity over time visualization (real-time + historic)
│   │   ├── hooks/
│   │   │   ├── useWebSocket.ts       # WebSocket connection management
│   │   │   ├── useEventStore.ts      # Zustand store (state + selectors)
│   │   │   ├── useSessionTimeouts.ts # Session state machine (Active → Inactive → Ended)
│   │   │   ├── useSupabaseHistory.ts # Historic data fetching from Supabase edge function (Phase 4)
│   │   │   └── useHistoricData.ts    # NEW Phase 5: SWR hook for historic data fetching with staleness checks
│   │   ├── mocks/             # NEW Phase 5: MSW handlers for testing
│   │   │   ├── index.ts       # Mock exports
│   │   │   ├── server.ts      # MSW server setup for Vitest
│   │   │   ├── handlers.ts    # Query endpoint handler (validates token, returns aggregates)
│   │   │   └── data.ts        # Mock data factories (createHourlyAggregate, generateMockAggregates)
│   │   ├── types/
│   │   │   └── events.ts             # TypeScript event interfaces
│   │   ├── utils/
│   │   │   ├── formatting.ts         # Timestamp, event type formatting
│   │   │   └── persistence.ts        # NEW Phase 6: Feature detection utilities (isPersistenceEnabled, getSupabaseUrl, etc.)
│   │   ├── __tests__/
│   │   │   ├── App.test.tsx          # Integration tests
│   │   │   ├── events.test.ts        # Event parsing/filtering tests
│   │   │   ├── formatting.test.ts    # Formatting utility tests
│   │   │   ├── components/           # NEW Phase 6: Component-specific tests
│   │   │   │   └── Heatmap.test.tsx  # Heatmap component tests (persistence detection, data merging, loading states)
│   │   │   └── hooks/                # NEW Phase 5: Hook-specific tests
│   │   │       └── useHistoricData.test.tsx  # useHistoricData SWR behavior tests
│   │   └── index.css
│   ├── public/
│   ├── vite.config.ts
│   ├── package.json
│   ├── tsconfig.json
│   └── vitest.config.ts       # NEW Phase 5: Vitest configuration with MSW setup
│
├── supabase/                   # Supabase configuration and migrations
│   ├── migrations/             # Database migration scripts
│   │   ├── 20260203000000_create_events_table.sql    # Events table + RLS + indexes
│   │   └── 20260203000001_create_functions.sql       # bulk_insert_events + get_hourly_aggregates
│   ├── functions/              # Edge functions
│   │   ├── _shared/
│   │   │   └── auth.ts         # Shared auth utilities (Ed25519 verification, token validation)
│   │   ├── ingest/             # Batch event ingest
│   │   │   ├── index.ts        # Receives batched events from Monitor, validates, inserts
│   │   │   └── index.test.ts   # Tests for ingest edge function
│   │   └── query/              # Historic data query
│   │       ├── index.ts        # Returns hourly aggregates to Client
│   │       └── index.test.ts   # Tests for query edge function
│   ├── .env.local.example      # Supabase environment template
│   ├── config.toml             # Supabase local development config
│   └── .gitignore
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
| `main.rs` | CLI entry (init/run commands), signal handling | `Cli`, `Command` |
| `config.rs` | Load from env vars: `VIBETEA_*` | `Config`, `PersistenceConfig` |
| `watcher.rs` | inotify/FSEvents for `~/.claude/projects/**/*.jsonl` | `FileWatcher`, `WatchEvent` |
| `parser.rs` | Parse JSONL, extract Session/Activity/Tool events | `SessionParser`, `ParsedEvent`, `ParsedEventKind` |
| `privacy.rs` | Remove code, prompts, sensitive data | `PrivacyPipeline`, `PrivacyConfig` |
| `crypto.rs` | Ed25519 keypair (generate, load, save) | `Crypto` |
| `sender.rs` | HTTP POST to server with retry/buffering (real-time only) | `Sender`, `SenderConfig`, `RetryPolicy` |
| `persistence.rs` | **NEW Phase 4**: Event batching and Supabase persistence | `PersistenceManager`, `EventBatcher`, `PersistenceError` |
| `types.rs` | Event schema (shared with server) | `Event`, `EventPayload`, `EventType` |
| `error.rs` | Error types | `MonitorError`, custom errors |

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
| `components/Heatmap.tsx` | Activity heatmap binned by time (real-time + historic) | `Heatmap` component |
| `hooks/useWebSocket.ts` | WebSocket lifecycle, reconnection with backoff | `useWebSocket()` hook |
| `hooks/useEventStore.ts` | Zustand store, event buffer, session state, filters, historic data | `useEventStore()` hook |
| `hooks/useSessionTimeouts.ts` | Session state machine (Active → Inactive → Ended) | `useSessionTimeouts()` hook |
| `hooks/useSupabaseHistory.ts` | Fetch historic data from edge function (Phase 4) | `useSupabaseHistory()` hook |
| `hooks/useHistoricData.ts` | **NEW Phase 5**: SWR hook for historic data with stale checks | `useHistoricData()` hook |
| `mocks/server.ts` | **NEW Phase 5**: MSW server instance for tests | `server` export |
| `mocks/handlers.ts` | **NEW Phase 5**: MSW handlers for `/functions/v1/query` | `queryHandlers` array |
| `mocks/data.ts` | **NEW Phase 5**: Mock data factories | `createHourlyAggregate()`, `generateMockAggregates()` |
| `mocks/index.ts` | **NEW Phase 5**: Re-exports for easy importing | All mocks |
| `types/events.ts` | TypeScript interfaces (VibeteaEvent, Session, etc.) | `VibeteaEvent`, `Session`, `HourlyAggregate` |
| `utils/formatting.ts` | Date/time/event type formatting | `formatTimestamp()`, `formatEventType()` |
| `utils/persistence.ts` | **NEW Phase 6**: Feature detection utilities | `isPersistenceEnabled()`, `getSupabaseUrl()`, `isAuthTokenConfigured()`, `getPersistenceStatus()` |
| `__tests__/` | Vitest unit + integration tests | — |
| `__tests__/components/Heatmap.test.tsx` | **NEW Phase 6**: Heatmap component tests | Test suites for persistence detection, data merging, loading states |
| `__tests__/hooks/useHistoricData.test.tsx` | **NEW Phase 5**: Tests for stale-while-revalidate behavior | Test suites for staleness, caching, refetch |

### `supabase/migrations/` - Database Schema

| File | Purpose | Responsibilities |
|------|---------|------------------|
| `20260203000000_create_events_table.sql` | Main events table | Create `public.events` table with id, source, timestamp, event_type, payload, created_at columns; create indexes on timestamp, source, (source + timestamp); enable RLS with implicit deny-all |
| `20260203000001_create_functions.sql` | Database functions | Create `bulk_insert_events(JSONB)` function for batch insertion with ON CONFLICT DO NOTHING; create `get_hourly_aggregates(days_back, source_filter)` for hourly aggregates; grant EXECUTE to service_role only |

### `supabase/functions/_shared/` - Edge Function Utilities

| File | Purpose | Exports |
|------|---------|---------|
| `auth.ts` | Shared authentication for all edge functions | `verifySignature()`, `getPublicKeyForSource()`, `validateBearerToken()`, `verifyIngestAuth()`, `verifyQueryAuth()`, `AuthResult` interface |

### `supabase/functions/ingest/` - Ingest Edge Function

| File | Purpose | Contract |
|------|---------|----------|
| `index.ts` | Receive batched events from Monitor | **Request**: POST with `X-Source-ID`, `X-Signature` headers, JSON array body; **Response**: `{inserted: number, message: string}` or error response |
| `index.test.ts` | Test ingest edge function | Tests signature verification, event validation, schema enforcement |

### `supabase/functions/query/` - Query Edge Function

| File | Purpose | Contract |
|------|---------|----------|
| `index.ts` | Return hourly aggregates to Client | **Request**: GET with `Authorization: Bearer token` header, optional query params `days` (7\|30) and `source`; **Response**: `{aggregates: HourlyAggregate[], meta: QueryMeta}` or error response |
| `index.test.ts` | Test query edge function | Tests bearer token validation, parameter parsing, RPC calls |

### `client/src/mocks/` - MSW Testing Setup (NEW Phase 5)

| File | Purpose | Responsibility |
|------|---------|-----------------|
| `index.ts` | Main export file | Re-exports `server`, `queryHandlers`, data factories for easy test imports |
| `server.ts` | MSW server instance | Pre-configured `setupServer()` with all VibeTea handlers; used in Vitest setup |
| `handlers.ts` | MSW request handlers | `queryHandler` for GET `/functions/v1/query`; validates Authorization header and days parameter |
| `data.ts` | Test data generators | `createHourlyAggregate()` for single aggregates; `generateMockAggregates(days)` for realistic mock data with work-hour variance |

**Usage in tests:**
```typescript
import { server } from '../mocks/server';
import { createHourlyAggregate } from '../mocks/data';

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### `client/src/__tests__/components/` - Component Testing (NEW Phase 6)

| File | Purpose | Test Coverage |
|------|---------|----------------|
| `Heatmap.test.tsx` | Tests for `Heatmap` component | Persistence detection, data merging, loading states, error handling, view toggles, cell interactions, empty states |

**Test categories:**
- **Persistence Detection**: Component returns null when disabled, renders when enabled
- **Data Merging**: Real-time counts override historic aggregates for current hour
- **Loading States**: Loading indicator during fetch, error messages with retry
- **View Toggles**: Switch between 7-day and 30-day views
- **Cell Interactions**: Hover tooltips, click filtering, keyboard navigation
- **Empty States**: Show when no events available

### `client/src/__tests__/hooks/` - Hook Testing (NEW Phase 5)

| File | Purpose | Test Coverage |
|------|---------|----------------|
| `useHistoricData.test.tsx` | Tests for `useHistoricData` hook | Initial fetch behavior, stale-while-revalidate caching, error handling, manual refetch, days parameter validation, edge cases |

**Test categories:**
- **Initial Fetch**: Auto-fetch on mount when no cache
- **Stale-While-Revalidate**: Return cached data immediately, refetch in background when stale (>5 min)
- **Error Handling**: Preserve data on error, set error messages
- **Manual Refetch**: `refetch()` bypasses staleness checks
- **Days Parameter**: Correct 7/30 day requests
- **Edge Cases**: Empty responses, invalid tokens, readonly arrays

## Module Boundaries

### Monitor Module

Self-contained CLI with these responsibilities:
1. **Watch** files via `FileWatcher`
2. **Parse** JSONL via `SessionParser`
3. **Filter** events via `PrivacyPipeline`
4. **Sign** events via `Crypto`
5. **Send** to server via `Sender` (real-time path)
6. **NEW Phase 4**: **Batch and send** to Supabase via `PersistenceManager` + `EventBatcher` (persistence path, if enabled)

No cross-dependencies with Server or Client.

```
monitor/src/main.rs
├── config.rs (load env)
├── watcher.rs → sender.rs
│   ↓
├── parser.rs → privacy.rs
│   ↓
├── sender.rs (HTTP, retry, buffering, real-time only)
│   ├── crypto.rs (sign events)
│   └── types.rs (Event schema)
│
└── NEW Phase 4: persistence.rs
    ├── PersistenceManager (background async task)
    │   ├── mpsc channel receiver
    │   ├── tokio timer
    │   └── spawned as background task
    │
    └── EventBatcher (buffer, flush, retry)
        ├── buffer: Vec<Event> (max 1000)
        ├── queue(event) → add to buffer
        ├── flush() → HTTP POST to ingest edge function
        ├── retry with exponential backoff (1s, 2s, 4s)
        └── crypto.rs (sign batch)
```

### Server Module

Central hub with these responsibilities:
1. **Route** HTTP requests to handlers
2. **Authenticate** monitors (verify signatures)
3. **Validate** tokens for WebSocket clients
4. **Broadcast** events to subscribers
5. **Rate limit** per-source

No direct dependencies on Monitor or Client implementation. No persistence concerns.

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
1. **Connect** to server via WebSocket (real-time)
2. **Fetch** historic data from Supabase edge function (if enabled)
3. **Manage** application state (Zustand)
4. **Display** events, sessions, heatmap (merged real-time + historic)
5. **Filter** by session/time range
6. **Persist** authentication token
7. **NEW Phase 5**: **Cache** historic data with stale-while-revalidate pattern
8. **NEW Phase 6**: **Detect** persistence capability and conditionally render components

No back-end dependencies (except server WebSocket and optional Supabase).

```
client/src/App.tsx (root)
├── hooks/
│   ├── useWebSocket.ts (WebSocket, reconnect)
│   ├── useEventStore.ts (Zustand state + historic data)
│   ├── useSessionTimeouts.ts (session state machine)
│   ├── useSupabaseHistory.ts (historic data fetching, Phase 4)
│   └── useHistoricData.ts (SWR caching, staleness checks, Phase 5)
├── components/
│   ├── TokenForm.tsx (auth)
│   ├── ConnectionStatus.tsx (status badge)
│   ├── EventStream.tsx (virtualized list)
│   ├── SessionOverview.tsx (table)
│   └── Heatmap.tsx (visualization with historic data, Phase 6 feature detection)
├── mocks/                         # NEW Phase 5
│   ├── index.ts
│   ├── server.ts
│   ├── handlers.ts
│   └── data.ts
├── __tests__/
│   ├── App.test.tsx
│   ├── events.test.ts
│   ├── formatting.test.ts
│   ├── components/                # NEW Phase 6
│   │   └── Heatmap.test.tsx
│   └── hooks/                     # NEW Phase 5
│       └── useHistoricData.test.tsx
├── types/events.ts (TypeScript interfaces)
└── utils/
    ├── formatting.ts
    └── persistence.ts (NEW Phase 6)
```

### Supabase Module

Database and edge functions with these responsibilities:
1. **Store** events persistently in PostgreSQL
2. **Lock down** database with RLS (service_role only access)
3. **Validate** requests from Monitor (Ed25519 signature)
4. **Validate** requests from Client (bearer token)
5. **Aggregate** event counts by hour for heatmap
6. **Ensure** idempotency (ON CONFLICT DO NOTHING)

No back-end dependencies (pull only via edge functions).

```
supabase/
├── migrations/
│   ├── 20260203000000_create_events_table.sql
│   └── 20260203000001_create_functions.sql
└── functions/
    ├── _shared/auth.ts (shared utilities)
    ├── ingest/
    │   ├── index.ts (validate Monitor auth, bulk insert)
    │   └── index.test.ts
    └── query/
        ├── index.ts (validate Client auth, return aggregates)
        └── index.test.ts
```

## Where to Add New Code

| If you're adding... | Put it in... | Example |
|---------------------|--------------|---------|
| **New Monitor command** | `monitor/src/main.rs` (add to `Command` enum) | `Command::Status` |
| **New Monitor feature** | `monitor/src/<feature>.rs` (new module) | `monitor/src/compression.rs` |
| **NEW Phase 4**: **Persistence test** | `monitor/src/persistence.rs` (in #[cfg(test)] module) | Tests for EventBatcher, PersistenceManager |
| **New Server endpoint** | `server/src/routes.rs` (add route handler) | `POST /events/:id/ack` |
| **New Server middleware** | `server/src/routes.rs` or `server/src/` (new module) | `server/src/middleware.rs` |
| **New event type** | `server/src/types.rs` + `monitor/src/types.rs` (sync both) | New `EventPayload` variant |
| **New DB table** | `supabase/migrations/TIMESTAMP_description.sql` | `supabase/migrations/20260210000000_create_sessions.sql` |
| **New edge function** | `supabase/functions/{name}/index.ts` (+ shared auth import) | `supabase/functions/export/index.ts` |
| **New database function** | `supabase/migrations/` (SQL function in new migration) | `get_event_details()` |
| **New edge function test** | `supabase/functions/{name}/index.test.ts` | `supabase/functions/export/index.test.ts` |
| **New Client component** | `client/src/components/` | `client/src/components/EventDetail.tsx` |
| **New Client hook** | `client/src/hooks/` | `client/src/hooks/useFilters.ts` |
| **NEW Phase 5**: **New hook test** | `client/src/__tests__/hooks/{hookName}.test.tsx` | `client/src/__tests__/hooks/useFilters.test.tsx` |
| **NEW Phase 6**: **New component test** | `client/src/__tests__/components/{ComponentName}.test.tsx` | `client/src/__tests__/components/EventDetail.test.tsx` |
| **NEW Phase 5**: **New MSW handler** | `client/src/mocks/handlers.ts` (add to queryHandlers array) | Handler for new edge function endpoint |
| **NEW Phase 5**: **New mock data factory** | `client/src/mocks/data.ts` (new export function) | `createSessionData()`, `generateEventHistory()` |
| **NEW Phase 6**: **New persistence utility** | `client/src/utils/persistence.ts` (add function) | `hasSupabaseToken()` |
| **New Client page** | `client/src/pages/` (if routing added) | `client/src/pages/Analytics.tsx` |
| **Shared utilities** | Monitor: `monitor/src/utils/` (if created), Server: `server/src/utils/`, Client: `client/src/utils/` | `format_`, `validate_` |
| **Tests** | Colocate with source: `file.rs` → `file_test.rs` (Rust), `file.ts` → `__tests__/file.test.ts` (TS) | — |

## Import Paths & Module Organization

### Monitor/Server (Rust)

**Convention**: Use fully qualified names from crate root via `use` statements.

```rust
// In monitor/src/main.rs
use vibetea_monitor::config::Config;
use vibetea_monitor::watcher::FileWatcher;
use vibetea_monitor::sender::Sender;
use vibetea_monitor::persistence::PersistenceManager;
use vibetea_monitor::types::Event;

// In server/src/routes.rs
use vibetea_server::auth::verify_signature;
use vibetea_server::broadcast::EventBroadcaster;
use vibetea_server::config::Config;
use vibetea_server::types::Event;
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
import { useHistoricData } from './hooks/useHistoricData';
import { isPersistenceEnabled } from './utils/persistence';
import type { VibeteaEvent, HourlyAggregate } from './types/events';

// In client/src/components/Heatmap.tsx
import { useEventStore } from '../hooks/useEventStore';
import { useHistoricData } from '../hooks/useHistoricData';
import { isPersistenceEnabled } from '../utils/persistence';
import type { Session, HourlyAggregate } from '../types/events';

// NEW Phase 5: In tests
import { server } from '../mocks/server';
import { createHourlyAggregate } from '../mocks/data';

// NEW Phase 6: Persistence utilities
import { isPersistenceEnabled, getSupabaseUrl, getPersistenceStatus } from '../utils/persistence';
```

**Conventions**:
- Components: PascalCase (e.g., `EventStream.tsx`)
- Hooks: camelCase starting with `use` (e.g., `useWebSocket.ts`, `useHistoricData.ts`)
- Mocks: camelCase (e.g., `handlers.ts`, `data.ts`)
- Utils: camelCase (e.g., `formatting.ts`, `persistence.ts`)
- Types: camelCase (e.g., `events.ts`)

### Supabase Edge Functions (TypeScript)

**Convention**: Import from shared auth utilities and external ES modules via `esm.sh` or direct imports.

```typescript
// In supabase/functions/ingest/index.ts
import { verifyIngestAuth, type AuthResult } from "../_shared/auth.ts";
import { createClient, SupabaseClient } from "https://esm.sh/@supabase/supabase-js@2";

// In supabase/functions/query/index.ts
import { verifyQueryAuth, type AuthResult } from "../_shared/auth.ts";
import { createClient, SupabaseClient } from "https://esm.sh/@supabase/supabase-js@2";

// In supabase/functions/_shared/auth.ts
import * as ed from "https://esm.sh/@noble/ed25519@2.0.0";
```

**Conventions**:
- Function directories match Supabase naming (lowercase with underscores)
- Shared utilities in `_shared/` (Supabase convention)
- External imports via ES modules (Deno runtime)
- Shared auth module exports `verifyIngestAuth()`, `verifyQueryAuth()` for use in ingest/query functions

## Entry Points

| Component | File | Launch Command |
|-----------|------|-----------------|
| **Monitor** | `monitor/src/main.rs` | `cargo run -p vibetea-monitor -- run` |
| **Server** | `server/src/main.rs` | `cargo run -p vibetea-server` |
| **Client** | `client/src/main.tsx` | `npm run dev` (from `client/`) |
| **Supabase** | `supabase/config.toml` | `supabase start` |
| **Tests** | `client/vitest.config.ts` | `npm run test` (from `client/`) |

## Generated/Auto-Configured Files

Files that are auto-generated or should not be manually edited:

| Location | Generator | Regenerate Command |
|----------|-----------|-------------------|
| `Cargo.lock` | Cargo | `cargo lock` (auto-managed) |
| `target/` | Rust compiler | `cargo build` |
| `client/dist/` | Vite | `npm run build` |
| `client/node_modules/` | pnpm | `pnpm install` |
| `supabase/.temp/` | Supabase CLI | Auto-managed during `supabase start` |

## Naming Conventions

### Rust Modules and Types

| Category | Pattern | Example |
|----------|---------|---------|
| Module names | `snake_case` | `parser.rs`, `privacy.rs`, `persistence.rs` |
| Type names | `PascalCase` | `Event`, `ParsedEvent`, `EventPayload`, `PersistenceManager`, `EventBatcher` |
| Function names | `snake_case` | `verify_signature()`, `calculate_backoff()`, `queue()`, `flush()` |
| Constant names | `UPPER_SNAKE_CASE` | `MAX_BODY_SIZE`, `EVENT_ID_PREFIX`, `MAX_BATCH_SIZE`, `REQUEST_TIMEOUT_SECS` |
| Test functions | `#[test]` or `_test.rs` suffix | `privacy_test.rs`, `test_queue_adds_event()` |

### TypeScript Components and Functions

| Category | Pattern | Example |
|----------|---------|---------|
| Component files | `PascalCase.tsx` | `EventStream.tsx`, `TokenForm.tsx` |
| Hook files | `camelCase.ts` | `useWebSocket.ts`, `useEventStore.ts`, `useHistoricData.ts` |
| Mock files | `camelCase.ts` | `handlers.ts`, `data.ts`, `server.ts` |
| Utility files | `camelCase.ts` | `formatting.ts`, `persistence.ts` |
| Type files | `camelCase.ts` | `events.ts` |
| Constants | `UPPER_SNAKE_CASE` | `TOKEN_STORAGE_KEY`, `MAX_BACKOFF_MS`, `STALE_THRESHOLD_MS` |
| Test files | `__tests__/{name}.test.ts` or `__tests__/{type}/{name}.test.tsx` | `__tests__/formatting.test.ts`, `__tests__/components/Heatmap.test.tsx` |

### Supabase and Database

| Category | Pattern | Example |
|----------|---------|---------|
| Migration files | `YYYYMMDDhhmmss_description.sql` | `20260203000000_create_events_table.sql` |
| Table names | `snake_case`, lowercase | `events`, `user_sessions` |
| Column names | `snake_case`, lowercase | `event_type`, `created_at` |
| Index names | `idx_{table}_{columns}` | `idx_events_timestamp`, `idx_events_source` |
| Function names | `snake_case`, lowercase | `bulk_insert_events()`, `get_hourly_aggregates()` |
| Edge function directories | `snake_case`, lowercase | `ingest`, `query`, `_shared` |
| Edge function files | `index.ts` for function, `index.test.ts` for tests | `supabase/functions/ingest/index.ts` |

## Dependency Boundaries (Import Rules)

### Monitor

```
✓ CAN import:     types, config, crypto, watcher, parser, privacy, sender, persistence, error
✓ CAN import:     std, tokio, serde, ed25519-dalek, notify, reqwest, thiserror
✗ CANNOT import:  server modules, client code, supabase modules
```

### Server

```
✓ CAN import:     types, config, auth, broadcast, rate_limit, error, routes
✓ CAN import:     std, tokio, axum, serde, ed25519-dalek, subtle
✗ CANNOT import:  monitor modules, client code, supabase modules (no persistence concern)
```

### Client

```
✓ CAN import:     components, hooks, mocks, types, utils, React, Zustand, Vitest, RTL, MSW, third-party UI libs
✓ CAN import:     @testing-library/react, vitest, msw (for tests)
✗ CANNOT import:  monitor code, server code (except via HTTP/WebSocket), supabase SDK (only HTTP to edge functions)
```

### Supabase Edge Functions

```
✓ CAN import:     _shared/auth.ts, @noble/ed25519, @supabase/supabase-js, esm.sh modules, Deno stdlib
✓ CAN import:     POST/GET handlers, database query logic, @noble/ed25519 for signature verification
✗ CANNOT import:  monitor code, server code, client code, node modules (Deno runtime)
```

## Environment Variables

### Monitor (`monitor/src/config.rs`)

| Variable | Purpose | Example | Required |
|----------|---------|---------|----------|
| `VIBETEA_SERVER_URL` | Real-time server endpoint | `http://localhost:8080` | Yes |
| `VIBETEA_SOURCE_ID` | Monitor identifier for signatures | `monitor-1` | Yes |
| `VIBETEA_KEY_PATH` | Directory with private key (default: ~/.vibetea) | `/home/user/.vibetea` | No |
| `VIBETEA_CLAUDE_DIR` | Claude Code directory to watch (default: ~/.claude) | `/home/user/.claude` | No |
| `VIBETEA_BUFFER_SIZE` | Real-time event buffer capacity (default: 1000) | `1000` | No |
| `VIBETEA_BASENAME_ALLOWLIST` | Comma-separated file extensions to include | `.ts,.js,.py` | No |
| `VIBETEA_SUPABASE_URL` | Supabase edge function URL (enables persistence) | `https://xxxx.supabase.co/functions/v1` | No |
| `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS` | Batch submission interval (default: 60) | `60` | No |
| `VIBETEA_SUPABASE_RETRY_LIMIT` | Max retry attempts for batch (default: 3, range 1-10) | `3` | No |

### Server (`server/src/config.rs`)

| Variable | Purpose | Example | Required |
|----------|---------|---------|----------|
| `PORT` | HTTP server port (default: 8080) | `8080` | No |
| `VIBETEA_PUBLIC_KEYS` | Monitor public keys (format: id:key,id2:key2) | `monitor1:base64key1,monitor2:base64key2` | Yes* |
| `VIBETEA_SUBSCRIBER_TOKEN` | WebSocket auth token | `secret-token` | Yes* |
| `VIBETEA_UNSAFE_NO_AUTH` | Disable auth (dev only, set to 'true') | `true` | No |
| `RUST_LOG` | Log level (default: info) | `debug` | No |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Client (`.env` in `client/` directory)

| Variable | Purpose | Example | Required |
|----------|---------|---------|----------|
| `VITE_SERVER_URL` | Real-time server WebSocket endpoint | `ws://localhost:8080` | Yes |
| `VITE_SUPABASE_URL` | Supabase project URL (enables historic data) | `https://xxxx.supabase.co` | No |
| `VITE_SUPABASE_QUERY_FUNCTION_NAME` | Edge function name for historic data (default: `query`) | `query` | No |

### Supabase (`.env.local` in `supabase/` directory)

| Variable | Purpose | Example | Required |
|----------|---------|---------|----------|
| `VIBETEA_PUBLIC_KEYS` | Monitor public keys (same as Server) | `monitor1:base64key1` | Yes |
| `VIBETEA_SUBSCRIBER_TOKEN` | WebSocket auth token (same as Server, used for query endpoint) | `secret-token` | Yes |

## Testing

### Client Testing Strategy (NEW Phase 5+)

**Framework**: Vitest with `@testing-library/react` and `happy-dom` environment

**Layers**:
- **Unit tests**: Individual utilities, pure functions → `__tests__/{name}.test.ts`
- **Component tests**: React components with state → `__tests__/components/{name}.test.tsx`
- **Hook tests**: React hooks with state management → `__tests__/hooks/{name}.test.tsx`
- **Integration tests**: Component + hook interactions → `__tests__/{name}.test.tsx`

**Mock Service Worker (MSW) Setup**:
- Server instance in `mocks/server.ts` listens on all requests
- Handlers in `mocks/handlers.ts` intercept and respond to edge function calls
- Data factories in `mocks/data.ts` generate realistic test data
- Use `beforeAll`, `afterEach`, `afterAll` hooks to manage server lifecycle

**Heatmap Component Testing (NEW Phase 6)**:
- Test persistence detection with `isPersistenceEnabled()` utility
- Test component returns null when persistence disabled
- Test data merging with real-time and historic aggregates
- Test loading states and error handling
- Test view toggles and cell interactions
- Test timezone handling for UTC-to-local bucket key conversion

**useHistoricData Hook Testing (Phase 5)**:
- Test stale-while-revalidate caching (5-min staleness window)
- Test automatic refetch when stale
- Test manual refetch via returned function
- Test error handling and state preservation
- Test days parameter (7 vs 30) handling
- Test readonly array return type

### Rust Testing (Phase 4+)

**Monitor Tests**:
- `monitor/src/persistence.rs`: EventBatcher unit tests (queue, flush, retry)
- `monitor/tests/`: Integration tests for end-to-end flows
- Run with `--test-threads=1` for env var isolation (see CLAUDE.md)

**Server Tests**:
- `server/tests/unsafe_mode_test.rs`: Auth bypass mode tests
- Signature verification tests (timing-safe comparison)
- Rate limiting tests
- WebSocket broadcast tests

---

*This document shows WHERE code lives. Consult ARCHITECTURE.md for HOW the system is organized.*
