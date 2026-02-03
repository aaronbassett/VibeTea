# Technology Stack

**Status**: Phase 11 Implementation - Supabase persistence with PostgreSQL and Edge Functions
**Last Updated**: 2026-02-03

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, JSONL parsing, privacy filtering, event signing, HTTP transmission |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization |
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
| @noble/ed25519             | ^2.0.0   | Ed25519 signature verification in Edge Functions (RFC 8032 compliant) |

### Supabase & PostgreSQL (Phase 11)

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

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit/component testing framework |
| @testing-library/react | ^16.3.2  | React testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM matchers for testing |
| jsdom                  | ^28.0.0  | DOM implementation for Node.js |

## Configuration Files

| File | Framework | Purpose |
|------|-----------|---------|
| `client/vite.config.ts` | Vite | Build configuration, WebSocket proxy to server on port 8080 |
| `client/tsconfig.json` | TypeScript | Strict mode, ES2020 target |
| `client/eslint.config.js` | ESLint | Flat config format with TypeScript support |
| `Cargo.toml` (workspace) | Cargo | Rust workspace configuration and shared dependencies |
| `server/Cargo.toml` | Cargo | Server package configuration |
| `monitor/Cargo.toml` | Cargo | Monitor package configuration |
| `supabase/config.toml` | Supabase CLI | Local development environment configuration (PostgreSQL 17, Deno 2 runtime) |

## Runtime Environment

| Aspect | Details |
|--------|---------|
| Server Runtime | Rust binary (tokio async) |
| Client Runtime | Browser (ES2020+) |
| Monitor Runtime | Native binary (Linux/macOS/Windows) with CLI |
| Supabase Functions | Deno 2 JavaScript runtime |
| Database Runtime | PostgreSQL 17 with PostgREST API |
| Node.js | Required for development and client build only |
| Async Model | Tokio (Rust), Promises (TypeScript/Deno) |
| WebSocket Support | Native (server-side via axum, client-side via browser) |
| WebSocket Proxy | Vite dev server proxies /ws to localhost:8080 |
| File System Monitoring | Rust notify crate (inotify/FSEvents) for JSONL tracking |
| CLI Support | Manual command parsing in monitor main.rs (init, run, help, version) |
| Local Supabase | Docker-based with PostgreSQL, PostgREST, Deno runtime, Auth (port 54321) |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature with X-Signature header |
| Server → Client | WebSocket | JSON | Bearer token |
| Client → Server | WebSocket | JSON | Bearer token |
| Monitor → File System | Native | JSONL | N/A (local file access) |
| Client → Supabase Functions | HTTPS POST | JSON | Bearer token (query endpoint) |
| Monitor → Supabase Functions | HTTPS POST | JSON | Ed25519 signature (ingest endpoint) |
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

## Build Output

| Component | Output | Format | Deployment |
|-----------|--------|--------|-----------|
| Server | Binary | ELF (Linux) | Docker container on Fly.io |
| Monitor | Binary | ELF/Mach-O/PE | Standalone executable for users |
| Client | Static files | JS + CSS (Brotli compressed) | CDN (Netlify/Vercel/Cloudflare) |
| Supabase Functions | TypeScript | Deno-compatible JavaScript | Hosted on Supabase Edge Functions |

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
- `parser.rs` - Claude Code JSONL parser (privacy-first metadata extraction)
- `watcher.rs` - File system watcher for `.claude/projects/**/*.jsonl` files with position tracking
- `privacy.rs` - **Phase 5**: Privacy pipeline for event sanitization before transmission
- `crypto.rs` - **Phase 6**: Ed25519 keypair generation, loading, saving, and event signing
- `sender.rs` - **Phase 6**: HTTP client with event buffering, exponential backoff retry, and rate limit handling
- `main.rs` - **Phase 6**: CLI entry point with init and run commands
- `lib.rs` - Public interface

### Supabase (`supabase/`)
- `config.toml` - **Phase 11**: Local development environment configuration (PostgreSQL 17, Deno 2, API on port 54321)
- `migrations/` - **Phase 11**: Database schema and function definitions
  - `20260203000000_create_events_table.sql` - Events table with JSONB payload, indexes, RLS
  - `20260203000001_create_functions.sql` - PostgreSQL functions: `bulk_insert_events()`, `get_hourly_aggregates()`
