# Feature Specification: VibeTea

**Feature Branch**: `001-vibetea`
**Created**: 2026-02-02
**Status**: Complete (Ready for Planning)
**Input**: Real-time AI coding assistant activity monitoring system

---

## Problem Statement

Modern developers increasingly rely on AI coding assistants, often running multiple agents (Claude Code, Cursor, Copilot) across different projects simultaneously. Currently, there is no unified way to:

- Monitor AI agent activity across multiple tools and sessions
- Visualize coding patterns and AI utilization over time
- Build integrations that react to AI assistant events in real-time
- Understand how AI assistants are being used across a team or organization

**VibeTea** solves this by providing a real-time event aggregation and broadcast system that consolidates activity streams from multiple AI agents into a unified WebSocket feed. Privacy is paramount—VibeTea broadcasts only structural metadata (event types, tool categories, timestamps) and never transmits code, prompts, file contents, or any sensitive information.

**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

---

## Personas

| Persona               | Description                                                                           | Primary Goals                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| **Solo Developer**    | Individual developer using AI coding assistants (Claude Code, Cursor, Copilot) daily  | Wants visibility into their AI-assisted workflow, see patterns, and understand utilization    |
| **Team Lead**         | Technical lead or engineering manager overseeing developers using AI assistants        | Wants to understand how AI assistants are being used across the team and ensure effectiveness |
| **Integration Builder** | Developer building custom tooling, dashboards, or automations                       | Wants a reliable, well-documented event stream API to build custom integrations               |

---

## System Architecture

VibeTea follows a hub-and-spoke architecture with three components:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Claude    │    │   Cursor    │    │   Copilot   │
│   Monitor   │    │   Monitor   │    │   Monitor   │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          ▼
                 ┌─────────────────┐
                 │    VibeTea      │
                 │     Server      │
                 └────────┬────────┘
                          │
       ┌──────────────────┼──────────────────┐
       ▼                  ▼                  ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Dashboard  │    │   Custom    │    │    CLI      │
