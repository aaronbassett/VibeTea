# Project Structure

**Status**: Phase 3 core server implementation - Auth, broadcast, routes, and rate limit modules complete
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Directory Layout

```
VibeTea/
├── server/                     # Rust HTTP server and event hub
│   ├── src/
│   │   ├── main.rs            # Server entry point with graceful shutdown (217 lines)
│   │   ├── lib.rs             # Public API exports
│   │   ├── config.rs          # Environment variable configuration (415 lines)
│   │   ├── error.rs           # Error types and conversions (460 lines)
│   │   ├── types.rs           # Event definitions (410 lines)
│   │   ├── routes.rs          # HTTP routes and handlers (1125 lines)
│   │   ├── auth.rs            # Ed25519 verification and token validation (765 lines)
│   │   ├── broadcast.rs       # Event broadcasting with filtering (1041 lines)
│   │   ├── rate_limit.rs      # Token bucket rate limiting (719 lines)
│   │   └── Cargo.toml         # Rust dependencies
│   └── tests/
│       └── unsafe_mode_test.rs # Integration tests (Phase 3)
│
├── monitor/                    # Rust file watcher and event producer
│   ├── src/
│   │   ├── main.rs            # Monitor entry point (placeholder)
│   │   ├── lib.rs             # Public API (types module only)
│   │   ├── config.rs          # Environment variable configuration (303 lines)
│   │   ├── error.rs           # Error types and conversions (173 lines)
│   │   ├── types.rs           # Event definitions and tests (341 lines)
│   │   └── Cargo.toml         # Rust dependencies
│   └── tests/                 # Integration tests (empty in Phase 3)
│
├── client/                     # TypeScript React web dashboard
│   ├── src/
│   │   ├── main.tsx           # React entry point (4 lines)
│   │   ├── App.tsx            # Root component (7 lines, placeholder)
│   │   ├── index.css          # Global styles
│   │   ├── types/
│   │   │   └── events.ts      # TypeScript event types (248 lines)
│   │   ├── hooks/
│   │   │   └── useEventStore.ts # Zustand event store (171 lines)
│   │   ├── components/        # Feature-specific components (empty)
│   │   └── utils/             # Shared utilities (empty)
│   ├── tests/                 # Vitest unit tests (empty)
│   ├── vite.config.ts         # Vite build configuration
│   ├── tsconfig.json          # TypeScript configuration
│   ├── package.json           # Dependencies and scripts
│   └── index.html             # HTML entry point
│
├── .sdd/
│   ├── codebase/
│   │   ├── STACK.md           # Technology stack
│   │   ├── INTEGRATIONS.md    # External services
│   │   ├── ARCHITECTURE.md    # System design patterns
│   │   ├── STRUCTURE.md       # This file
│   │   ├── CONVENTIONS.md     # Code style and naming
│   │   ├── TESTING.md         # Test strategy
│   │   ├── SECURITY.md        # Auth mechanisms
│   │   └── CONCERNS.md        # Tech debt and risks
│   └── memory/                # SDD memory files
│
├── specs/                      # Requirements and specifications
│   └── 001-vibetea/
│       ├── contracts/         # API contracts
│       ├── checklists/        # Task tracking
│       └── retro/             # Retrospectives
│
├── discovery/                  # Design notes and decisions
├── Cargo.toml                 # Workspace root (Rust monorepo)
├── Cargo.lock                 # Dependency lock file
└── target/                    # Rust build artifacts
```

## Key Directories

### `server/src/` - HTTP Server and Event Hub

| File | Purpose | Lines |
|------|---------|-------|
| `main.rs` | Server bootstrap with logging, signal handling, graceful shutdown | 217 |
| `lib.rs` | Module re-exports for public API | ~10 |
| `config.rs` | Environment variable parsing (VIBETEA_PUBLIC_KEYS, VIBETEA_SUBSCRIBER_TOKEN, PORT, VIBETEA_UNSAFE_NO_AUTH) | 415 |
| `error.rs` | Error type definitions and Display implementations | 460 |
| `types.rs` | Event struct and enum definitions with serde | 410 |
| `routes.rs` | HTTP route handlers (POST /events, GET /ws, GET /health) and AppState | 1125 |
| `auth.rs` | Ed25519 signature verification and token validation with constant-time comparison | 765 |
| `broadcast.rs` | Event broadcaster with multi-subscriber support and filtering | 1041 |
| `rate_limit.rs` | Per-source token bucket rate limiting with stale entry cleanup | 719 |

