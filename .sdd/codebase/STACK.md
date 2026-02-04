# Technology Stack

**Status**: Phase 5 Implementation - Client Supabase Integration with MSW Testing
**Last Updated**: 2026-02-04

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, JSONL parsing, privacy filtering, event signing, HTTP transmission, event batching |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization with historic data queries |
| Supabase Functions | TypeScript | (Deno 2) | Edge Functions for event ingestion and querying |
| Supabase Database | PostgreSQL | 17 | Relational database for event persistence |

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
| serial_test        | 3.2     | Serial test execution for environment variable tests | Server, Monitor (test-only) |

### TypeScript/JavaScript (Client & Supabase Functions)

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
| @noble/ed25519             | ^2.0.0   | Ed25519 signature verification in Edge Functions (RFC 8032 compliant, Phase 3) |
| @supabase/supabase-js      | 2        | Supabase JavaScript client for Edge Functions (Phase 3) |

### Supabase & PostgreSQL (Phase 3+)

| Component | Version | Purpose |
|-----------|---------|---------|
| Supabase | (latest) | Backend-as-a-Service with PostgreSQL database and Edge Functions |
| PostgreSQL | 17 | Relational database for event persistence |
| Deno | 2 | JavaScript runtime for Supabase Edge Functions |
| PostgREST API | (included) | Auto-generated REST API from PostgreSQL schema |

## Build Tools & Package Managers

| Tool     | Version  | Purpose |
|----------|----------|---------|
| cargo    | -        | Rust package manager and build system |
| pnpm     | -        | Node.js package manager (client) |
| rustfmt  | -        | Rust code formatter |
| clippy   | -        | Rust linter |
| prettier | ^3.8.1   | Code formatter (TypeScript) |
| ESLint   | ^9.39.2  | JavaScript/TypeScript linter |
| supabase CLI | (latest) | Supabase local development and deployment |

## Development & Testing

### Rust Testing
| Package      | Version | Purpose |
|--------------|---------|---------|
| tokio-test   | 0.4     | Tokio testing utilities |
| tempfile     | 3.15    | Temporary file/directory management for tests |
| serial_test  | 3.2     | Serial test execution for environment variable tests |
| wiremock     | 0.6     | HTTP mocking for integration tests |

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit/component testing framework |
| @testing-library/react | ^16.3.2  | React testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM matchers for testing |
| jsdom                  | ^28.0.0  | DOM implementation for Node.js |
| msw                    | ^2.12.8  | **Phase 5**: Mock Service Worker for API mocking in tests |

### Deno Testing (Phase 3)
| Module | Version | Purpose |
|--------|---------|---------|
| deno.land/std | ^0.224.0 | Standard library with assert and BDD testing utilities |
| deno.land/std/assert | ^0.224.0 | Assertion functions for Deno tests |
| deno.land/std/testing/bdd | ^0.224.0 | BDD-style test framework (describe, it, beforeEach, afterEach) |
| deno.land/std/testing/asserts | ^0.224.0 | Additional assertion helpers |

## Configuration Files

| File | Framework | Purpose |
|------|-----------|---------|
| `client/vite.config.ts` | Vite | Build configuration, WebSocket proxy to server on port 8080 |
| `client/tsconfig.json` | TypeScript | Strict mode, ES2020 target |
| `client/eslint.config.js` | ESLint | Flat config format with TypeScript support |
| `client/package.json` | npm/pnpm | Client dependencies including MSW for test mocking |
| `Cargo.toml` (workspace) | Cargo | Rust workspace configuration and shared dependencies |
| `server/Cargo.toml` | Cargo | Server package configuration |
| `monitor/Cargo.toml` | Cargo | Monitor package configuration |
| `supabase/config.toml` | Supabase CLI | Local development environment configuration (PostgreSQL 17, Deno 2 runtime) |

## Runtime Environment

| Aspect | Details |
|--------|---------|
| Server Runtime | Rust binary (tokio async) |
| Client Runtime | Browser (ES2020+) |
| Monitor Runtime | Native binary (Linux/macOS/Windows) with CLI and async persistence manager |
| Supabase Functions | Deno 2 JavaScript runtime (Phase 3) |
| Database Runtime | PostgreSQL 17 with PostgREST API |
| Node.js | Required for development and client build only |
| Async Model | Tokio (Rust), Promises (TypeScript/Deno) |
| WebSocket Support | Native (server-side via axum, client-side via browser) |
| WebSocket Proxy | Vite dev server proxies /ws to localhost:8080 |
| File System Monitoring | Rust notify crate (inotify/FSEvents) for JSONL tracking |
| CLI Support | Manual command parsing in monitor main.rs (init, run, help, version) |
| Event Persistence | Async batching with timer-based and capacity-based flushing (Phase 4) |
| Local Supabase | Docker-based with PostgreSQL, PostgREST, Deno runtime, Auth (port 54321) |
| Client Testing | MSW intercepts fetch/HTTP requests for isolated unit tests (Phase 5) |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature with X-Signature header |
| Server → Client | WebSocket | JSON | Bearer token |
| Client → Server | WebSocket | JSON | Bearer token |
| Monitor → File System | Native | JSONL | N/A (local file access) |
| Client → Supabase Functions | HTTPS POST/GET | JSON | Bearer token (query endpoint, Phase 5) |
| Monitor → Supabase Functions | HTTPS POST | JSON | Ed25519 signature (ingest endpoint, Phase 3) |
| Monitor Persistence → Supabase | HTTPS POST | JSON | Ed25519 signature (Phase 4) |
| Supabase Functions → PostgreSQL | SQL | JSON | Service role key |

