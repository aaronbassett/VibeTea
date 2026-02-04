# Architecture

> **Purpose**: Document system design, patterns, component relationships, and data flow.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Architecture Overview

VibeTea is a distributed real-time event hub system that monitors Claude Code sessions and broadcasts events to subscribers. The architecture follows a producer-consumer pattern with three main components:

- **Monitor (Producer)**: Rust CLI that watches Claude Code session files and sends events to the server
- **Server (Event Hub)**: Rust HTTP server that ingests events, authenticates requests, and broadcasts to WebSocket clients
- **Client (Consumer)**: React TypeScript web dashboard that subscribes to events and displays real-time session activity

The system is designed for privacy-first monitoring with zero session data persistence.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Event-Driven** | Pub/sub architecture using WebSocket broadcast channels for real-time event distribution |
| **Layered (Server)** | HTTP routes → Auth → Business logic → Broadcast → WebSocket connections |
| **Modular Monorepo** | Cargo workspace with separate `server` and `monitor` binaries sharing types and utilities |
| **Client-Server** | React client communicates with server via HTTP (POST) for events, WebSocket (GET) for subscriptions |

## Core Components

### Monitor (Event Producer)

- **Purpose**: Watches Claude Code session files at `~/.claude/projects/**/*.jsonl`, parses events, applies privacy filtering, signs with Ed25519 keypair, and sends to the server
- **Location**: `monitor/src/`
- **Entry Point**: `monitor/src/main.rs` (CLI-based with `init` and `run` subcommands)
- **Key Modules**:
  - `watcher`: File system watcher using `notify` crate for file change detection
  - `parser`: JSONL parsing from Claude Code session files (privacy-preserving)
  - `privacy`: Event sanitization pipeline to remove sensitive data
  - `crypto`: Ed25519 keypair generation and Ed25519 signature creation
  - `sender`: HTTP client with buffering, retry logic, and rate limiting
  - `config`: Environment variable configuration management
- **Dependencies**: Monitor → Server (HTTP POST with signature)
- **Key Behaviors**:
  - CLI-based: `vibetea-monitor init` generates keypair, `vibetea-monitor run` starts daemon
  - Parses JSONL lines for session start, activity, tool invocation, and completion events
  - Applies privacy filtering before transmission (removes code, prompts, responses)
  - Signs event payloads with Ed25519 private key
  - Buffers events (default 1000) and flushes periodically or on reaching limit
  - Graceful shutdown with 5-second timeout to flush pending events

### Server (Event Hub)

- **Purpose**: Receives events from monitors, validates authentication/signatures, manages rate limiting, and broadcasts events to WebSocket subscribers
- **Location**: `server/src/`
- **Entry Point**: `server/src/main.rs` (HTTP server on port 8080)
- **Key Modules**:
  - `routes`: HTTP route handlers (POST /events, GET /ws, GET /health)
  - `auth`: Ed25519 signature verification and bearer token validation
  - `broadcast`: Event broadcaster using tokio broadcast channels for pub/sub
  - `rate_limit`: Per-source rate limiting to prevent abuse
  - `config`: Environment variable configuration and auth settings
  - `error`: Centralized error types with HTTP status mapping
  - `types`: Shared event data structures (EventType, EventPayload, etc.)
- **Dependencies**: Server → Monitor (receives events), Server → Clients (broadcasts events)
- **Key Behaviors**:
  - Validates incoming events using public keys registered via VIBETEA_PUBLIC_KEYS
  - Implements constant-time comparison for tokens (prevent timing attacks)
  - Rate limiting per source_id with exponential backoff
  - Spawns background cleanup task every 30 seconds for stale rate limit entries
  - Broadcasts events to all connected WebSocket clients
  - Supports optional subscriber filtering by event type, project, session
  - Graceful shutdown with 30-second timeout for in-flight requests
  - Structured JSON logging for production monitoring

### Client (Event Consumer)