**Configuration Variables**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_PUBLIC_KEYS` | Monitor public keys for signature verification (format: `source1:key1,source2:key2`) | If unsafe mode off | None |
| `VIBETEA_SUBSCRIBER_TOKEN` | Bearer token for client WebSocket connections | If unsafe mode off | None |
| `PORT` | HTTP server port | No | 8080 |
| `VIBETEA_UNSAFE_NO_AUTH` | Disable all authentication (dev only, set to 'true') | No | false |
| `RUST_LOG` | Log level filter (default: info) | No | info,tower_http=debug,axum::rejection=trace |

**Key Types & Constants**:
- `AppState` - Shared state containing config, broadcaster, rate limiter, and start time
- `RateLimitResult` - Enum: Allowed or Limited with retry-after duration
- `RateLimitResult::Allowed` - Request is within limit
- `RateLimitResult::Limited { retry_after_secs }` - Request exceeded limit, include Retry-After header
- `RATE_LIMITER_CLEANUP_INTERVAL` - 30 seconds
- `GRACEFUL_SHUTDOWN_TIMEOUT` - 30 seconds

### `server/src/routes.rs` - HTTP Endpoints

| Handler | Endpoint | Purpose |
|---------|----------|---------|
| `post_events()` | POST /events | Accept events with Ed25519 signature verification |
| `get_ws()` | GET /ws | WebSocket upgrade with token validation |
| `handle_websocket()` | WS connection | Forward events to client with filtering |
| `get_health()` | GET /health | Health check with status, connections, uptime |

**Request/Response Examples**:

POST /events:
```json
// Request body (single or array)
{
  "id": "evt_k7m2n9p4q1r6s3t8u5v0",
  "source": "monitor-1",
  "timestamp": "2026-02-02T14:30:00Z",
  "event_type": "session",
  "payload": { ... }
}

// Response: 202 Accepted (empty body)
// or 401 Unauthorized / 429 Too Many Requests
```

GET /health:
```json
{
  "status": "ok",
  "connections": 42,
  "uptime_seconds": 3600
}
```

### `server/src/auth.rs` - Cryptographic Authentication

| Function | Purpose |
|----------|---------|
| `verify_signature()` | Ed25519 signature verification against request body |
| `validate_token()` | Constant-time bearer token comparison |

**Error Types**:
- `UnknownSource` - Source ID not in configured public keys
- `InvalidSignature` - Signature verification failed
- `InvalidBase64` - Base64 decoding failed for signature or public key
- `InvalidPublicKey` - Public key is malformed or wrong length
- `InvalidToken` - Token mismatch or empty

### `server/src/broadcast.rs` - Event Distribution

| Type | Purpose |
|------|---------|
| `EventBroadcaster` | Central hub for event distribution using tokio broadcast channel |
| `SubscriberFilter` - Optional filtering by source, event type, or project |

**Key Methods**:
- `EventBroadcaster::new()` - Create with default capacity (1000)
- `EventBroadcaster::with_capacity()` - Create with custom capacity
- `EventBroadcaster::broadcast()` - Send event to all subscribers
- `EventBroadcaster::subscribe()` - Get receiver for new connection
- `EventBroadcaster::subscriber_count()` - Get active connections
- `SubscriberFilter::matches()` - Check if event matches all criteria (AND logic)

### `server/src/rate_limit.rs` - Request Rate Limiting

| Type | Purpose |
|------|---------|
| `RateLimiter` | Thread-safe per-source rate limiting |
| `TokenBucket` - Per-source token bucket implementation |

**Constants**:
- `DEFAULT_RATE` - 100.0 tokens per second
- `DEFAULT_CAPACITY` - 100 tokens (burst size)
- `STALE_ENTRY_TIMEOUT` - 60 seconds (cleanup threshold)

**Key Methods**:
- `RateLimiter::new(rate, capacity)` - Create custom limiter
- `RateLimiter::default()` - Create with defaults
- `check_rate_limit(source_id)` - Check if request is allowed
- `cleanup_stale_entries()` - Remove inactive sources
- `spawn_cleanup_task(interval)` - Background cleanup every N seconds
- `source_count()` - Get number of tracked sources

### `monitor/` - File Watcher and Event Producer

| File | Purpose | Lines |
|------|---------|-------|
| `main.rs` | Monitor entry point (placeholder) | ~10 |
| `lib.rs` | Public API (exports types only) | ~10 |
| `config.rs` | Environment variable parsing | 303 |
| `error.rs` | Error types (Config, IO, JSON, HTTP, Crypto, Watch) | 173 |
| `types.rs` | Event definitions with ID generation | 341 |

**Configuration Variables**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | VibeTea server URL (e.g., `https://vibetea.fly.dev`) | Yes | None |
| `VIBETEA_SOURCE_ID` | Monitor identifier | No | System hostname |
| `VIBETEA_KEY_PATH` | Directory containing `key.priv` and `key.pub` | No | `~/.vibetea` |
| `VIBETEA_CLAUDE_DIR` | Claude Code directory to watch | No | `~/.claude` |
| `VIBETEA_BUFFER_SIZE` | Event buffer capacity before flush | No | 1000 |
| `VIBETEA_BASENAME_ALLOWLIST` | Comma-separated file patterns to allow | No | All files |