│   Client    │    │ Integration │    │   Client    │
└─────────────┘    └─────────────┘    └─────────────┘
```

**Note**: v1 scope includes Claude Code Monitor only. Cursor and Copilot monitors are planned for future versions.

---

## User Scenarios & Testing

### User Story 1 - Server Event Hub Core (Priority: P1)

**As a** VibeTea operator,
**I want** a central server that receives events from Monitors and broadcasts them to Clients,
**so that** I can run a reliable event hub that connects multiple sources to multiple consumers.

**Why this priority**: The Server is the central nervous system of VibeTea. Without it, no events can flow between Monitors and Clients. All other components depend on this being operational first.

**Independent Test**: Can be tested by sending mock events via curl to POST /events and verifying WebSocket broadcasts to connected clients.

**Acceptance Scenarios**:

1. **Given** a Monitor sends a POST request to `/events` with valid signature headers, **When** the request contains a valid event JSON and the source's public key is registered, **Then** the server accepts with 202 Accepted and broadcasts the event to all connected WebSocket clients

2. **Given** a Monitor sends an array of events to `/events`, **When** all events are valid, **Then** the server accepts with 202 Accepted and broadcasts each event to subscribers

3. **Given** a Client connects to `/ws?token=<subscriber_token>`, **When** the token is valid, **Then** the connection is established and the client receives all subsequently broadcast events

4. **Given** a Client connects with `?source=laptop-1&type=tool`, **When** events arrive, **Then** the client receives only events matching ALL specified filters (AND logic). Events must have source="laptop-1" AND type="tool" to be delivered.

5. **Given** a Monitor sends more than 100 events/second from the same `X-Source-ID`, **When** the rate limit is exceeded, **Then** the server returns 429 Too Many Requests with a Retry-After header indicating seconds until the bucket refills

6. **Given** a monitoring system queries `/health`, **When** the server is running, **Then** it returns 200 OK with connection statistics

7. **Given** a request with invalid/missing signature (POST /events) or invalid/missing token (GET /ws), **When** authentication is enabled (default), **Then** the server returns 401 Unauthorized

8. **Given** `VIBETEA_UNSAFE_NO_AUTH=true` is set, **When** the server starts, **Then** it logs a warning about disabled authentication and accepts all requests without auth validation

---

### User Story 2 - Monitor Claude Code Watcher (Priority: P1)

**As a** developer using Claude Code,
**I want** a Monitor daemon that watches my Claude Code sessions,
**so that** my AI coding activity is automatically captured and forwarded to VibeTea.

**Why this priority**: Without the Monitor, there's no source of events to aggregate. This is co-equal with the Server as the core functionality.

**Independent Test**: Can be tested by running the Monitor while using Claude Code and verifying events are emitted for tool usage.

**Acceptance Scenarios**:

1. **Given** Claude Code starts a new session, **When** a new `.jsonl` file is created in `~/.claude/projects/`, **Then** the Monitor detects the file within 1 second and begins watching it for new events

2. **Given** a Claude Code session file receives new JSONL lines, **When** the lines contain `assistant` events with tool_use content, **Then** the Monitor parses the tool name and status and extracts the file basename from tool inputs (if applicable)

3. **Given** multiple Claude Code sessions are active simultaneously, **When** events occur in any session, **Then** all events are captured with their respective session IDs

4. **Given** a Claude Code session ends, **When** a `summary` event is written to the session file, **Then** the Monitor emits a session-ended event and can stop watching that file

5. **Given** the Monitor starts while Claude Code sessions are already active, **When** the Monitor initializes, **Then** it discovers existing session files and begins watching from the current file position (no replay)

**Claude Code Event Mapping**:

| Claude Code Type          | VibeTea Type | Extracted Fields                |
| ------------------------- | ------------ | ------------------------------- |
| `assistant` with tool_use | `tool`       | tool name, status="started"     |
| `progress` (PostToolUse)  | `tool`       | tool name, status="completed"   |
| `user`                    | `activity`   | timestamp only                  |
| `summary`                 | `summary`    | summary text (session ended)    |
| First event in new file   | `session`    | action="started", project       |

---

### User Story 3 - Monitor Privacy Pipeline (Priority: P1)

**As a** developer concerned about privacy,
**I want** the Monitor to strip all sensitive data before transmission,
**so that** my code, prompts, and file contents are never sent over the network.

**Why this priority**: Privacy is a non-negotiable requirement. Without this, developers won't trust or use the system.

**Independent Test**: Can be tested by capturing emitted events and verifying no sensitive content is present.

**Acceptance Scenarios**:

1. **Given** an event contains a file path `/home/user/project/src/auth.ts`, **When** the event is processed, **Then** only the basename `auth.ts` is transmitted and the full path is never sent

2. **Given** an event contains file content, diffs, or code, **When** the event is processed, **Then** that content is completely stripped and only metadata (tool name, file basename) is transmitted

3. **Given** an event contains user prompts or assistant responses, **When** the event is processed, **Then** that content is completely stripped and never transmitted

4. **Given** an event contains a Bash tool use, **When** the event is processed, **Then** only the `description` field is transmitted (if present) and the actual command is never sent

5. **Given** an event contains grep patterns or search queries, **When** the event is processed, **Then** the pattern is omitted entirely and only the tool name is transmitted

6. **Given** `VIBETEA_BASENAME_ALLOWLIST=".ts,.js,.rs"` is set, **When** an event contains a file with extension `.env`, **Then** the basename is replaced with `[filtered]` and only allowed extensions are transmitted as-is

**Allowed vs. Prohibited Fields**:

| Allowed (Transmit)         | Prohibited (Never Transmit)  |
| -------------------------- | ---------------------------- |
| Event type                 | File contents, code, diffs   |
| Tool category/name         | User prompts                 |
| Status (started/completed) | Assistant responses          |
| File basename              | Full file paths              |
| Token counts               | Bash commands                |
| Project name               | Search queries, grep patterns|
| Timestamps (ISO-8601)      | URLs, API responses          |
| Session ID (UUID)          | Thinking text, error messages|

---

### User Story 4 - Monitor Server Connection (Priority: P1)

**As a** developer running the Monitor,
**I want** it to maintain a reliable connection to the VibeTea server,
**so that** events are delivered even when the network is unreliable.

**Why this priority**: Reliability is critical for a monitoring system. Users need confidence that their events won't be lost.

**Independent Test**: Can be tested by starting Monitor, disconnecting network, reconnecting, and verifying buffered events are delivered.

**Acceptance Scenarios**:

1. **Given** the Monitor starts with valid `VIBETEA_SERVER_URL` and keypair at `VIBETEA_KEY_PATH`, **When** the server is reachable, **Then** the Monitor establishes a connection and begins sending signed events

2. **Given** the Monitor runs `vibetea init`, **When** no keypair exists at `VIBETEA_KEY_PATH`, **Then** the Monitor generates a new Ed25519 keypair, saves `key.priv` and `key.pub` files, and displays the public key for registration with the server

3. **Given** the Monitor is connected, **When** the connection is lost, **Then** the Monitor buffers events locally and attempts reconnection with exponential backoff

4. **Given** reconnection is needed, **When** attempts fail repeatedly, **Then** delays follow: 1s → 2s → 4s → 8s → 16s → 32s → 60s (max) with each delay having ±25% jitter

5. **Given** the server is unreachable for extended time, **When** the 1000-event buffer fills, **Then** oldest events are dropped (FIFO eviction) and a warning is logged when buffer reaches 80% capacity

6. **Given** the Monitor has buffered events, **When** connection is re-established, **Then** buffered events are sent in order and new events continue flowing

7. **Given** the server returns 429 Too Many Requests, **When** the Monitor receives this response, **Then** it respects the Retry-After header and buffers events during the wait period

---

### User Story 5 - Client WebSocket Connection (Priority: P2)

**As a** dashboard user,
**I want** the Client to maintain a reliable WebSocket connection,
**so that** I see real-time events without manual refresh.

**Why this priority**: The Client enables visualization but isn't required for core functionality. Events can still be collected without a UI.

**Independent Test**: Can be tested by opening dashboard, verifying connection indicator, and testing reconnection after network interruption.

**Acceptance Scenarios**:

1. **Given** the user navigates to `/?token=abc123`, **When** the page loads, **Then** the Client connects to the server with that token and stores the token in localStorage

2. **Given** a token exists in localStorage, **When** the user navigates to `/` (no token in URL), **Then** the Client connects using the stored token

3. **Given** no token in URL or localStorage, **When** the page loads, **Then** a token input form is displayed and the user can enter and submit a token

4. **Given** the user provides an invalid token, **When** connection fails with 401, **Then** an error message is displayed, localStorage is cleared, and the token input form is shown

5. **Given** the Client UI, **When** connection state changes, **Then** a visual indicator shows: Connected (green) / Reconnecting (yellow) / Disconnected (red)

6. **Given** the WebSocket connection is lost, **When** the Client detects disconnection, **Then** it attempts reconnection with exponential backoff and displays "Reconnecting..." status

---

### User Story 6 - Client Live Event Stream (Priority: P2)

**As a** developer viewing the dashboard,
**I want** to see a scrolling feed of real-time events,
**so that** I can monitor AI assistant activity as it happens.

**Why this priority**: This is the primary visualization for real-time monitoring, essential for the dashboard experience.

**Independent Test**: Can be tested by generating events from Monitor and verifying they appear in the stream with correct formatting.

**Acceptance Scenarios**:

1. **Given** the Client is connected, **When** events arrive, **Then** each event is displayed with: timestamp, event type icon, project name, context (basename) and new events animate in from the top

2. **Given** the user has not scrolled up, **When** new events arrive, **Then** the stream automatically scrolls to show newest events

3. **Given** the user scrolls more than 50px from the bottom, **When** new events arrive, **Then** auto-scroll is disabled and a "Jump to latest" button appears

4. **Given** auto-scroll is paused, **When** the user clicks "Jump to latest", **Then** the stream scrolls to the bottom and auto-scroll resumes

5. **Given** auto-scroll is paused, **When** the user manually scrolls to within 50px of the bottom, **Then** auto-scroll resumes and the "Jump to latest" button disappears

6. **Given** events of different types, **When** displayed, **Then** each type has a distinct icon: tool (🔧), activity (💬), session (🚀), summary (📋), error (⚠️)

---

### User Story 7 - Client Activity Heatmap (Priority: P2)

**As a** developer wanting to see patterns,
**I want** a GitHub-style contribution heatmap,
**so that** I can visualize my AI coding activity over time.

**Why this priority**: Pattern visualization provides unique value beyond real-time monitoring.

**Independent Test**: Can be tested by accumulating events over time and verifying heatmap cells reflect correct activity levels.

**Acceptance Scenarios**:

1. **Given** the Client has received events, **When** the heatmap is rendered, **Then** it shows a grid where each cell represents one hour and cells are colored by event volume

2. **Given** cells with varying event counts, **When** rendered, **Then** colors follow: 0 events (#1a1a2e), 1-10 (#2d4a3e), 11-25 (#3d6b4f), 26-50 (#4d8c5f), 51+ (#5dad6f)

3. **Given** the dashboard loads, **When** the heatmap is displayed, **Then** it shows the last 7 days by default

4. **Given** the user clicks "Show 30 days", **When** the view expands, **Then** the heatmap shows the last 30 days

5. **Given** the user is in a specific timezone, **When** the heatmap is displayed, **Then** hours are aligned to the user's local timezone

6. **Given** the user clicks a heatmap cell, **When** the cell is clicked, **Then** the event stream filters to show only events from that hour and the filter can be cleared with a single click

---

### User Story 8 - Client Session Overview (Priority: P2)

**As a** developer with multiple sessions,
**I want** to see cards showing active sessions,
**so that** I can quickly see which projects have active AI assistants.

**Why this priority**: Session awareness helps users understand their concurrent AI usage.

**Independent Test**: Can be tested by starting multiple Claude Code sessions and verifying distinct session cards appear.

**Acceptance Scenarios**:

1. **Given** a session has received events in the last 5 minutes, **When** the session overview is rendered, **Then** a card is displayed showing: project name, session duration, activity indicator

2. **Given** an active session card, **When** events have arrived in the last 60 seconds, **Then** a pulsing activity indicator is displayed with frequency based on event volume: 1-5 events = 1Hz pulse, 6-15 events = 2Hz pulse, 16+ events = 3Hz pulse. Indicator stops pulsing when no events received for 60 seconds.

3. **Given** a session has no events for 5+ minutes but < 30 minutes, **When** the session overview is rendered, **Then** the card shows with "Last active: X minutes ago" label and appears dimmed

4. **Given** a session has no events for 30+ minutes, **When** the session overview updates, **Then** the session card is removed from display

5. **Given** a session card is displayed, **When** the user clicks it, **Then** the event stream filters to that session only and the filter can be cleared

6. **Given** a session receives a summary event (explicit end), **When** the event is processed, **Then** the card shows "Ended" status and follows the 30-minute removal rule

---

### User Story 9 - Client Statistics Panel (Priority: P3)

**As a** developer analyzing my AI usage,
**I want** a statistics panel showing usage metrics,
**so that** I can understand my AI assistant usage patterns.

**Why this priority**: Analytics are valuable but not essential for core monitoring functionality.

**Independent Test**: Can be tested by accumulating events and verifying statistics display correctly for different time periods.

**Acceptance Scenarios**:

1. **Given** the statistics panel, **When** the user selects a time period, **Then** statistics update to reflect: Last hour, Last 24 hours, Last 7 days, or Last 30 days

2. **Given** a selected time period, **When** statistics are displayed, **Then** the total event count is shown prominently

3. **Given** events in the selected period, **When** the breakdown is displayed, **Then** a small bar chart shows count per event type (tool, activity, etc.)

4. **Given** events in the selected period, **When** the project list is displayed, **Then** projects are ranked by event count (top 5)

5. **Given** tool events in the selected period, **When** the breakdown is displayed, **Then** a donut/pie chart shows tool usage distribution

6. **Given** the dashboard loaded at a specific time, **When** statistics are displayed, **Then** a disclaimer shows: "Statistics based on events since [page load time]"

---

### Edge Cases

| ID   | Scenario                                   | Handling                                                              |
| ---- | ------------------------------------------ | --------------------------------------------------------------------- |
| EC-1 | Sensitive filename (e.g., `api-keys.json`) | When `VIBETEA_BASENAME_ALLOWLIST` is set, files with extensions NOT in the allowlist have their basename replaced with `[filtered]`. When unset, all basenames transmitted as-is. Example: allowlist=".ts,.js,.rs" and file is `.env` → transmitted as `[filtered]` |
| EC-2 | Server unavailable during Monitor startup  | Buffer events locally, retry with exponential backoff                 |
| EC-3 | WebSocket disconnect during event stream   | Client auto-reconnects, shows "Reconnecting" status                   |
| EC-4 | High event volume (>100 events/sec)        | Server returns 429, Monitor buffers and retries                       |
| EC-5 | Multiple simultaneous Claude Code sessions | Each session tracked independently by session ID                      |
| EC-6 | Buffer fills during extended outage        | FIFO eviction (drop oldest), log warning at 80%                       |
| EC-7 | User scrolls during high-volume events     | Auto-scroll pauses, "Jump to latest" button shown                     |
| EC-8 | Dashboard opened with no events yet        | Show empty state with "Waiting for events..." message                 |

---

## Requirements

### Functional Requirements

| ID    | Requirement                                                                | Story   |
| ----- | -------------------------------------------------------------------------- | ------- |
| FR-1  | Server MUST accept events via POST /events with Ed25519 signature auth     | Story 1 |
| FR-1a | Server MUST support unsafe mode to disable auth (`VIBETEA_UNSAFE_NO_AUTH`) | Story 1 |
| FR-2  | Server MUST broadcast events to WebSocket subscribers                      | Story 1 |
| FR-3  | Server MUST support filtering by source, type, project (combined with AND logic) | Story 1 |
| FR-4  | Server MUST rate limit at 100 events/sec per unique `X-Source-ID` header value | Story 1 |
| FR-5  | Monitor MUST watch `~/.claude/projects/**/*.jsonl`                         | Story 2 |
| FR-6  | Monitor MUST parse Claude Code JSONL events                                | Story 2 |
| FR-7  | Monitor MUST strip all sensitive data (privacy pipeline)                   | Story 3 |
| FR-8  | Monitor MUST maintain connection with auto-reconnect                       | Story 4 |
| FR-8a | Monitor MUST generate Ed25519 keypair via `vibetea init`                   | Story 4 |
| FR-8b | Monitor MUST sign event batches with private key                           | Story 4 |
| FR-9  | Monitor MUST buffer up to 1000 events during outage                        | Story 4 |
| FR-10 | Client MUST connect via WebSocket with auth token                          | Story 5 |
| FR-11 | Client MUST display live event stream with auto-scroll                     | Story 6 |
| FR-12 | Client MUST display activity heatmap (7/30 day views)                      | Story 7 |
| FR-13 | Client MUST display active session cards                                   | Story 8 |
| FR-14 | Client MUST display statistics panel                                       | Story 9 |

### Non-Functional Requirements

| ID    | Requirement                    | Target           |
| ----- | ------------------------------ | ---------------- |
| NFR-1 | Monitor CPU usage (idle)       | < 1%             |
| NFR-2 | Monitor memory footprint       | < 5 MB           |
| NFR-3 | Monitor binary size            | < 10 MB          |
| NFR-4 | Server concurrent connections  | 100+             |
| NFR-5 | Event latency (end-to-end)     | < 100 ms         |
| NFR-6 | Client time to interactive     | < 2 seconds (3G) |

**NFR-5 Measurement Definition**: Time measured from when a Claude Code event is written to a `.jsonl` file until the corresponding VibeTea event appears in the Client dashboard. Measured on localhost with all components running locally. Network latency in production deployments is additive.

### Key Entities

- **Event**: A single occurrence of AI assistant activity with id, source, timestamp, type, and payload
- **Session**: A Claude Code session identified by UUID, tied to a project, with start/end lifecycle
- **Source**: A unique Monitor instance identified by hostname or custom ID, authenticated via Ed25519 keypair
- **Subscriber**: A Client connected via WebSocket, authenticated via token, optionally filtered by source/type/project

---

## Event Schema

All events broadcast through VibeTea follow this envelope structure:

```json
{
  "id": "evt_abc123",
  "source": "macbook-pro",
  "timestamp": "2025-01-15T14:30:00Z",
  "type": "tool",
  "payload": {
    "sessionId": "sess_xyz789",
    "project": "vibetea",
    "tool": "file_read",
    "status": "completed",
    "context": "server.rs"
  }
}
```

### Event Types

| Type       | Description          | Payload Fields                                   |
| ---------- | -------------------- | ------------------------------------------------ |
| `session`  | Session lifecycle    | `sessionId`, `action` (started/ended), `project` |
| `activity` | General activity     | `sessionId`, `project`                           |
| `tool`     | Tool invocation      | `sessionId`, `tool`, `status`, `context`         |
| `agent`    | Agent state changes  | `sessionId`, `state`                             |
| `summary`  | Session summary      | `sessionId`, `summary`                           |
| `error`    | Error events         | `sessionId`, `category`                          |

---

## API Specification

### Server Endpoints

#### POST /events

Ingest endpoint for Monitors.

| Aspect       | Specification                                                   |
| ------------ | --------------------------------------------------------------- |
| Auth         | Ed25519 signature (see headers below)                           |
| Headers      | `X-Source-ID`: Monitor identifier                               |
|              | `X-Signature`: Base64-encoded Ed25519 signature of request body |
| Content-Type | `application/json`                                              |
| Body         | Single event or array of events                                 |
| Success      | `202 Accepted`                                                  |
| Rate limited | `429 Too Many Requests` with `Retry-After`                      |
| Unauthorized | `401 Unauthorized` (invalid/missing signature or unknown source)|

**Note**: When `VIBETEA_UNSAFE_NO_AUTH=true`, signature headers are ignored.

#### GET /ws

WebSocket endpoint for Clients.

| Aspect   | Specification                      |
| -------- | ---------------------------------- |
| Auth     | Token as query param: `?token=xxx` |
| Filters  | `?source=`, `?type=`, `?project=`  |
| Messages | JSON events pushed by server       |

#### GET /health

Health check endpoint.

| Aspect   | Specification                                      |
| -------- | -------------------------------------------------- |
| Auth     | None                                               |
| Response | `200 OK` with `{"status": "ok", "connections": N}` |

---

## Technical Clarifications

This section addresses implementation details identified during specification review.

### Error Handling

| Component | Failure Mode                | Behavior                                              |
| --------- | --------------------------- | ----------------------------------------------------- |
| Monitor   | Invalid JSONL line          | Log warning with line number, skip line, continue     |
| Monitor   | Invalid UTF-8 in file       | Skip line, log warning, continue                      |
| Monitor   | Missing required config     | Exit with error code 1 and clear message              |
| Server    | Slow WebSocket consumer     | Per-client buffer (100 events), FIFO eviction if full |
| Server    | Invalid event from Monitor  | Return 400 Bad Request with error details             |
| Client    | WebSocket auth failure      | Show error, clear localStorage, display token form    |
| Client    | WebSocket parse error       | Log warning, skip malformed message                   |

### Graceful Shutdown

**Server**:
- Listen for SIGTERM/SIGINT signals
- Stop accepting new connections immediately
- Allow in-flight requests 30 seconds to complete
- Close WebSocket connections with 1001 Going Away code

**Monitor**:
- Flush buffered events on clean shutdown (best effort, 5 second timeout)
- Log shutdown progress

### State Management (Client)

| State               | Retention                    | Persistence           |
| ------------------- | ---------------------------- | --------------------- |
| Event buffer        | Last 1000 events             | In-memory only        |
| Session list        | Sessions with events < 30min | In-memory only        |
| Statistics          | Incremental aggregates       | In-memory only        |
| Auth token          | Until logout/401             | localStorage          |
| Selected time period| Session duration             | In-memory only        |

**Auto-scroll clarification**: Active when scroll position is within 50px of container's maximum scroll height. Pauses when user scrolls beyond threshold. Resumes when user clicks "Jump to latest" OR manually scrolls back within threshold.

**Session state machine**:
```
New File Detected → Active (first event)
Active → Inactive (no events for 5 min)
Inactive → Active (event received)
Active/Inactive → Ended (summary event received)
Ended/Inactive → Removed (30 min since last event)
```

### Event ID Generation

- Generated by: Monitor (at event creation time)
- Format: `evt_` prefix + 20 character alphanumeric random string
- Uniqueness: Sufficient entropy that collisions are negligible

### Timestamps

- All timestamps in UTC
- Serialization format: RFC 3339 (e.g., `2025-01-15T14:30:00Z`)
- Client displays in user's local timezone

### Accessibility (Client)

- Event stream: `aria-live="polite"` for new events
- Connection status: `aria-live="assertive"` for state changes
- All interactive elements keyboard accessible
- Respect `prefers-reduced-motion` for animations
- Heatmap cells include `aria-label` with event count

---

## Configuration

### Server Environment Variables

| Variable                   | Required | Default | Description                                            |
| -------------------------- | -------- | ------- | ------------------------------------------------------ |
| `VIBETEA_PUBLIC_KEYS`      | Yes*     | -       | Authorized Monitor public keys. Format: `source1:pubkey1,source2:pubkey2` where pubkey is base64-encoded Ed25519 public key (44 chars). No spaces. Example: `laptop:MCowBQYDK2VwAyEA...,desktop:MCowBQYDK2VwAyEA...` |
| `VIBETEA_SUBSCRIBER_TOKEN` | Yes*     | -       | Auth token for Clients                                 |
| `PORT`                     | No       | 8080    | HTTP server port                                       |
| `VIBETEA_UNSAFE_NO_AUTH`   | No       | false   | Set `true` to disable all authentication (dev only)    |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor Environment Variables

| Variable                     | Required | Default      | Description                                      |
| ---------------------------- | -------- | ------------ | ------------------------------------------------ |
| `VIBETEA_SERVER_URL`         | Yes      | -            | Server URL (e.g., `https://vibetea.fly.dev`)     |
| `VIBETEA_SOURCE_ID`          | No       | hostname     | Monitor identifier (must match key registration) |
| `VIBETEA_KEY_PATH`           | No       | `~/.vibetea` | Directory containing `key.priv` and `key.pub`    |
| `VIBETEA_CLAUDE_DIR`         | No       | `~/.claude`  | Claude Code directory                            |
| `VIBETEA_BUFFER_SIZE`        | No       | 1000         | Event buffer capacity                            |
| `VIBETEA_BASENAME_ALLOWLIST` | No       | (all)        | Comma-separated extensions to allow              |

