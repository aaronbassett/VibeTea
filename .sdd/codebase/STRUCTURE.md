# Project Structure

> **Purpose**: Document directory layout, module boundaries, and where to add new code.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

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
│   │   │   └── useSupabaseHistory.ts # Historic data fetching from Supabase edge function
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
| `hooks/useEventStore.ts` | Zustand store, event buffer, session state, filters | `useEventStore()` hook |
| `hooks/useSessionTimeouts.ts` | Session state machine (Active → Inactive → Ended) | `useSessionTimeouts()` hook |
| `hooks/useSupabaseHistory.ts` | Fetch historic data from edge function | `useSupabaseHistory()` hook |
| `types/events.ts` | TypeScript interfaces (VibeteaEvent, Session, etc.) | `VibeteaEvent`, `Session`, `HourlyAggregate` |
| `utils/formatting.ts` | Date/time/event type formatting | `formatTimestamp()`, `formatEventType()` |
| `__tests__/` | Vitest unit + integration tests | — |

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

No back-end dependencies (except server WebSocket and optional Supabase).

```
client/src/App.tsx (root)
├── hooks/
│   ├── useWebSocket.ts (WebSocket, reconnect)
│   ├── useEventStore.ts (Zustand state)
│   ├── useSessionTimeouts.ts (session state machine)
│   └── useSupabaseHistory.ts (historic data fetching)
├── components/
│   ├── TokenForm.tsx (auth)
│   ├── ConnectionStatus.tsx (status badge)
│   ├── EventStream.tsx (virtualized list)
│   ├── SessionOverview.tsx (table)
│   └── Heatmap.tsx (visualization with historic data)
└── types/events.ts (TypeScript interfaces)
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
import { useSupabaseHistory } from './hooks/useSupabaseHistory';
import type { VibeteaEvent, HourlyAggregate } from './types/events';

// In client/src/components/Heatmap.tsx
import { useEventStore } from '../hooks/useEventStore';
import { useSupabaseHistory } from '../hooks/useSupabaseHistory';
import type { Session, HourlyAggregate } from '../types/events';
```

**Conventions**:
- Components: PascalCase (e.g., `EventStream.tsx`)
- Hooks: camelCase starting with `use` (e.g., `useWebSocket.ts`)
- Utils: camelCase (e.g., `formatting.ts`)
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
| Hook files | `camelCase.ts` | `useWebSocket.ts`, `useEventStore.ts` |
| Utility files | `camelCase.ts` | `formatting.ts` |
| Type files | `camelCase.ts` | `events.ts` |
| Constants | `UPPER_SNAKE_CASE` | `TOKEN_STORAGE_KEY`, `MAX_BACKOFF_MS` |
| Test files | `__tests__/{name}.test.ts` | `__tests__/formatting.test.ts` |

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
✓ CAN import:     components, hooks, types, utils, React, Zustand, third-party UI libs
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

---

*This document shows WHERE code lives. Consult ARCHITECTURE.md for HOW the system is organized.*