- **Purpose**: Real-time web dashboard displaying session activity, event stream, heatmap, and connection status
- **Location**: `client/src/`
- **Entry Point**: `client/src/main.tsx` (React 19 app with Vite)
- **Key Components**:
  - `App.tsx`: Main component coordinating global state and layout
  - `components/`: Presentational components (EventStream, Heatmap, SessionOverview, ConnectionStatus, TokenForm)
  - `hooks/`: Custom React hooks for WebSocket management and event store
  - `utils/`: Formatting utilities for timestamps and data display
  - `types/`: TypeScript type definitions for events
- **Dependencies**: Client → Server (WebSocket subscription)
- **Key Behaviors**:
  - Stores authentication token in localStorage
  - Opens WebSocket connection with bearer token authentication
  - Manages event store with Zustand (global state)
  - Implements session timeout detection (30 seconds of inactivity)
  - Supports filtering by session and time range
  - Real-time UI updates via WebSocket events
  - Graceful disconnection/reconnection with status indicators

## Data Flow

### Primary Event Ingestion Flow

```
Monitor watches file → Parses JSONL → Privacy filters → Signs with Ed25519 →
  Buffers event → HTTP POST /events to Server (with X-Source-ID, X-Signature headers) →
  Server authenticates signature → Rate limit check → Broadcast to subscribers
```

### Primary Client Subscription Flow

```
Client provides bearer token → WebSocket GET /ws to Server →
  Server validates token → Creates subscription → Sends events in real-time →
  Client receives, applies optional filters → React state update → UI renders
```

### Detailed Steps for Event Ingestion

1. **Event Detection**: File watcher detects changes in `~/.claude/projects/**/*.jsonl`
2. **Parsing**: SessionParser extracts structured events from JSONL (session start, activity, tool invocations, completion)
3. **Privacy Filtering**: PrivacyPipeline removes code content, prompts, responses; retains metadata only
4. **Signing**: Monitor signs the JSON-serialized event payload with Ed25519 private key
5. **HTTP Request**: POST /events with body as signed payload, headers:
   - `X-Source-ID`: Monitor identifier (e.g., hostname)
   - `X-Signature`: Base64-encoded Ed25519 signature
   - `Content-Type: application/json`
6. **Server Validation**:
   - Extracts source_id from header
   - Looks up public key in VIBETEA_PUBLIC_KEYS map
   - Verifies signature using `ed25519_dalek::VerifyingKey::verify_strict()`
   - Constant-time comparison to prevent timing attacks
7. **Rate Limiting**: Per-source rate limiter checks request count, returns 429 if exceeded
8. **Broadcasting**: Valid event is distributed to all connected WebSocket subscribers
9. **Persistence**: Events are NOT persisted to disk (ephemeral, real-time only)

### Detailed Steps for Client Subscription

1. **Token Management**: Client checks localStorage for saved token
2. **Token Entry**: If missing, TokenForm prompts for token entry
3. **WebSocket Connection**: useWebSocket hook opens WS connection to GET /ws with token in query param
4. **Server Authentication**: Server validates token against VIBETEA_SUBSCRIBER_TOKEN
5. **Subscription Active**: Server queues all broadcast events for this client
6. **Event Reception**: Client receives events as JSON, parses, and stores in Zustand
7. **Filtering**: useEventStore applies optional filters (session_id, time range)
8. **UI Update**: React components subscribe to store changes and render updates
9. **Session Timeouts**: useSessionTimeouts tracks inactivity and marks sessions as timed out after 30s

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|------------|---------------|
| **HTTP Routes** | HTTP parsing, header extraction | Auth, Broadcast, Rate Limiter | Database (none exists) |
| **Authentication** | Signature/token validation | Config (public keys) | Internal state, routes directly |
| **Rate Limiting** | Per-source request counting | Config | Routes, Auth (no feedback to them) |
| **Broadcast** | Event distribution to subscribers | Types, Events | Auth, Routes directly |
| **Config** | Environment variable parsing | Nothing (immutable) | Routes, Auth (must be passed) |
| **Monitor File Watcher** | File system event detection | Parser, Crypto | Sender (event queuing only) |
| **Monitor Privacy Pipeline** | Event sanitization | Type definitions | Sender, Watcher, Crypto |
| **Monitor Sender** | HTTP transmission with retry | Crypto (for signing), Config | Watcher, Parser |

