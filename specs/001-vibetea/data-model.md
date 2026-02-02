# Data Model: VibeTea

**Feature**: 001-vibetea | **Date**: 2026-02-02

This document defines the entities, relationships, and state transitions for VibeTea.

---

## Entities

### Event

The core data unit flowing through the system. Events are immutable once created.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique identifier: `evt_` + 20 alphanumeric chars |
| `source` | string | Yes | Monitor identifier (hostname or custom ID) |
| `timestamp` | string | Yes | RFC 3339 UTC (e.g., `2026-02-02T14:30:00Z`) |
| `type` | enum | Yes | One of: `session`, `activity`, `tool`, `agent`, `summary`, `error` |
| `payload` | object | Yes | Type-specific payload (see below) |

**Event ID Generation** (Monitor):
```
evt_ + 20 random alphanumeric characters
Example: evt_a1b2c3d4e5f6g7h8i9j0
```

### EventPayload

Payload structure varies by event type:

**session**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sessionId` | string | Yes | UUID identifying the session |
| `action` | enum | Yes | `started` or `ended` |
| `project` | string | Yes | Project name (directory basename) |

**activity**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sessionId` | string | Yes | UUID identifying the session |
| `project` | string | No | Project name |

**tool**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sessionId` | string | Yes | UUID identifying the session |
| `tool` | string | Yes | Tool name (e.g., `Read`, `Write`, `Bash`) |
| `status` | enum | Yes | `started` or `completed` |
| `context` | string | No | File basename (if applicable) |
| `project` | string | No | Project name |

**agent**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sessionId` | string | Yes | UUID identifying the session |
| `state` | string | Yes | Agent state description |

**summary**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sessionId` | string | Yes | UUID identifying the session |
| `summary` | string | Yes | Session summary text |

**error**:
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sessionId` | string | Yes | UUID identifying the session |
| `category` | string | Yes | Error category (not error message content) |

---

### Session (Client-side derived)

Sessions are derived from events in the Client. Not stored server-side.

| Field | Type | Description |
|-------|------|-------------|
| `sessionId` | string | UUID from events |
| `source` | string | Monitor that emitted events |
| `project` | string | Project name |
| `startedAt` | Date | Timestamp of first event |
| `lastEventAt` | Date | Timestamp of most recent event |
| `status` | enum | `active`, `inactive`, `ended` |
| `eventCount` | number | Total events in this session |

**Session State Machine**:

```
┌─────────────────────────────────────────────────────────┐
│                                                          │
│  ┌─────────┐    first event    ┌────────┐               │
│  │  (new)  │ ─────────────────▶│ active │               │
│  └─────────┘                   └────┬───┘               │
│                                     │                    │
│                    ┌────────────────┼────────────────┐  │
│                    │                │                │  │
│                    ▼                ▼                │  │
│            no events for 5m    summary event        │  │
│                    │                │                │  │
│                    ▼                ▼                │  │
│              ┌──────────┐     ┌─────────┐           │  │
│              │ inactive │     │  ended  │           │  │
│              └────┬─────┘     └────┬────┘           │  │
│                   │                │                │  │
│         event     │                │                │  │
│         received  │                │                │  │
│                   ▼                │                │  │
│              ┌────────┐            │                │  │
│              │ active │◀───────────┘                │  │
│              └────────┘     (event received)        │  │
│                   │                                  │  │
│                   │         no events for 30m       │  │
│                   └─────────────────────────────────┘  │
│                                     │                   │
│                                     ▼                   │
│                              ┌───────────┐              │
│                              │  removed  │              │
│                              │ from view │              │
│                              └───────────┘              │
└─────────────────────────────────────────────────────────┘
```

**Transitions**:
- `(new) → active`: First event received for this sessionId
- `active → inactive`: No events for 5 minutes
- `inactive → active`: Event received
- `active/inactive → ended`: Summary event received
- `inactive/ended → removed`: No events for 30 minutes

---

### Source (Server-side)

Registered Monitors that can send events.

| Field | Type | Description |
|-------|------|-------------|
| `sourceId` | string | Unique identifier (hostname or custom) |
| `publicKey` | string | Base64-encoded Ed25519 public key |

**Registration**: Sources are pre-registered via `VIBETEA_PUBLIC_KEYS` environment variable:
```
VIBETEA_PUBLIC_KEYS=laptop:abc123...,desktop:def456...
```

---

### Subscriber (Server-side, ephemeral)

WebSocket clients connected to receive events.

| Field | Type | Description |
|-------|------|-------------|
| `connectionId` | internal | Server-assigned connection identifier |
| `filters` | object | Optional filters: `source`, `type`, `project` |
| `connectedAt` | timestamp | Connection time |

**Authentication**: Token provided via `?token=` query parameter, validated against `VIBETEA_SUBSCRIBER_TOKEN`.

---

## Relationships

```
┌──────────────┐
│    Source    │
│  (Monitor)   │
└──────┬───────┘
       │ emits
       ▼
┌──────────────┐         ┌──────────────┐
│    Event     │────────▶│   Session    │
│              │ belongs │  (derived)   │
└──────┬───────┘   to    └──────────────┘
       │
       │ broadcast to
       ▼
┌──────────────┐
│  Subscriber  │
│   (Client)   │
└──────────────┘
```

- **Source → Event**: One-to-many. A Source emits many Events.
- **Event → Session**: Many-to-one. Events belong to a Session (via `sessionId`).
- **Event → Subscriber**: Many-to-many (broadcast). Events are broadcast to all matching Subscribers.

---

## Validation Rules

### Event Validation (Server)

| Field | Rule |
|-------|------|
| `id` | Must match `^evt_[a-zA-Z0-9]{20}$` |
| `source` | Must match registered Source in `VIBETEA_PUBLIC_KEYS` |
| `timestamp` | Must be valid RFC 3339, within 5 minutes of server time |
| `type` | Must be one of allowed enum values |
| `payload.sessionId` | Must be valid UUID format |

### Privacy Validation (Monitor)

Before transmission, Monitor MUST verify:
- No `content` fields present
- No full file paths (only basenames)
- No `command` fields from Bash tool use
- No `pattern` fields from search tools

---

## Data Retention

| Component | Data | Retention |
|-----------|------|-----------|
| Monitor | Event buffer | 1000 events max (FIFO eviction) |
| Server | Events | None (transient broadcast only) |
| Server | Connections | Until disconnect |
| Client | Event buffer | 1000 events max (in-memory) |
| Client | Sessions | Until 30 min inactive |
| Client | Statistics | Aggregates since page load |
| Client | Auth token | localStorage (until logout/401) |

---

## Example Event JSON

**tool event**:
```json
{
  "id": "evt_k7m2n9p4q1r6s3t8u5v0",
  "source": "macbook-pro",
  "timestamp": "2026-02-02T14:30:00Z",
  "type": "tool",
  "payload": {
    "sessionId": "550e8400-e29b-41d4-a716-446655440000",
    "project": "vibetea",
    "tool": "Read",
    "status": "completed",
    "context": "main.rs"
  }
}
```

**session started event**:
```json
{
  "id": "evt_a1b2c3d4e5f6g7h8i9j0",
  "source": "macbook-pro",
  "timestamp": "2026-02-02T14:00:00Z",
  "type": "session",
  "payload": {
    "sessionId": "550e8400-e29b-41d4-a716-446655440000",
    "action": "started",
    "project": "vibetea"
  }
}
```

---

*Data model complete. See contracts/ for API specification.*
