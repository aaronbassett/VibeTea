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
│   │   ├── crypto.rs          # Ed25519 keypair generation/management with key loading strategies
│   │   ├── sender.rs          # HTTP client with retry and buffering
│   │   ├── types.rs           # Event type definitions
│   │   └── error.rs           # Error types
│   ├── tests/
│   │   ├── privacy_test.rs    # Privacy filtering tests
│   │   ├── sender_recovery_test.rs  # Retry logic tests
│   │   └── env_key_test.rs    # Environment variable key loading tests
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
| `main.rs` | CLI entry (init/run commands), signal handling | `Cli`, `Command` |
| `config.rs` | Load from env vars: `VIBETEA_*` | `Config` |
| `watcher.rs` | inotify/FSEvents for `~/.claude/projects/**/*.jsonl` | `FileWatcher`, `WatchEvent` |
| `parser.rs` | Parse JSONL, extract Session/Activity/Tool events | `SessionParser`, `ParsedEvent`, `ParsedEventKind` |
| `privacy.rs` | Remove code, prompts, sensitive data | `PrivacyPipeline`, `PrivacyConfig` |
| `crypto.rs` | Ed25519 keypair with dual loading strategy (env var + file fallback) | `Crypto`, `KeySource`, `CryptoError` |
| `sender.rs` | HTTP POST to server with retry/buffering | `Sender`, `SenderConfig`, `RetryPolicy` |
| `types.rs` | Event schema (shared with server) | `Event`, `EventPayload`, `EventType` |
| `error.rs` | Error types | `MonitorError`, custom errors |

### Crypto Module Details (`monitor/src/crypto.rs`)

The crypto module provides Ed25519 key management with flexible loading strategies:

| Method | Purpose | Returns |
|--------|---------|---------|
| `Crypto::generate()` | Generate new Ed25519 keypair using OS RNG | `Crypto` instance |
| `Crypto::load_from_env()` | Load 32-byte seed from `VIBETEA_PRIVATE_KEY` env var | `(Crypto, KeySource::EnvironmentVariable)` |
| `Crypto::load_with_fallback(dir)` | Try env var first, fallback to file if not set | `(Crypto, KeySource)` |
| `Crypto::load(dir)` | Load from file only (`dir/key.priv`) | `Crypto` instance |
| `Crypto::save(dir)` | Save keypair to files (mode 0600/0644) | `Result<()>` |
| `Crypto::public_key_base64()` | Get public key as base64 (RFC 4648) | `String` |
| `Crypto::public_key_fingerprint()` | Get first 8 chars of public key (for logging) | `String` |
| `Crypto::seed_base64()` | Export seed as base64 (for `VIBETEA_PRIVATE_KEY`) | `String` |
| `Crypto::sign(message)` | Sign message, return base64 signature | `String` |
| `Crypto::sign_raw(message)` | Sign message, return raw 64-byte signature | `[u8; 64]` |

**Key Loading Behavior:**
- `load_with_fallback()` used in `monitor/src/main.rs` at startup (see lines 183-187)
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
1. **Watch** files via `FileWatcher`
2. **Parse** JSONL via `SessionParser`
3. **Filter** events via `PrivacyPipeline`
4. **Sign** events via `Crypto` (with dual-source key loading)
5. **Send** to server via `Sender`

No cross-dependencies with Server or Client.

```
monitor/src/main.rs
├── config.rs (load env)
├── crypto.rs (load keys from env var OR file, track KeySource)
├── watcher.rs → sender.rs
│   ↓
├── parser.rs → privacy.rs
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
| **New key loading method** | `monitor/src/crypto.rs` (add method to `Crypto`) | `Crypto::load_from_stdin()` |
| **New Server endpoint** | `server/src/routes.rs` (add route handler) | `POST /events/:id/ack` |
| **New Server middleware** | `server/src/routes.rs` or `server/src/` (new module) | `server/src/middleware.rs` |
| **New event type** | `server/src/types.rs` + `monitor/src/types.rs` (sync both) | New `EventPayload` variant |
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
use vibetea_monitor::crypto::{Crypto, KeySource};
use vibetea_monitor::watcher::FileWatcher;
use vibetea_monitor::sender::Sender;
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
| Module names | `snake_case` | `parser.rs`, `privacy.rs` |
| Type names | `PascalCase` | `Event`, `ParsedEvent`, `EventPayload` |
| Function names | `snake_case` | `verify_signature()`, `calculate_backoff()` |
| Constant names | `UPPER_SNAKE_CASE` | `MAX_BODY_SIZE`, `EVENT_ID_PREFIX` |
| Test functions | `#[test]` or `_test.rs` suffix | `privacy_test.rs` |
| Enum variants | `PascalCase` | `KeySource::EnvironmentVariable`, `KeySource::File` |

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
✓ CAN import:     types, config, crypto, watcher, parser, privacy, sender, error
✓ CAN import:     std, tokio, serde, ed25519-dalek, notify, reqwest, zeroize
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

## Test Organization

### Monitor Tests

Located in `monitor/tests/` with `serial_test` crate for environment variable safety:

| File | Purpose | Key Pattern |
|------|---------|-------------|
| `env_key_test.rs` | Environment variable key loading (FR-001 through FR-028) | `#[test] #[serial]` |
| `privacy_test.rs` | Privacy filtering validation | — |
| `sender_recovery_test.rs` | Retry logic and buffering | — |

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
