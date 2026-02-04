# Project Structure

> **Purpose**: Document directory layout, module boundaries, and where to add new code.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Directory Layout

```
VibeTea/
├── server/                  # Rust HTTP server (event hub)
│   ├── src/
│   │   ├── main.rs         # Server entry point (startup, logging, graceful shutdown)
│   │   ├── lib.rs          # Library root with module declarations
│   │   ├── routes.rs       # HTTP route handlers (POST /events, GET /ws, GET /health)
│   │   ├── auth.rs         # Ed25519 signature verification and token validation
│   │   ├── broadcast.rs    # Event broadcaster with Tokio broadcast channel
│   │   ├── rate_limit.rs   # Per-source rate limiting
│   │   ├── config.rs       # Environment variable configuration
│   │   ├── error.rs        # Error types and HTTP status mapping
│   │   └── types.rs        # Shared event types (EventType, EventPayload, etc.)
│   ├── tests/              # Integration tests for server
│   └── Cargo.toml          # Package manifest (inherits from workspace)
├── monitor/                 # Rust CLI monitor (event producer)
│   ├── src/
│   │   ├── main.rs         # Monitor entry point (CLI, init and run commands)
│   │   ├── lib.rs          # Library root with module declarations
│   │   ├── watcher.rs      # File system watcher for session files
│   │   ├── parser.rs       # JSONL parsing from Claude Code sessions
│   │   ├── privacy.rs      # Event sanitization pipeline
│   │   ├── crypto.rs       # Ed25519 keypair and signing operations
│   │   ├── sender.rs       # HTTP client with buffering and retries
│   │   ├── config.rs       # Environment variable configuration
│   │   ├── types.rs        # Event type definitions
│   │   └── error.rs        # Error types
│   ├── tests/              # Integration tests for monitor
│   └── Cargo.toml          # Package manifest (inherits from workspace)
├── client/                  # React TypeScript web dashboard
│   ├── src/
│   │   ├── main.tsx        # Vite entry point
│   │   ├── App.tsx         # Root React component (state coordination, layout)
│   │   ├── components/     # Presentational components
│   │   │   ├── ConnectionStatus.tsx     # Connection indicator and status
│   │   │   ├── EventStream.tsx          # Live event log display
│   │   │   ├── Heatmap.tsx              # Activity heatmap visualization
│   │   │   ├── SessionOverview.tsx      # Summary of active sessions
│   │   │   └── TokenForm.tsx            # Authentication token input
│   │   ├── hooks/          # Custom React hooks
│   │   │   ├── useWebSocket.ts          # WebSocket connection management
│   │   │   ├── useEventStore.ts         # Zustand event store
│   │   │   └── useSessionTimeouts.ts    # Session inactivity detection
│   │   ├── types/          # TypeScript type definitions
│   │   │   └── events.ts               # Event type definitions (mirrors server)
│   │   ├── utils/          # Utility functions
│   │   │   └── formatting.ts           # Time and data formatting
│   │   ├── __tests__/      # Unit tests
│   │   └── vite-env.d.ts   # Vite environment types
│   ├── dist/               # Built artifacts (Vite output)
│   ├── tests/              # Integration tests
│   ├── package.json        # Dependencies (React, Zustand, TailwindCSS, Vite)
│   ├── vite.config.ts      # Vite build configuration
│   ├── tsconfig.json       # TypeScript configuration
│   ├── tailwind.config.ts  # TailwindCSS configuration
│   └── eslint.config.js    # ESLint configuration
├── specs/                   # Feature specifications and requirements
│   ├── 001-vibetea/        # Core feature specification
│   ├── 002-supabase-auth/  # Supabase authentication spec
│   ├── 002-client-frontend-redesign/ # UI redesign spec
│   ├── 003-monitor-tui/    # Terminal UI spec
│   ├── 004-monitor-gh-actions/ # GitHub Actions monitoring
│   └── 005-monitor-enhanced-tracking/ # Enhanced tracking features
├── .sdd/                    # System Design Document
│   ├── codebase/           # Codebase documentation (this directory)
│   │   ├── ARCHITECTURE.md # System design and patterns
│   │   ├── STRUCTURE.md    # Directory layout (this file)
│   │   ├── STACK.md        # Technology stack
│   │   ├── INTEGRATIONS.md # External services
│   │   ├── CONVENTIONS.md  # Code style and naming
│   │   ├── TESTING.md      # Test strategy
│   │   ├── SECURITY.md     # Auth and security
│   │   └── CONCERNS.md     # Tech debt and risks
│   └── memory/             # Persistent project memory
├── Cargo.toml              # Workspace manifest (server, monitor)
├── fly.toml                # Fly.io deployment configuration
├── Dockerfile              # Container image definition
├── ralph.toml              # Ralph project configuration
├── CLAUDE.md               # Critical learnings and patterns
└── README.md               # Project overview

```