## Data Serialization

| Component | Serialization | Notes |
|-----------|---------------|-------|
| Server/Monitor | serde (Rust) | JSON with snake_case for env configs |
| Client | TypeScript/JSON | camelCase for API contracts |
| Events | serde_json | Standardized event schema across components |
| Claude Code Files | JSONL (JSON Lines) | Privacy-first parsing extracting only metadata |
| Cryptographic Keys | Base64 + Raw bytes | Public keys base64 encoded, private keys raw 32-byte seeds |
| Database Events | JSONB (PostgreSQL) | Full event payload stored as JSON in `events.payload` column |
| Edge Function Auth | Base64 + Base64 | Ed25519 signatures and public keys encoded base64 |
| Event Persistence | serde_json + Vec | Batched events serialized to JSON for Supabase ingest |
| Historic Data | JSON Array | HourlyAggregate objects returned by query endpoint (Phase 5) |

## Build Output

| Component | Output | Format | Deployment |
|-----------|--------|--------|-----------|
| Server | Binary | ELF (Linux) | Docker container on Fly.io |
| Monitor | Binary | ELF/Mach-O/PE | Standalone executable for users |
| Client | Static files | JS + CSS (Brotli compressed) | CDN (Netlify/Vercel/Cloudflare) |
| Supabase Functions | TypeScript | Deno-compatible JavaScript | Hosted on Supabase Edge Functions (Phase 3) |

## Module Organization

### Client (`client/src`)
- `components/` - React components
  - `ConnectionStatus.tsx` - **Phase 7**: Visual WebSocket connection status indicator
  - `TokenForm.tsx` - **Phase 7**: Token management and persistence UI
  - `EventStream.tsx` - **Phase 8**: Virtual scrolling event stream with 1000+ event support
  - `Heatmap.tsx` - **Phase 9**: Activity heatmap with CSS Grid, color scale, 7/30-day views, accessibility
  - `SessionOverview.tsx` - **Phase 10**: Session cards with activity indicators and status badges
- `hooks/useEventStore.ts` - Zustand store for WebSocket event state with session tracking, timeout management, and historic data caching
- `hooks/useWebSocket.ts` - **Phase 7**: WebSocket connection management with auto-reconnect
- `hooks/useSessionTimeouts.ts` - **Phase 10**: Session timeout checking (5min active→inactive, 30min removal)
- `hooks/useHistoricData.ts` - **Phase 5**: Stale-while-revalidate caching for historic event aggregates from Supabase
- `types/events.ts` - Event type definitions with discriminated union types, HourlyAggregate schema (Phase 5)
- `utils/` - Utility functions
  - `formatting.ts` - **Phase 8**: Timestamp and duration formatting utilities (5 functions, 331 lines)
- `mocks/` - **Phase 5**: MSW mock handlers for testing
  - `handlers.ts` - Query endpoint mock handler with auth and parameter validation (111 lines)
  - `server.ts` - MSW server setup for Vitest integration (26 lines)
  - `data.ts` - Mock data and response builders for query endpoint testing
  - `index.ts` - MSW exports barrel file
- `__tests__/` - Test files
  - `formatting.test.ts` - **Phase 8**: Comprehensive formatting utility tests (33 test cases)
  - `events.test.ts` - **Phase 5**: Event type and schema validation tests
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
- `config.rs` - Configuration from environment variables (server URL, source ID, key path, buffer size, persistence config)
- `error.rs` - Error types
- `types.rs` - Event types
- `parser.rs` - Claude Code JSONL parser (privacy-first metadata extraction)
- `watcher.rs` - File system watcher for `.claude/projects/**/*.jsonl` files with position tracking
- `privacy.rs` - **Phase 5**: Privacy pipeline for event sanitization before transmission
- `crypto.rs` - **Phase 6**: Ed25519 keypair generation, loading, saving, and event signing
- `sender.rs` - **Phase 6**: HTTP client with event buffering, exponential backoff retry, and rate limit handling
- `persistence.rs` - **Phase 4**: Event batching and async persistence manager for Supabase edge function
- `main.rs` - **Phase 6**: CLI entry point with init and run commands
- `lib.rs` - Public interface