---

## Development Standards

### Code Quality

| Requirement | Rust (Monitor & Server)      | TypeScript (Client)             |
| ----------- | ---------------------------- | ------------------------------- |
| Formatting  | rustfmt (default settings)   | Prettier (project config)       |
| Linting     | clippy (default lints min)   | ESLint (recommended + React)    |
| Type safety | Native                       | Strict mode (`strict: true`)    |

### Git Hooks

Pre-commit hooks via **Lefthook** enforce:
- Formatting checks (rustfmt, Prettier)
- Linting (clippy, ESLint)
- Type checking (cargo check, tsc --noEmit)
- No sensitive data in commits (privacy validation)

### Testing Requirements

| Component | Framework   | Scope                                        |
| --------- | ----------- | -------------------------------------------- |
| Monitor   | cargo test  | Unit tests + integration with mock server    |
| Server    | cargo test  | Unit tests + WebSocket integration tests     |
| Client    | Vitest      | Component tests + WebSocket mock tests       |

### CI/CD Pipeline

GitHub Actions workflow MUST:
- Run all tests on every PR
- Verify formatting and linting pass
- Build release binaries (Rust)
- Build production client bundle

### Deployment

| Component | Target | Requirements                                    |
| --------- | ------ | ----------------------------------------------- |
| Server    | Fly.io | Single Rust binary in Docker, auto-deploy main  |
| Client    | CDN    | Static build output, deployable to any CDN      |
| Monitor   | Local  | Cross-platform binaries (Linux, macOS, Windows) |

