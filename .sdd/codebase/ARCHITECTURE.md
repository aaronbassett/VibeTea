# Architecture

**Status**: Phase 1-2 implementation - Configuration, error handling, and type system in place
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Architecture Overview

VibeTea is a three-tier real-time event streaming system with clear separation of concerns:

- **Monitor** (Rust): Event producer that captures Claude Code session activity
- **Server** (Rust): Event hub that authenticates monitors and broadcasts to clients
- **Client** (TypeScript/React): Event consumer that displays sessions and activities

The system follows a hub-and-spoke pattern where monitors are trusted publishers and clients are passive subscribers. All communication is event-driven with no persistent state required on the server.

## Architecture Pattern

| Pattern | Description |
|---------|-------------|
| **Hub-and-Spoke** | Monitors push events to the server, clients subscribe via WebSocket |
| **Event-Driven** | All state changes flow through immutable, versioned events |
| **Layered** | Monitor: Config → Types; Server: Config → Types → Error; Client: Types → Hooks → Components |
| **Pub/Sub** | Server acts as event broker with asymmetric authentication (monitors sign, clients consume) |

## Core Components

### Monitor Component

**Purpose**: Captures Claude Code session activity and transmits to server
**Location**: `monitor/src/`
**Technologies**: Rust, tokio, file watching, Ed25519 cryptography

**Key Modules**:
- `config.rs` - Environment variable parsing for monitor configuration (server URL, keys, buffer size)
- `types.rs` - Event definitions shared with server (EventType, EventPayload, Event)
- `error.rs` - Error hierarchy covering configuration, I/O, JSON, HTTP, cryptographic, and file watch errors
- `main.rs` - Application entry point (placeholder for Phase 3)

**Dependencies**:
- `types` ← defined in `monitor/src/types.rs`
- `config` ← defined in `monitor/src/config.rs`
- `error` ← defined in `monitor/src/error.rs`

### Server Component

**Purpose**: Receives authenticated events from monitors, broadcasts to subscribed clients
**Location**: `server/src/`
**Technologies**: Rust, tokio, axum, WebSocket, cryptographic verification

**Key Modules**:
- `config.rs` - Environment variable parsing (public keys, subscriber tokens, port)
- `error.rs` - Error hierarchy for config, auth, validation, rate limiting, WebSocket, and internal errors
- `types.rs` - Event definitions with untagged serde deserialization (EventType, EventPayload, Event)
- `lib.rs` - Module declarations for public API
- `main.rs` - Application entry point (placeholder for Phase 3)

**Dependencies**:
- `types` ← defined in `server/src/types.rs`
- `config` ← defined in `server/src/config.rs`
- `error` ← defined in `server/src/error.rs`

### Client Component

**Purpose**: Subscribes to server events, displays sessions and activities
**Location**: `client/src/`
**Technologies**: TypeScript, React, Zustand, Vite

**Key Modules**:
- `types/events.ts` - TypeScript definitions matching Rust types with type guards
- `hooks/useEventStore.ts` - Zustand store for event state management and session aggregation
- `App.tsx` - Root component (placeholder for Phase 3)
- `main.tsx` - React entry point

**Dependencies**:
- `types/events` ← defined in `client/src/types/events.ts`
- `hooks/useEventStore` ← defined in `client/src/hooks/useEventStore.ts`

## Data Flow

### Monitor → Server Flow

```
Claude Code Session Activity
         ↓
    Monitor (Rust)
         ↓
    Capture & Queue Events
         ↓
    Sign with Ed25519 Private Key
         ↓
    HTTPS POST to Server
         ↓
    Server (Rust)
         ↓
    Verify Signature with Public Key
         ↓
    Validate Event Schema
         ↓
    Broadcast via WebSocket
```

**Flow Steps**:
1. Monitor watches Claude Code directory for session activity
2. Events are generated with unique ID (evt_ + 20 chars), source ID, and timestamp
3. Events are buffered and signed with the monitor's private key
4. Signed batch is sent to server's `/events` endpoint via HTTPS POST
5. Server verifies signature using configured public key for the source
6. Server validates event schema against types
7. Server broadcasts to all connected WebSocket clients

### Server → Client Flow

```
Authenticated Event
        ↓
   Server (Rust)
        ↓
   WebSocket Broadcast
        ↓
   Client (TypeScript)
        ↓
   Zustand Store Update
        ↓
   Session Aggregation
        ↓
   Component Re-render
        ↓
   Display in UI
```

**Flow Steps**:
1. Server receives validated events from monitor
2. Event is immediately broadcast to all connected WebSocket clients
3. Client receives event via WebSocket listener
4. Event is added to Zustand store via `addEvent` action
5. Store performs session aggregation (create/update session state)
6. Components subscribed to store state re-render
7. UI displays updated sessions and activities

## Layer Boundaries