## Key Directories

### `server/src/` - Event Hub Implementation

| File | Purpose | Responsibility |
|------|---------|-----------------|
| `main.rs` | Server binary entry point | Startup, config loading, signal handling, graceful shutdown |
| `lib.rs` | Library root | Module declarations for reusability |
| `routes.rs` | HTTP route definitions | Handlers for POST /events, GET /ws, GET /health |
| `auth.rs` | Authentication logic | Ed25519 signature verification, token validation |
| `broadcast.rs` | Event distribution | Tokio broadcast channel, subscriber filtering |
| `rate_limit.rs` | Request throttling | Per-source rate limit tracking with cleanup |
| `config.rs` | Configuration management | Environment variable parsing, validation |
| `error.rs` | Error handling | Custom error types, HTTP status mapping |
| `types.rs` | Data models | EventType, EventPayload, Event, etc. |

### `monitor/src/` - Session Monitoring Implementation

| File | Purpose | Responsibility |
|------|---------|-----------------|
| `main.rs` | Monitor binary entry point | CLI parsing, init/run subcommands, event loop |
| `lib.rs` | Library root | Module declarations for reusability |
| `watcher.rs` | File system watching | Detects changes in `~/.claude/projects/**/*.jsonl` |
| `parser.rs` | JSONL parsing | Extracts session events from Claude Code logs |
| `privacy.rs` | Data sanitization | Removes sensitive content, retains metadata only |
| `crypto.rs` | Cryptographic operations | Generates Ed25519 keypairs, signs payloads |
| `sender.rs` | HTTP transmission | Buffers events, handles retries, manages backoff |
| `config.rs` | Configuration management | Environment variable parsing, defaults |
| `types.rs` | Event definitions | Event, EventPayload, EventType enums |
| `error.rs` | Error handling | Custom error types for monitor operations |

### `client/src/` - React Dashboard

| Directory | Purpose | Naming Convention |
|-----------|---------|-------------------|
| `components/` | React UI components | PascalCase files, export React.FC<Props> |
| `hooks/` | Custom React hooks | camelCase files, export useHookName functions |
| `types/` | TypeScript types | PascalCase for types, types.ts or domain.ts |
| `utils/` | Helper functions | camelCase files, export utility functions |
| `__tests__/` | Unit tests | `{component}.test.tsx`, `{util}.test.ts` |

### `tests/` Directories

Integration tests are located in each package:
- `server/tests/` - Server integration tests
- `monitor/tests/` - Monitor integration tests
- `client/tests/` - Client E2E tests

## Module Boundaries

### Server Modules

The server is organized as layers that flow downward:

```
routes (HTTP handlers)
  ↓ uses
auth + broadcast + rate_limit
  ↓ uses
types + config + error
```

**Module Access Rules:**
- `routes` can access: `auth`, `broadcast`, `rate_limit`, `config`, `types`, `error`
- `auth` can access: `config`, `types`, `error` (NO access to routes or broadcast)
- `broadcast` can access: `types`, `error` (independent utility)
- `rate_limit` can access: nothing (independent utility)
- `config` has no dependencies (data only)

### Monitor Modules

The monitor forms a pipeline for event processing:

```
watcher (file system) → parser (JSONL) → privacy (sanitize) → sender (HTTP)
                                               ↓
                                            types
                                               ↓
                                            crypto
```

**Module Access Rules:**
- `watcher` creates events from file changes, passes to parser
- `parser` creates ParsedEvent from JSONL, passes to main loop
- `privacy` sanitizes EventPayload, returns sanitized event
- `sender` queues events, handles HTTP transmission with retry
- `crypto` provides keypair and signing (used by sender)
- All modules can access `config`, `types`, `error`

### Client Module Hierarchy