## Dependency Rules

- **Server Routes** depend on Auth, Broadcast, RateLimiter, Config (one-way)
- **Auth** depends only on Config (public key lookup)
- **Broadcast** is independent, used by Routes
- **RateLimiter** is independent, used by Routes
- **Monitor** components form a pipeline: Watcher → Parser → PrivacyPipeline → Event → Sender
- **Client** has no dependencies on server internals; communicates only via HTTP/WebSocket APIs
- **No circular dependencies**: All dependencies flow downward (routes depend on lower layers)

## Key Interfaces & Contracts

| Interface | Purpose | Implementations |
|-----------|---------|-----------------|
| `AppState` | Shared server state holder | Created once at startup, cloned for each request |
| `EventBroadcaster` | Multi-producer broadcast channel | Tokio broadcast channel with up to 1000 event capacity |
| `RateLimiter` | Per-source rate limit tracking | In-memory HashMap with cleanup task |
| `FileWatcher` | File system change notification | Wrapper around `notify` crate with mpsc channel |
| `SessionParser` | JSONL event parsing | Stateful per-session, parses Claude Code JSONL format |
| `PrivacyPipeline` | Event payload sanitization | Configurable via VIBETEA_BASENAME_ALLOWLIST |
| `Crypto` | Ed25519 operations | Keypair generation, signing, verification |
| `Sender` | HTTP transmission with retries | Buffered, with exponential backoff |

## State Management

| State Type | Location | Pattern | Scope |
|------------|----------|---------|-------|
| **Server Config** | AppState (Arc) | Read-only, loaded once | Application lifetime |
| **Rate Limits** | AppState.rate_limiter (Arc<Mutex>) | Per-source counters with TTL | Runtime, cleaned every 30s |
| **Broadcast Events** | AppState.broadcaster (Arc) | Tokio broadcast channel | All connected WebSocket clients |
| **Client Auth Token** | localStorage | String persistence | Session/browser lifetime |
| **Client Events** | Zustand store | Immutable event log with filters | React component lifetime |
| **Client Connections** | useWebSocket hook | WebSocket instance | Component lifetime |
| **Monitor Session Parsers** | HashMap in main loop | Per-file state | Until file is removed |
| **Monitor Event Buffer** | Sender buffer (Vec) | Queue with configurable size | Until flushed |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Logging** | Structured JSON logging with tracing crate | `server/src/main.rs`, `monitor/src/main.rs`, all modules |
| **Error Handling** | Custom error types with thiserror, HTTP status mapping | `server/src/error.rs`, `monitor/src/error.rs` |
| **Authentication** | Ed25519 signatures + bearer tokens | `server/src/auth.rs`, `monitor/src/crypto.rs` |
| **Privacy** | Event payload filtering before transmission | `monitor/src/privacy.rs` |
| **Rate Limiting** | Per-source request counting with cleanup | `server/src/rate_limit.rs` |
| **Graceful Shutdown** | Signal handlers (SIGTERM/SIGINT) with timeouts | `server/src/main.rs`, `monitor/src/main.rs` |
| **Configuration** | Environment variables with defaults | `server/src/config.rs`, `monitor/src/config.rs` |

---

## What Does NOT Belong Here

- Directory structure details → STRUCTURE.md
- Technology versions → STACK.md
- External service configs → INTEGRATIONS.md
- Code style rules → CONVENTIONS.md

---

*This document describes HOW the system is organized. Keep focus on patterns and relationships.*
