# Architecture

> **Purpose**: Document system design, patterns, component relationships, and data flow.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Architecture Overview

VibeTea is a distributed event aggregation and broadcast system for AI coding assistants with three independent components:

- **Monitor** (Rust CLI) - Watches local Claude Code session files and forwards privacy-filtered events to the server
- **Server** (Rust HTTP API) - Central hub that receives events from monitors and broadcasts them to subscribers via WebSocket
- **Client** (React SPA) - Real-time dashboard displaying aggregated event streams, session activity, and usage heatmaps

The system follows a **hub-and-spoke architecture** where the Server acts as a central event bus, decoupling multiple Monitor sources from multiple Client consumers. Events flow unidirectionally: Monitors → Server → Clients, with no persistent storage.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Central Server acts as event aggregation point, Monitors feed events, Clients consume |
| **Producer-Consumer** | Monitors are event producers, Clients are event consumers, Server mediates asynchronous delivery |
| **Privacy-First** | Events contain only structural metadata (timestamps, tool names, file basenames), never code or sensitive content |
| **Real-time Streaming** | WebSocket-based live event delivery with no message persistence (fire-and-forget) |

## Core Components

### Monitor (Source)

- **Purpose**: Watches Claude Code session files in `~/.claude/projects/**/*.jsonl` and emits structured events
- **Location**: `monitor/src/`
- **Key Responsibilities**:
  - File watching via `inotify` (Linux), `FSEvents` (macOS), `ReadDirectoryChangesW` (Windows)
  - Privacy-preserving JSONL parsing (extracts metadata only)
  - Cryptographic signing of events (Ed25519)
  - Event buffering and exponential backoff retry
  - Graceful shutdown with event flushing
- **Dependencies**:
  - Monitors depend on **Server** (via HTTP POST to `/events`)
  - Monitors depend on local Claude Code installation (`~/.claude/`)
- **Dependents**: None (source component)

### Server (Hub)

- **Purpose**: Central event aggregation point that validates, authenticates, and broadcasts events to all subscribers
- **Location**: `server/src/`
- **Key Responsibilities**:
  - Receiving and authenticating events from Monitors (Ed25519 signature verification)
  - Rate limiting (per-source basis, configurable)
  - Broadcasting events to WebSocket subscribers
  - Token-based authentication for WebSocket clients
  - Graceful shutdown with timeout for in-flight requests
- **Dependencies**:
  - Broadcasts to **Clients** (via WebSocket `/ws` endpoint)
  - Depends on Monitor-provided public keys for signature verification
- **Dependents**: Clients (consumers)

### Client (Consumer)

- **Purpose**: Real-time dashboard displaying aggregated event stream from the Server
- **Location**: `client/src/`
- **Key Responsibilities**:
  - WebSocket connection management with exponential backoff reconnection
  - Event buffering (1000 events max) with FIFO eviction
  - Session state management (Active, Inactive, Ended, Removed)
  - Event filtering (by session ID, time range)
  - Real-time visualization (event list, session overview, heatmap)
- **Dependencies**:
  - Depends on **Server** (via WebSocket connection to `/ws`)
  - No persistence layer (in-memory Zustand store)
- **Dependents**: None (consumer component)

## Data Flow

### Primary Monitor-to-Client Flow

Claude Code → Monitor → Server → Client:
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
11. React renders updated event list, session overview, heatmap

### Detailed Request/Response Cycle

1. **Event Creation** (Monitor/Parser):
   - JSONL line parsed from `~/.claude/projects/<uuid>.jsonl`
   - `SessionParser` extracts timestamp, tool name, action
   - `PrivacyPipeline` removes sensitive fields (code, prompts)
   - `Event` struct created with unique ID (`evt_` prefix + 20-char suffix)

2. **Event Signing** (Monitor/Sender):
   - Event payload serialized to JSON
   - Ed25519 signature computed over message body
   - Event queued in local buffer (max 1000, FIFO eviction)

3. **Event Transmission** (Monitor/Sender):
   - `POST /events` with headers: `X-Source-ID`, `X-Signature`
   - On 429 (rate limit): parse `Retry-After` header
   - On network failure: exponential backoff (1s → 60s, ±25% jitter)
   - On success: continue flushing buffered events

4. **Event Ingestion** (Server):
   - Extract `X-Source-ID` and `X-Signature` headers
   - Load Monitor's public key from config (`VIBETEA_PUBLIC_KEYS`)
   - Verify Ed25519 signature using `subtle::ConstantTimeEq` (timing-safe)
   - Rate limit check per source_id
   - Broadcast event to all WebSocket subscribers via `tokio::broadcast`