### `client/src/` - React TypeScript Dashboard

| File | Purpose | Lines |
|------|---------|-------|
| `main.tsx` | React DOM render entry point | 4 |
| `App.tsx` | Root component (placeholder) | 7 |
| `types/events.ts` | TypeScript event type definitions with type guards | 248 |
| `hooks/useEventStore.ts` | Zustand store for event state and session management | 171 |
| `index.css` | Global styles | ~50 |

**Dependencies**:
- React 19.2.4 - UI framework
- TypeScript 5.x - Type safety
- Zustand - State management
- Vite - Build tool and dev server

## Module Boundaries

### Server Module Structure (Phase 3)

**Public API** (`src/lib.rs`):
```rust
pub mod auth;
pub mod broadcast;
pub mod config;
pub mod error;
pub mod rate_limit;
pub mod routes;
pub mod types;
```

**Module Dependencies**:
```
routes.rs
  ├── uses: config, error, types, auth, broadcast, rate_limit
  └── provides: HTTP API and AppState

auth.rs
  ├── uses: config (public_keys, subscriber_token)
  └── provides: verify_signature, validate_token

broadcast.rs
  ├── uses: types (Event)
  └── provides: EventBroadcaster, SubscriberFilter

rate_limit.rs
  ├── uses: (self-contained)
  └── provides: RateLimiter, TokenBucket, RateLimitResult

config.rs
  ├── uses: (environment only)
  └── provides: Config struct

error.rs
  ├── uses: config (error conversions)
  └── provides: Error types

types.rs
  ├── uses: (no dependencies)
  └── provides: Event, EventType, EventPayload

main.rs
  ├── uses: config, routes, rate_limit
  └── provides: Server bootstrap and graceful shutdown
```

**Responsibility Separation**:
- `config` ← Configuration loading and validation from environment
- `error` ← Error type definitions and conversions
- `types` ← Event schema and serialization
- `routes` ← HTTP request/response handling and AppState creation
- `auth` ← Cryptographic verification and token validation
- `broadcast` ← Event distribution to multiple clients
- `rate_limit` ← Per-source request rate limiting
- `main` ← Server startup, logging setup, signal handling, graceful shutdown

### Monitor Module Structure

**Public API** (`src/lib.rs`):
```rust
pub mod types;

pub use types::{Event, EventPayload, EventType, SessionAction, ToolStatus};
```

**Responsibility Separation**:
- `config` ← Configuration loading and validation (internal only)
- `error` ← Error type definitions and conversions (internal only)
- `types` ← Event schema and ID generation (public API)
- `main` ← File watching and event transmission (Phase 3)

### Client Module Structure

**Module Organization**:
- `types/events.ts` - All event type definitions and type guards
- `hooks/useEventStore.ts` - Zustand store for event state and session management
- `components/` - Feature-specific React components (Phase 3+)
- `utils/` - Shared utility functions (Phase 3+)

## Where to Add New Code

