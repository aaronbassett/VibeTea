# Project Structure

**Status**: Phase 1-2 scaffolding complete with core modules
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Directory Layout

```
VibeTea/
├── server/                     # Rust HTTP server and event hub
│   ├── src/
│   │   ├── main.rs            # Server entry point (placeholder)
│   │   ├── lib.rs             # Public API exports
│   │   ├── config.rs          # Environment variable configuration (200+ lines)
│   │   ├── error.rs           # Error types and conversions (460+ lines)
│   │   ├── types.rs           # Event definitions and tests (410+ lines)
│   │   └── Cargo.toml         # Rust dependencies
│   └── tests/                 # Integration tests (empty in Phase 2)
│
├── monitor/                    # Rust file watcher and event producer
│   ├── src/
│   │   ├── main.rs            # Monitor entry point (placeholder)
│   │   ├── lib.rs             # Public exports (types, config)
│   │   ├── config.rs          # Environment variable configuration (300+ lines)
│   │   ├── error.rs           # Error types and conversions (170+ lines)
│   │   ├── types.rs           # Event definitions and tests (340+ lines)
│   │   └── Cargo.toml         # Rust dependencies
│   └── tests/                 # Integration tests (empty in Phase 2)
│
├── client/                     # TypeScript React web dashboard
│   ├── src/
│   │   ├── main.tsx           # React entry point (4 lines)
│   │   ├── App.tsx            # Root component (7 lines, placeholder)
│   │   ├── index.css          # Global styles
│   │   ├── types/
│   │   │   └── events.ts      # TypeScript event types (249 lines)
│   │   ├── hooks/
│   │   │   └── useEventStore.ts # Zustand event store (172 lines)
│   │   ├── components/        # Feature-specific components (empty in Phase 2)
│   │   └── utils/             # Shared utilities (empty in Phase 2)
│   ├── tests/                 # Vitest unit tests (empty in Phase 2)
│   ├── vite.config.ts         # Vite build configuration
│   ├── tsconfig.json          # TypeScript configuration
│   ├── package.json           # Dependencies and scripts
│   └── index.html             # HTML entry point
│
├── .sdd/
│   ├── codebase/
│   │   ├── STACK.md           # Technology stack (Phase 1)
│   │   ├── ARCHITECTURE.md    # System design patterns (Phase 2)
│   │   └── STRUCTURE.md       # This file
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

### `server/` - HTTP Server and Event Hub

| Directory | Purpose | Naming Convention |
|-----------|---------|-------------------|
| `src/` | Rust source code | `{module}.rs` |
| `src/config.rs` | Environment variable parsing with validation (PUBLIC_KEYS, SUBSCRIBER_TOKEN, PORT) | Single module |
| `src/error.rs` | Error type definitions and conversions (ConfigError, ServerError) | Single module |
| `src/types.rs` | Event struct and enum definitions with tests (Event, EventType, EventPayload) | Single module |
| `tests/` | Integration tests (to be populated Phase 3) | `{feature}_test.rs` |

**Configuration**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_PUBLIC_KEYS` | Monitor public keys for signature verification (format: `source1:key1,source2:key2`) | If unsafe mode off | None |
| `VIBETEA_SUBSCRIBER_TOKEN` | Bearer token for client WebSocket connections | If unsafe mode off | None |
| `PORT` | HTTP server port | No | 8080 |
| `VIBETEA_UNSAFE_NO_AUTH` | Disable all authentication (dev only) | No | false |

### `monitor/` - File Watcher and Event Producer

| Directory | Purpose | Naming Convention |
|-----------|---------|-------------------|
| `src/` | Rust source code | `{module}.rs` |
| `src/config.rs` | Environment variable parsing (SERVER_URL, SOURCE_ID, KEY_PATH, CLAUDE_DIR, BUFFER_SIZE, ALLOWLIST) | Single module |
| `src/error.rs` | Error type definitions (Config, IO, JSON, HTTP, Crypto, Watch) | Single module |
| `src/types.rs` | Event definitions with ID generation (Event, EventType, EventPayload, SessionAction, ToolStatus) | Single module |
| `tests/` | Integration tests (to be populated Phase 3) | `{feature}_test.rs` |

**Configuration**:
| Variable | Purpose | Required | Default |
|----------|---------|----------|---------|
| `VIBETEA_SERVER_URL` | VibeTea server URL (e.g., `https://vibetea.fly.dev`) | Yes | None |
| `VIBETEA_SOURCE_ID` | Monitor identifier (must match server public key registration) | No | System hostname |
| `VIBETEA_KEY_PATH` | Directory containing `key.priv` and `key.pub` | No | `~/.vibetea` |
| `VIBETEA_CLAUDE_DIR` | Claude Code directory to watch | No | `~/.claude` |
| `VIBETEA_BUFFER_SIZE` | Event buffer capacity before flush | No | 1000 |
| `VIBETEA_BASENAME_ALLOWLIST` | Comma-separated file patterns to allow (e.g., `jsonl,json,log`) | No | All files |

### `client/` - React TypeScript Dashboard