- `functions/` - **Phase 11**: Edge Functions (Deno TypeScript)
  - `_shared/auth.ts` - Ed25519 signature verification using @noble/ed25519, bearer token validation

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary, minimal base image |
| Client | CDN | Static files | Optimized builds with compression |
| Monitor | Local | Native binary | Users download and run locally |
| Supabase Functions | Supabase Hosted | Deno Container | Auto-deployed from `supabase/functions/` |
| Database | Supabase Hosted | PostgreSQL Container | Managed PostgreSQL 17 instance |

## Phase 11 Additions (Supabase Persistence)

**Phase 11 Database Schema** (`supabase/migrations/20260203000000_create_events_table.sql`):
- **events table**: Stores event persistence with columns:
  - `id TEXT PRIMARY KEY` - Event identifier (evt_<20-char>)
  - `source TEXT NOT NULL` - Monitor source identifier
  - `timestamp TIMESTAMPTZ NOT NULL` - Event occurrence time
  - `event_type TEXT NOT NULL` - Enum: session, activity, tool, agent, summary, error
  - `payload JSONB NOT NULL` - Full event payload (privacy-filtered)
  - `created_at TIMESTAMPTZ` - Persistence timestamp
- **Indexes**: Time-range queries (DESC), source filtering, composite source+timestamp
- **Row Level Security**: Enabled with deny-all (service_role only access)
- **Capacity**: Designed for high-volume event ingestion

**Phase 11 PostgreSQL Functions** (`supabase/migrations/20260203000001_create_functions.sql`):
- **bulk_insert_events(events_json JSONB)**: Atomic batch insertion with ON CONFLICT idempotency
  - Accepts array of events as JSONB
  - Parses JSON fields: id, source, timestamp, eventType, payload
  - Returns count of successfully inserted events
  - Granted to service_role only
- **get_hourly_aggregates(days_back INT, source_filter TEXT)**: Hourly event count aggregation
  - Retrieves hourly event counts for heatmap visualization
  - Supports 7-day (default) or 30-day lookback
  - Optional source filtering
  - Returns (source, date, hour, event_count) sorted descending
  - Granted to service_role only

**Phase 11 Supabase Edge Functions** (`supabase/functions/_shared/auth.ts`):
- **@noble/ed25519**: RFC 8032 compliant Ed25519 signature verification (version 2.0.0)
  - Uses `ed.verifyAsync()` for async verification
  - Validates key length (32 bytes) and signature length (64 bytes)
  - Returns boolean verification result
- **verifySignature()**: Base64-encoded signature verification
  - Decodes public key and signature from base64
  - Validates cryptographic formats
  - Integrates with monitor's signing in `monitor/src/crypto.rs`
- **getPublicKeyForSource()**: Source-specific public key lookup
  - Parses `VIBETEA_PUBLIC_KEYS` environment variable
  - Format: `source_id:public_key,source_id2:public_key2` (comma-separated)
  - Returns public key or null if not found
- **validateBearerToken()**: Bearer token validation for client authentication
  - Parses `Authorization: Bearer <token>` header
  - Constant-time string comparison (Deno limitation noted in code)
  - Returns boolean validation result
- **verifyIngestAuth()**: Combined Ed25519 + X-Source-ID header authentication
  - Extracts X-Source-ID header
  - Extracts X-Signature header
  - Verifies signature against registered public key
  - Returns AuthResult with isValid and optional error/sourceId
- **verifyQueryAuth()**: Bearer token authentication for query endpoints
  - Extracts Authorization header
  - Validates bearer token
  - Returns AuthResult

**Supabase Configuration** (`supabase/config.toml`):
- **Database**: PostgreSQL 17 on port 54322 (shadow db 54320)
- **API**: PostgREST on port 54321, schemas: public, graphql_public
- **Studio**: Web UI on port 54323
- **Edge Runtime**: Deno 2, hot reload policy per_worker
- **Auth**: Enabled with JWT (3600s expiry), email signup enabled
- **Email**: Inbucket test server on port 54324
- **Storage**: S3 protocol support enabled

## Not Yet Integrated

- Supabase Edge Functions deployed to production
- Client persistence queries via Supabase API
- Database connection pooling from server/monitor
- Background job scheduling for data aggregation
- Event archival/retention policies
- Database backup and disaster recovery