| Layer | Responsibility | Can Access | Cannot Access |
|-------|----------------|----------|---------------|
| **Monitor HTTP Layer** | Serialize events, sign payload, send HTTPS | Config, Types, Error | Server internals |
| **Monitor Config Layer** | Parse env vars, validate paths | Filesystem, environment | Types directly |
| **Server HTTP/WS Layer** | Accept events, verify signatures, broadcast | Config, Types, Error | Stores, databases |
| **Server Config Layer** | Parse env vars, validate auth credentials | Filesystem, environment | HTTP layer |
| **Client HTTP/WS Layer** | Connect to server, receive events | Types, Zustand store | Server internals |
| **Client Store Layer** | Manage event buffer, aggregate sessions | Types, event stream | Server directly |
| **Client Component Layer** | Display UI, handle user actions | Store hooks, types | WebSocket layer directly |

## Dependency Rules

- **No circular dependencies**: Monitor → Types → (nothing), Server → Types → (nothing), Client → Store → Types → (nothing)
- **No persistent storage**: All layers are ephemeral; events exist only in memory
- **Asymmetric auth**: Monitors authenticate with cryptographic signatures; clients authenticate with bearer tokens
- **Type safety**: All three languages use strong typing; event schema is enforced at compile time
- **Layered imports**: Higher layers (HTTP) depend on lower layers (Config, Types, Error), not vice versa

## Key Interfaces & Contracts

### Monitor ↔ Server Contract

**Endpoint**: `POST /events`
**Auth**: Ed25519 signature in header
**Body**: JSON array of events

```json
[
  {
    "id": "evt_k7m2n9p4q1r6s3t8u5v0",
    "source": "macbook-pro",
    "timestamp": "2026-02-02T14:30:00Z",
    "type": "tool",
    "payload": {
      "sessionId": "550e8400-e29b-41d4-a716-446655440000",
      "tool": "Read",
      "status": "completed",
      "context": "main.rs",
      "project": "vibetea"
    }
  }
]
```

**Server Config**:
```
VIBETEA_PUBLIC_KEYS=source1:base64key1,source2:base64key2
VIBETEA_SUBSCRIBER_TOKEN=secret-token
PORT=8080
```

### Server ↔ Client Contract

**Endpoint**: `WS /subscribe`
**Auth**: Bearer token in query param or header
**Message Format**: JSON-serialized VibeteaEvent

```json
{
  "id": "evt_k7m2n9p4q1r6s3t8u5v0",
  "source": "macbook-pro",
  "timestamp": "2026-02-02T14:30:00Z",
  "type": "tool",
  "payload": {
    "sessionId": "550e8400-e29b-41d4-a716-446655440000",
    "tool": "Read",
    "status": "completed",
    "context": "main.rs",
    "project": "vibetea"
  }
}
```

## State Management

| State Type | Location | Pattern | Scope |
|-----------|----------|---------|-------|
| **Event Buffer** | Client Zustand store | FIFO with max 1000 events | Client-side only |
| **Session State** | Client Zustand store | Derived from events (created/updated on new events) | Client-side only |
| **Connection Status** | Client Zustand store | ConnectionStatus union (connecting/connected/disconnected/reconnecting) | Client-side only |
| **Config State** | Monitor/Server | Immutable, parsed at startup | Process lifetime |
| **Auth Credentials** | Server memory | Public keys map, subscriber token | Process lifetime |

## Cross-Cutting Concerns

| Concern | Implementation | Location |
|---------|----------------|----------|
| **Error Handling** | Type-specific error enums with `thiserror` (Rust) | `server/src/error.rs`, `monitor/src/error.rs` |
| **Configuration** | Environment variable parsing with validation | `server/src/config.rs`, `monitor/src/config.rs` |
| **Cryptographic Signing** | Ed25519 for monitor → server authentication | Monitor to be implemented in Phase 3 |
| **Event Schema** | Shared types defined in both Rust and TypeScript | `server/src/types.rs`, `monitor/src/types.rs`, `client/src/types/events.ts` |
| **Logging** | Rust: tracing crate; TypeScript: console (to be enhanced) | Integrated in config parsing |
| **Session Aggregation** | Client-side state machine based on event type | `client/src/hooks/useEventStore.ts` |

## Event Schema

All events follow the same structure:

```rust
pub struct Event {
    pub id: String,              // evt_ + 20 alphanumeric
    pub source: String,          // Monitor identifier
    pub timestamp: DateTime<Utc>, // RFC 3339 UTC
    pub event_type: EventType,   // session|activity|tool|agent|summary|error
    pub payload: EventPayload,   // Type-specific data
}
```

**EventPayload Variants**:
- **Session**: `session_id`, `action` (started/ended), `project`
- **Activity**: `session_id`, `project` (optional)
- **Tool**: `session_id`, `tool` (name), `status` (started/completed), `context` (optional), `project` (optional)
- **Agent**: `session_id`, `state` (string)
- **Summary**: `session_id`, `summary` (string)
- **Error**: `session_id`, `category` (string)

---

*This document describes HOW the system is organized. Keep focus on patterns and relationships.*