### Supabase (`supabase/`, Phase 3)
- `config.toml` - Local development environment configuration (PostgreSQL 17, Deno 2, API on port 54321)
- `migrations/` - Database schema and function definitions
  - `20260203000000_create_events_table.sql` - Events table with JSONB payload, indexes, RLS
  - `20260203000001_create_functions.sql` - PostgreSQL functions: `bulk_insert_events()`, `get_hourly_aggregates()`
- `functions/` - Edge Functions (Deno TypeScript, Phase 3)
  - `_shared/auth.ts` - **Phase 3**: Ed25519 signature verification using @noble/ed25519, bearer token validation, source key lookup
  - `ingest/index.ts` - **Phase 3**: Event ingestion with request validation, batch size limits, event schema validation
  - `ingest/index.test.ts` - **Phase 3**: Auth tests for ingest endpoint using Deno test framework
  - `query/index.ts` - **Phase 3**: Query endpoint with parameter validation (days, source filtering)
  - `query/index.test.ts` - **Phase 3**: Auth tests for query endpoint with EnvGuard for environment isolation
  - `_tests/rls.test.ts` - **Phase 3**: Row-level security negative tests

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary, minimal base image |
| Client | CDN | Static files | Optimized builds with compression |
| Monitor | Local | Native binary | Users download and run locally |
| Supabase Functions | Supabase Hosted | Deno Container | Auto-deployed from `supabase/functions/` (Phase 3) |
| Database | Supabase Hosted | PostgreSQL Container | Managed PostgreSQL 17 instance |

## Phase 5 Additions (Client Supabase Integration with MSW Testing)

**Client Historic Data Fetching** (`client/src/hooks/useHistoricData.ts`):
- **useHistoricData Hook**: Implements stale-while-revalidate caching pattern (141 lines)
  - Fetches hourly event aggregates from Supabase query endpoint
  - Stale threshold: 5 minutes (STALE_THRESHOLD_MS)
  - Automatic background refetch when cached data is stale
  - Returns immediately with cached data while revalidating
  - Manual refetch capability for user-triggered refreshes
  - Proper loading and error state handling
  - Integrates with Zustand store state (historicData, historicDataStatus, historicDataFetchedAt, historicDataError)

**Zustand Store Extensions** (`client/src/hooks/useEventStore.ts`):
- New state properties for historic data management:
  - `historicData`: Cached HourlyAggregate array
  - `historicDataStatus`: 'idle' | 'loading' | 'error' | 'success'
  - `historicDataFetchedAt`: Timestamp when last successfully fetched (null if never)
  - `historicDataError`: Error message string (null if no error)
- New actions:
  - `fetchHistoricData(days)`: Async fetch from Supabase query endpoint with bearer token auth
  - `clearHistoricData()`: Reset historic data to initial state

**MSW Testing Infrastructure** (`client/src/mocks/`):
- **handlers.ts** (111 lines): Query endpoint mock handlers
  - GET `/functions/v1/query` handler with:
    - Bearer token validation (Authorization header)
    - Days parameter validation (7 or 30, defaults to 7)
    - Mock HourlyAggregate response data
    - Error handling: 401 for missing/invalid auth, 400 for invalid parameters
  - Helper functions: `extractBearerToken()`, `parseDaysParam()`

- **server.ts** (26 lines): MSW server setup for Vitest
  - Pre-configured setupServer instance
  - Ready for beforeAll/afterEach/afterAll hooks

- **data.ts**: Mock data generation
  - `MOCK_BEARER_TOKEN`: Test token constant
  - `createQueryResponse()`: Generates mock aggregates
  - `errorResponses`: Error response objects

**Type Definitions** (`client/src/types/events.ts`):
- **HourlyAggregate Interface**: Phase 5 addition
  - `source`: Monitor identifier
  - `date`: ISO date string
  - `hour`: UTC hour (0-23)
  - `eventCount`: Number of events in that hour

**Zustand Store Historic Data Actions**:
- `fetchHistoricData(days)` implementation:
  - Sets status to 'loading'
  - Constructs query URL with Authorization header
  - Fetches from Supabase query endpoint with bearer token
  - Updates historicData, historicDataFetchedAt on success
  - Sets historicDataError on failure, status to 'error'
  - Supports 7 or 30 day lookback periods

**Testing Patterns** (Phase 5):
- MSW intercepts fetch calls in tests
- No real HTTP requests during unit tests
- Query endpoint behavior validated: auth, parameters, response format
- useHistoricData hook can be tested in isolation with mocked data
- Supports integration with heatmap visualization component

**Dependencies** (Phase 5 additions):
- `msw` ^2.12.8 - Mock Service Worker for API mocking
- Uses existing `zustand` for state management
- Uses existing TypeScript and testing libraries

**Environment Variables** (Client - Phase 5):
- No new environment variables required for client
- Bearer token stored in TokenForm component or Zustand store
- Supabase URL derived from deployment or configuration

## Not Yet Integrated

- Database connection pooling from server/monitor
- Background job scheduling for data aggregation
- Event archival/retention policies
- Database backup and disaster recovery
- Heatmap component UI using HourlyAggregate data (Phase 9)