---

## Success Criteria

### Measurable Outcomes

| ID   | Criterion                                     | Measurement                        |
| ---- | --------------------------------------------- | ---------------------------------- |
| SC-1 | Developer sees activity within 5 min of setup | Time from install to first event   |
| SC-2 | Events appear within 200ms of occurring       | End-to-end latency measurement     |
| SC-3 | System runs 24+ hours without intervention    | Uptime without crashes/restarts    |
| SC-4 | Monitor binary under 10MB                     | File size measurement              |
| SC-5 | Monitor uses minimal resources                | < 1% CPU, < 5MB memory during idle |
| SC-6 | All CI checks pass on every merged PR         | GitHub Actions success rate        |

---

## Assumptions

- Users have Claude Code installed and actively use it for development
- Users are comfortable with environment variable configuration
- Server deployment target is Fly.io (affects binary requirements)
- Client deployment target is static CDN hosting
- v1 focuses on single-user/small team use cases (no multi-tenancy)

---

## Out of Scope (v1)

- Event persistence/history (events are transient)
- User accounts or multi-tenancy
- Mobile-specific client
- Cursor, Copilot, or other agent monitors
- Custom event types from Monitors
- Analytics, aggregations, or derived metrics beyond basic statistics

---

## Glossary

| Term             | Definition                                                                     |
| ---------------- | ------------------------------------------------------------------------------ |
| Monitor          | Lightweight daemon that watches AI agent activity and forwards normalized events |
| Server           | Central event hub that receives events from Monitors and broadcasts to Clients |
| Client           | Web-based dashboard that visualizes the real-time event stream                 |
| Event Envelope   | Standard JSON wrapper containing id, source, timestamp, type, payload          |
| Privacy Pipeline | Monitor component that strips sensitive data before transmission               |
| JSONL            | JSON Lines format—one JSON object per line                                     |
| Ed25519          | Elliptic curve signature algorithm used for Monitor authentication             |
| Keypair          | Monitor's private key (signing) and public key (verification)                  |
| Subscriber Token | Auth token for Clients to receive events                                       |
| Unsafe Mode      | Development mode that disables all authentication                              |