| Directory | Purpose | Naming Convention |
|-----------|---------|-------------------|
| `src/` | TypeScript/React source code | `{name}.ts{x}` |
| `src/main.tsx` | React DOM render entry point | Single file |
| `src/App.tsx` | Root component | Single file |
| `src/types/` | Type definitions | `{domain}.ts` |
| `src/types/events.ts` | Event types and type guards | Single file (249 lines) |
| `src/hooks/` | Custom React hooks | `use{Hook}.ts` |
| `src/hooks/useEventStore.ts` | Zustand event store (status, events, sessions) | Single file (172 lines) |
| `src/components/` | React components by feature | `{Feature}/{Component}.tsx` |
| `src/utils/` | Utility functions | `{purpose}.ts` |
| `tests/` | Vitest unit tests | `{module}.test.ts` |

**Dependencies**:
- React 19.2.4 - UI framework
- TypeScript 5.x - Type safety
- Zustand - State management
- Tailwind CSS - Utility-first styling (via index.css)
- Vite - Build tool and dev server

## Module Boundaries

### Server Module Structure

**Public API** (`src/lib.rs`):
```rust
pub mod config;
pub mod error;
pub mod types;
```

**Module Dependencies**:
- `config` - No dependencies on other modules
- `error` - Imports from `config` only
- `types` - No dependencies on other modules
- `main` - Imports from `config`, `error`, `types`

**Responsibility Separation**:
- `config` ← Configuration loading and validation
- `error` ← Error type definitions and conversions
- `types` ← Event schema and serialization (Phase 3: routing, state)
- `main` ← Server startup and WebSocket handling (Phase 3)

### Monitor Module Structure

**Public API** (`src/lib.rs`):
```rust
pub mod config;
pub mod error;
pub mod types;

pub use types::{Event, EventPayload, EventType, SessionAction, ToolStatus};
```

**Module Dependencies**:
- `config` - No dependencies on other modules
- `error` - Imports from `config` only
- `types` - No dependencies on other modules
- `main` - Imports from `config`, `error`, `types`

**Responsibility Separation**:
- `config` ← Configuration loading and validation
- `error` ← Error type definitions and conversions
- `types` ← Event schema and ID generation
- `main` ← File watching and event transmission (Phase 3)

### Client Module Structure

**Module Organization**:
- `types/events.ts` - All event type definitions and type guards
- `hooks/useEventStore.ts` - Zustand store for event state and session management
- `components/` - Feature-specific React components (to be created Phase 3)
- `utils/` - Shared utility functions (to be created Phase 3)

**Export Structure**:
- `hooks/useEventStore.ts` exports `useEventStore` hook and selector utilities
- `types/events.ts` exports all types and type guard functions
- `App.tsx` imports from both types and hooks

## Where to Add New Code

| If you're adding... | Put it in... | Example |
|---------------------|--------------|---------|
| New server API route | `server/src/` (Phase 3: new `routes/` submodule) | `routes/events.rs` |
| New server handler | `server/src/` (Phase 3: new `handlers/` submodule) | `handlers/websocket.rs` |
| New monitor capability | `monitor/src/` (Phase 3: new `watch/` submodule) | `watch/file_watcher.rs` |
| New React component | `client/src/components/{feature}/` | `client/src/components/sessions/SessionList.tsx` |
| New client hook | `client/src/hooks/` | `client/src/hooks/useWebSocket.ts` |
| New utility function | `client/src/utils/` | `client/src/utils/formatDate.ts` |
| New type definition | `client/src/types/` | `client/src/types/api.ts` |
| Server tests | `server/tests/` | `server/tests/event_validation.rs` |
| Monitor tests | `monitor/tests/` | `monitor/tests/config_parsing.rs` |
| Client tests | `client/tests/` | `client/tests/hooks/useEventStore.test.ts` |

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
| `server/src/main.rs` | Server application bootstrap | Placeholder (Phase 3) |
| `server/src/lib.rs` | Server public library API | Exports config, error, types |
| `monitor/src/main.rs` | Monitor application bootstrap | Placeholder (Phase 3) |
| `monitor/src/lib.rs` | Monitor public library API | Exports types and re-exports key types |
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

## Phase 2 Additions

The following modules were added in Phase 2 for all components:

**Server**:
- `src/config.rs` - Full environment variable parsing with tests
- `src/error.rs` - Comprehensive error hierarchy with display implementations
- `src/types.rs` - Complete event schema with serialization/deserialization tests

**Monitor**:
- `src/config.rs` - Full environment variable parsing with tests
- `src/error.rs` - Error types covering config, IO, JSON, HTTP, crypto, and file watching
- `src/types.rs` - Event schema with ID generation and serialization tests

**Client**:
- `src/types/events.ts` - Full TypeScript event types with type guards
- `src/hooks/useEventStore.ts` - Zustand store with event buffering and session aggregation

## Phase 3 Additions (Planned)

The following modules are planned for Phase 3 implementation:

**Server**:
- `src/routes/` - HTTP route handlers
- `src/websocket/` - WebSocket connection management and broadcasting
- `src/auth/` - Signature verification and token validation
- `src/handlers/` - Request/response handlers

**Monitor**:
- `src/watch/` - File system watcher using `notify` crate
- `src/signing/` - Ed25519 signature generation
- `src/client/` - HTTP client for server communication
- `src/buffer/` - Event buffer and batching logic

**Client**:
- `src/components/` - React components (Sessions, Events, Dashboard)
- `src/utils/` - Formatting and helper functions
- Integration with WebSocket connection

---

*This document shows WHERE code lives. Update when directory structure changes.*