| If you're adding... | Put it in... | Example |
|---------------------|--------------|---------|
| New server feature logic | `server/src/{feature}.rs` in public mod or routes | `auth.rs`, `broadcast.rs`, `rate_limit.rs` |
| New HTTP route | `server/src/routes.rs` function | `post_events`, `get_ws`, `get_health` |
| New error type | `server/src/error.rs` enum variant | `AuthError`, `RateLimitError` |
| New monitor capability | `monitor/src/` new module | `monitor/src/watch.rs`, `monitor/src/signing.rs` |
| New React component | `client/src/components/{feature}/` | `client/src/components/sessions/SessionList.tsx` |
| New client hook | `client/src/hooks/` | `client/src/hooks/useWebSocket.ts` |
| New utility function | `client/src/utils/` | `client/src/utils/formatDate.ts` |
| New type definition | `client/src/types/` | `client/src/types/api.ts` |
| Server integration tests | `server/tests/` | `server/tests/unsafe_mode_test.rs` |
| Monitor tests | `monitor/tests/` | `monitor/tests/config_parsing.rs` |

## Import Paths

### Rust (Monorepo Workspace)

All crates are defined in root `Cargo.toml`:
```toml
[workspace]
members = ["server", "monitor"]
```

Within each crate, use relative imports:
```rust
use crate::config::Config;
use crate::error::ServerError;
use crate::types::Event;
use crate::routes::{create_router, AppState};
use crate::auth::verify_signature;
use crate::broadcast::EventBroadcaster;
use crate::rate_limit::RateLimiter;
```

### TypeScript

Configure path aliases in `client/tsconfig.json`:
```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"],
      "@types/*": ["src/types/*"],
      "@hooks/*": ["src/hooks/*"],
      "@components/*": ["src/components/*"],
      "@utils/*": ["src/utils/*"]
    }
  }
}
```

Example usage:
```typescript
import type { VibeteaEvent } from '@types/events';
import { useEventStore } from '@hooks/useEventStore';
import { formatDate } from '@utils/formatDate';
```

## Entry Points

| File | Purpose | Status |
|------|---------|--------|
| `server/src/main.rs` | Server application bootstrap with graceful shutdown | Phase 3 complete |
| `server/src/lib.rs` | Server public library API | Exports all modules |
| `monitor/src/main.rs` | Monitor application bootstrap | Placeholder (Phase 3) |
| `monitor/src/lib.rs` | Monitor public library API | Exports types only |
| `client/src/main.tsx` | React DOM render | Renders App into #root |
| `client/src/App.tsx` | Root React component | Placeholder (Phase 3) |
| `client/index.html` | HTML template | Vite entry point |

## Generated Files

Files that are auto-generated or compile-time artifacts:

| Location | Generator | Notes |
|----------|-----------|-------|
| `target/debug/` | `cargo build` | Rust debug binaries and artifacts |
| `target/release/` | `cargo build --release` | Rust release binaries and artifacts |
| `client/dist/` | `npm run build` (Vite) | Bundled client JavaScript and CSS |
| `Cargo.lock` | `cargo` | Dependency lock file (committed) |

## Phase 3 Implementation Summary

The following modules were added/completed in Phase 3 for the server:

**Server Core Implementation**:
- `src/main.rs` - Complete server entry point with:
  - Structured JSON logging initialization
  - Graceful shutdown handling (SIGTERM/SIGINT)
  - Background rate limiter cleanup task (30s interval)
  - Configuration loading with error reporting
  - TCP listener binding and server startup

- `src/routes.rs` - Complete HTTP API with:
  - POST /events - Event ingestion with signature verification and rate limiting
  - GET /ws - WebSocket subscription with optional filtering
  - GET /health - Health check endpoint
  - AppState shared across handlers
  - Comprehensive test coverage (60+ tests)

- `src/auth.rs` - Complete authentication module with:
  - Ed25519 signature verification (verify_strict for robustness)
  - Constant-time token comparison using subtle crate
  - Detailed error types with classification methods
  - Comprehensive test coverage (40+ tests)

- `src/broadcast.rs` - Complete event distribution with:
  - Tokio broadcast channel for multi-subscriber support
  - EventBroadcaster with configurable capacity
  - SubscriberFilter with source/type/project filtering
  - Comprehensive test coverage (50+ tests)

- `src/rate_limit.rs` - Complete rate limiting with:
  - Token bucket algorithm per source
  - Configurable rate and capacity
  - Automatic stale entry cleanup
  - Background cleanup task spawning
  - Comprehensive test coverage (20+ tests)

**Testing**:
- `server/tests/unsafe_mode_test.rs` - Integration tests in unsafe mode

---

*This document shows WHERE code lives. Update when directory structure or module organization changes.*