```
App (root coordinator)
  ↓
hooks/ (state management)
  - useWebSocket (connection)
  - useEventStore (Zustand store)
  - useSessionTimeouts (inactivity)
  ↓
components/ (presentational, receive props from hooks)
  - EventStream
  - Heatmap
  - SessionOverview
  - ConnectionStatus
  - TokenForm
  ↓
utils/ (pure functions)
  - formatting (timestamp, data display)
  ↓
types/ (TypeScript definitions)
```

## Where to Add New Code

### Adding a New Server API Endpoint

1. **Route definition**: Add handler to `server/src/routes.rs`
2. **Business logic**: Create new module if needed (e.g., `server/src/billing.rs`)
3. **Types**: Add variant to `EventType` or new payload in `server/src/types.rs` if needed
4. **Auth**: If requires authentication, call `auth::verify_signature()` in handler
5. **Rate limiting**: If needs limiting, check `rate_limit` in handler

Example structure:
```
server/src/
├── routes.rs (add route: `router.post("/new-endpoint", handle_new)`)
├── billing.rs (NEW: business logic)
└── error.rs (add error variant if needed)
```

### Adding a New Event Type

1. **Server types**: Add variant to `EventType` enum in `server/src/types.rs`
2. **Payload**: Add variant to `EventPayload` in `server/src/types.rs`
3. **Monitor support**: Add parsing in `monitor/src/parser.rs`
4. **Privacy filtering**: Add rules in `monitor/src/privacy.rs`
5. **Client display**: Add component in `client/src/components/`

### Adding a New React Component

1. **Create file**: `client/src/components/YourComponent.tsx` (PascalCase)
2. **Define props**: Use TypeScript interface extending React.HTMLAttributes
3. **Export**: `export const YourComponent: React.FC<YourComponentProps> = ...`
4. **Use in App**: Import and add to `client/src/App.tsx` or parent component
5. **Test**: Create `client/src/__tests__/YourComponent.test.tsx`

### Adding a New Hook

1. **Create file**: `client/src/hooks/useYourHook.ts` (camelCase)
2. **Define return type**: TypeScript interface for returned value
3. **Export**: `export function useYourHook(): ReturnType { ... }`
4. **Use in component**: Import and call in component body (before other hooks)
5. **Test**: Create `client/src/__tests__/useYourHook.test.ts`

## Import Paths

| Alias | Maps To | Usage | Example |
|-------|---------|-------|---------|
| (none) | Relative imports | Within same package | `use crate::types::Event;` (Rust) |
| `@/` | `client/src/` | Client cross-module | `import { App } from '@/App';` |
| (none) | Relative imports | Client cross-module | `import { Event } from '../types/events';` |

## Entry Points

| File | Purpose | How to Run |
|------|---------|-----------|
| `server/src/main.rs` | Server binary | `cargo run --bin vibetea-server` |
| `monitor/src/main.rs` | Monitor binary | `cargo run --bin vibetea-monitor -- init` |
| `client/src/main.tsx` | Client app | `npm run dev` |

## Generated Files

Files generated automatically and should NOT be manually edited:

| Location | Generator | Regenerate Command |
|----------|-----------|-------------------|
| `client/dist/` | Vite | `npm run build` |
| `target/` | Cargo | `cargo build` |
| `target/doc/` | Rustdoc | `cargo doc --no-deps` |

## Configuration Files

### Workspace Level (`Cargo.toml`)

- Defines Rust workspace members: `server`, `monitor`
- Declares shared dependencies with versions
- Sets release profile optimization

### Package Level

- `server/Cargo.toml` - Server-specific dependencies
- `monitor/Cargo.toml` - Monitor-specific dependencies
- `client/package.json` - Node.js dependencies
- `server/src/config.rs` - Runtime environment variables
- `monitor/src/config.rs` - Runtime environment variables

### Build Configuration

- `Dockerfile` - Container image for server deployment
- `fly.toml` - Fly.io deployment manifest
- `vite.config.ts` - Client build configuration
- `tsconfig.json` - TypeScript compiler options
- `tailwind.config.ts` - TailwindCSS utility classes
- `ralph.toml` - Ralph configuration (project metadata)

---

## What Does NOT Belong Here

- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md
- Code style rules → CONVENTIONS.md
- Test patterns → TESTING.md

---

*This document shows WHERE code lives. Update when directory structure changes.*