5. **Event Delivery** (Server → Client):
   - WebSocket subscriber receives `BroadcastEvent`
   - Client's `useWebSocket` hook calls `addEvent()` action
   - Zustand store updates event buffer (evicts oldest if > 1000)
   - Session state updated (Active/Inactive/Ended/Removed)
   - React re-renders only affected components (via Zustand selectors)

6. **Visualization** (Client):
   - `EventStream` component renders with virtualized scrolling
   - `SessionOverview` shows active sessions with metadata
   - `Heatmap` displays activity over time bins
   - `ConnectionStatus` shows server connectivity

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|------------|---------------|
| **Monitor** | Observe local activity, preserve privacy, authenticate | FileSystem, HTTP client, Crypto | Server internals, other Monitors, network stack details |
| **Server** | Route, authenticate, broadcast, rate limit | All Monitors' public keys, event config, rate limiter state | File system, external APIs, Client implementation details |
| **Client** | Display, interact, filter, manage WebSocket | Server WebSocket, local storage (token), state store | Server internals, other Clients' state, file system |

## Dependency Rules

- **Monitor → Server**: Only on startup (load server URL from config), then periodically via HTTP POST
- **Server → Monitor**: None (server doesn't initiate contact)
- **Server ↔ Client**: Bidirectional (Client initiates WebSocket, Server sends events, Client sends nothing back)
- **Cross-Monitor**: No inter-Monitor communication (all go through Server)
- **Persistence**: None at any layer (no database, no cache persistence)

## Key Interfaces & Contracts

| Interface | Purpose | Implementations |
|-----------|---------|-----------------|
| `Event` | Core event struct with type + payload | JSON serialization via `serde` |
| `EventPayload` | Tagged union of event variants | Session, Activity, Tool, Agent, Summary, Error |
| `EventType` | Enum discriminator | 6 variants (Session, Activity, Tool, Agent, Summary, Error) |
| `AuthError` | Auth failure codes | InvalidSignature, UnknownSource, InvalidToken |
| `RateLimitResult` | Rate limit outcome | Allowed, Blocked (with retry delay) |
| `SubscriberFilter` | Optional event filtering | by_event_type, by_project, by_source |
| `KeySource` | Tracks key origin for logging | EnvironmentVariable, File(PathBuf) |
| `Crypto` | Ed25519 key operations | generate(), load(), save(), sign(), public_key_fingerprint() |

## Authentication & Authorization

### Monitor Authentication (Source)

- **Mechanism**: Ed25519 signature verification
- **Flow**:
  1. Monitor generates keypair: `vibetea-monitor init`
  2. Public key registered with Server: `VIBETEA_PUBLIC_KEYS=monitor1:base64pubkey`
  3. On `POST /events`, Monitor signs message body with private key
  4. Server verifies signature against pre-registered public key
  5. Invalid signatures rejected with 401 Unauthorized
- **Security**: Uses `ed25519_dalek::VerifyingKey::verify_strict()` (RFC 8032 compliant)
- **Timing Attack Prevention**: `subtle::ConstantTimeEq` for signature comparison
- **Key Tracking**: `KeySource` enum tracks whether keys loaded from file or environment variable (logged at startup for verification)

### Client Authentication (Consumer)

- **Mechanism**: Bearer token (HTTP header)
- **Flow**:
  1. Client obtains token (out-of-band, server-configured)
  2. Token sent in WebSocket upgrade request: `?token=secret`
  3. Server calls `validate_token()` to check token
  4. Invalid/missing tokens rejected with 401 Unauthorized
- **Storage**: Client stores token in localStorage under `vibetea_token` key

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

### Monitor State
| State Type | Location | Pattern |
|------------|----------|---------|
| **Event Buffer** | `VecDeque<Event>` (max configurable) | FIFO eviction, flushed on graceful shutdown |
| **Session Parsers** | `HashMap<PathBuf, SessionParser>` | Keyed by file path, created on first write, removed on file delete |
| **Retry State** | `Sender` internal | Tracks backoff attempt count per send operation |
| **Crypto Keys** | Loaded once at startup | KeySource tracked for logging, indicates origin (env var or file) |

## Cryptographic Key Management

### Key Loading Strategy

The Monitor implements a flexible key loading mechanism with precedence rules:

1. **Environment Variable (`VIBETEA_PRIVATE_KEY`)**
   - Takes absolute precedence when set
   - Must contain base64-encoded 32-byte Ed25519 seed (RFC 4648 standard base64)
   - Whitespace trimmed before decoding (handles newlines, CRLF, spaces)
   - Used via `Crypto::load_from_env()` for direct loading
   - Used via `Crypto::load_with_fallback()` as primary source (no fallback on error)

2. **File-based Key (`~/.vibetea/key.priv`)**
   - Used as fallback when env var not set
   - Contains raw 32-byte seed (binary format)
   - Loaded via `Crypto::load()` or `Crypto::load_with_fallback()`
   - File permissions enforced: `0600` (owner read/write only)

3. **Key Precedence Rules**
   - If `VIBETEA_PRIVATE_KEY` is set: use it (even if file exists and file would error)
   - If `VIBETEA_PRIVATE_KEY` is set but invalid: error immediately (no fallback to file)
   - If `VIBETEA_PRIVATE_KEY` not set: use file at `VIBETEA_KEY_PATH/key.priv`

4. **Runtime Behavior in `run_monitor()`**
   - Calls `Crypto::load_with_fallback(&config.key_path)`
   - Returns tuple `(Crypto, KeySource)` indicating origin
   - Logs key fingerprint and source at INFO level
   - Logs warning if file key exists but env var takes precedence

### Memory Safety & Zeroization

The crypto module implements memory-safe key handling:

- **Intermediate Buffers**: All decoded/processed key material is zeroed after use via `zeroize` crate
- **Seed Array Zeroization**: `seed: [u8; SEED_LENGTH]` zeroed immediately after creating `SigningKey`
- **Error Path Safety**: Decoded buffers zeroed even on error paths (e.g., invalid length)
- **Key Derivation**: Signing key created from seed, then seed immediately zeroed
- **Comment Tags**: All zeroization points marked with FR-020 (security feature reference)

### Key Exposure Logging

The Monitor logs key origin at startup to help users verify key loading:

```rust
// When loaded from environment variable
info!(
    source = "environment",
    fingerprint = %crypto.public_key_fingerprint(),
    "Cryptographic key loaded"
);

// When loaded from file
info!(
    source = "file",
    path = %path.display(),
    fingerprint = %crypto.public_key_fingerprint(),
    "Cryptographic key loaded"
);

// Warning if both sources exist
info!(
    ignored_path = %config.key_path.display(),
    "File key exists but VIBETEA_PRIVATE_KEY takes precedence"
);
```

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Logging** | Structured JSON (tracing crate) | Server: `main.rs`, Monitor: `main.rs`, Client: error boundaries in components |
| **Error Handling** | Custom error enums + thiserror | `server/src/error.rs`, `monitor/src/error.rs` |
| **Rate Limiting** | Per-source counter with TTL | `server/src/rate_limit.rs` |
| **Privacy** | Event payload sanitization | `monitor/src/privacy.rs` (removes sensitive fields) |
| **Graceful Shutdown** | Signal handlers + timeout | `server/src/main.rs`, `monitor/src/main.rs` |
| **Retry Logic** | Exponential backoff with jitter | `monitor/src/sender.rs`, `client/src/hooks/useWebSocket.ts` |
| **Key Management** | Ed25519 key generation, storage, signing | `monitor/src/crypto.rs` (with KeySource tracking and memory zeroing) |

## Design Decisions

### Why Hub-and-Spoke?

- **Decouples sources from sinks**: Multiple Monitor instances can run independently
- **Centralized authentication**: Server is the only point needing cryptographic keys
- **Easy horizontal scaling**: Monitors and Clients scale independently
- **No inter-Monitor coupling**: Monitors don't need to know about each other

### Why No Persistence?

- **Simplifies deployment**: No database to manage
- **Supports distributed monitoring**: Each Client sees latest events only
- **Privacy-first**: Events never written to disk (except logs)
- **Real-time focus**: System optimized for live streams, not historical analysis

### Why Ed25519?

- **Widely supported**: NIST-standardized modern elliptic curve
- **Signature verification only**: Public key crypto prevents Monitors impersonating each other
- **Timing-safe implementation**: `subtle::ConstantTimeEq` prevents timing attacks
- **Small key size**: ~32 bytes per key, easy to share via env vars

### Why WebSocket?

- **Bi-directional low-latency**: Better than HTTP polling for real-time updates
- **Connection persistence**: Single connection replaces request/response overhead
- **Native browser support**: No additional libraries needed for basic connectivity
- **Standard protocol**: Works with existing proxies and load balancers

### Why Environment Variable Precedence?

- **Container-friendly**: Secrets in env vars are standard practice for containerized apps
- **No file permissions required**: Works in restricted environments (CI, serverless)
- **Emergency override**: Can temporarily use different key without file system changes
- **Key rotation support**: Switch keys by changing env var without file I/O
- **Explicit precedence rules**: Clear error handling (env var errors don't silently fallback)

---

*This document describes HOW the system is organized. Consult STRUCTURE.md for WHERE code lives.*
