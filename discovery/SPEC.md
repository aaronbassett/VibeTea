# Feature Specification: VibeTea

**Feature Branch**: `feature/vibetea`
**Created**: 2026-02-02
**Last Updated**: 2026-02-02 13:00 UTC
**Status**: Complete (Ready for Implementation)
**Discovery**: See `discovery/` folder for full context

---

## Problem Statement

Modern developers increasingly rely on AI coding assistants, often running multiple agents (Claude Code, Cursor, Copilot) across different projects simultaneously. Currently, there is no unified way to:

- Monitor AI agent activity across multiple tools and sessions
- Visualize coding patterns and AI utilization over time
- Build integrations that react to AI assistant events in real-time
- Understand how AI assistants are being used across a team or organization

Existing solutions like PixelHQ-bridge are tightly coupled to specific visualization clients and don't provide the flexibility needed for custom integrations or multi-source aggregation.

**VibeTea** solves this by providing a real-time event aggregation and broadcast system that consolidates activity streams from multiple AI agents into a unified WebSocket feed. Privacy is paramount—VibeTea broadcasts only structural metadata (event types, tool categories, timestamps) and never transmits code, prompts, file contents, or any sensitive information.

## Personas

| Persona | Description | Primary Goals |
|---------|-------------|---------------|
| **Solo Developer** | Individual developer using AI coding assistants (Claude Code, Cursor, Copilot) daily for coding tasks | Wants visibility into their AI-assisted workflow, see patterns in how they use AI, and understand their utilization over time |
| **Team Lead** | Technical lead or engineering manager overseeing a team of developers using AI assistants | Wants to understand how AI assistants are being used across the team, identify patterns, and ensure effective utilization |
| **Integration Builder** | Developer building custom tooling, dashboards, or automations that consume AI activity data | Wants a reliable, well-documented event stream API to build custom integrations without coupling to specific visualization clients |

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

### Story 1: Server Event Hub Core (P1)

**As a** VibeTea operator,
**I want** a central server that receives events from Monitors and broadcasts them to Clients,
**so that** I can run a reliable event hub that connects multiple sources to multiple consumers.

#### Acceptance Scenarios

**Scenario 1.1: Event Ingestion**
- GIVEN a Monitor sends a POST request to `/events` with valid signature headers
- WHEN the request contains a valid event JSON and the source's public key is registered
- THEN the server accepts with 202 Accepted
- AND broadcasts the event to all connected WebSocket clients

**Scenario 1.2: Batch Event Ingestion**
- GIVEN a Monitor sends an array of events to `/events`
- WHEN all events are valid
- THEN the server accepts with 202 Accepted
- AND broadcasts each event to subscribers

**Scenario 1.3: WebSocket Subscription**
- GIVEN a Client connects to `/ws?token=<subscriber_token>`
- WHEN the token is valid
- THEN the connection is established
- AND the client receives all subsequently broadcast events

**Scenario 1.4: Subscription Filtering**
- GIVEN a Client connects with `?source=laptop-1&type=tool`
- WHEN events arrive
- THEN the client receives only events matching both filters

**Scenario 1.5: Rate Limiting**
- GIVEN a Monitor sends more than 100 events/second
- WHEN the rate limit is exceeded
- THEN the server returns 429 Too Many Requests
- AND includes a Retry-After header

**Scenario 1.6: Health Check**
- GIVEN a monitoring system queries `/health`
- WHEN the server is running
- THEN it returns 200 OK with connection statistics

**Scenario 1.7: Invalid Auth**
- GIVEN a request with invalid/missing signature (POST /events) or invalid/missing token (GET /ws)
- WHEN authentication is enabled (default)
- THEN the server returns 401 Unauthorized

**Scenario 1.8: Unsafe Mode**
- GIVEN `VIBETEA_UNSAFE_NO_AUTH=true` is set
- WHEN the server starts
- THEN it logs a warning about disabled authentication
- AND accepts all requests without auth validation

#### Technical Requirements

| Requirement | Value |
|-------------|-------|
| Framework | Rust + axum + tokio |
| Concurrent WebSocket connections | 100+ |
| Rate limit (per source) | 100 events/sec, 200 burst |
| Rate limit (global) | 1000 events/sec |
| Monitor auth | Ed25519 signature verification (`VIBETEA_PUBLIC_KEYS`) |
| Client auth | Static token (`VIBETEA_SUBSCRIBER_TOKEN`) |
| Unsafe mode | `VIBETEA_UNSAFE_NO_AUTH=true` disables all auth |
| Deployment | Single binary, Fly.io compatible |

---

### Story 2: Monitor Claude Code Watcher (P1)

**As a** developer using Claude Code,
**I want** a Monitor daemon that watches my Claude Code sessions,
**so that** my AI coding activity is automatically captured and forwarded to VibeTea.

#### Acceptance Scenarios

**Scenario 2.1: Session Detection**
- GIVEN Claude Code starts a new session
- WHEN a new `.jsonl` file is created in `~/.claude/projects/`
- THEN the Monitor detects the file within 1 second
- AND begins watching it for new events

**Scenario 2.2: Event Parsing**
- GIVEN a Claude Code session file receives new JSONL lines
- WHEN the lines contain `assistant` events with tool_use content
- THEN the Monitor parses the tool name and status
- AND extracts the file basename from tool inputs (if applicable)

**Scenario 2.3: Multi-Session Support**
- GIVEN multiple Claude Code sessions are active simultaneously
- WHEN events occur in any session
- THEN all events are captured with their respective session IDs

**Scenario 2.4: Session End Detection**
- GIVEN a Claude Code session ends
- WHEN a `summary` event is written to the session file
- THEN the Monitor emits a session-ended event
- AND can stop watching that file

**Scenario 2.5: Startup with Existing Sessions**
- GIVEN the Monitor starts while Claude Code sessions are already active
- WHEN the Monitor initializes
- THEN it discovers existing session files
- AND begins watching from the current file position (no replay)

#### Technical Requirements

| Requirement | Value |
|-------------|-------|
| Language | Rust |
| File watching | notify crate (inotify on Linux, FSEvents on macOS) |
| Signing | ed25519-dalek crate |
| Watch path | `~/.claude/projects/**/*.jsonl` (configurable via `VIBETEA_CLAUDE_DIR`) |
| Key path | `~/.vibetea/key.priv`, `~/.vibetea/key.pub` (configurable via `VIBETEA_KEY_PATH`) |
| CPU usage (idle) | < 1% |
| Memory footprint | < 5 MB |
| Binary size | < 10 MB |

#### Claude Code Event Mapping

| Claude Code Type | VibeTea Type | Extracted Fields |
|------------------|--------------|------------------|
| `assistant` with tool_use | `tool` | tool name, status="started" |
| `progress` (PostToolUse) | `tool` | tool name, status="completed" |
| `user` | `activity` | timestamp only |
| `summary` | `summary` | summary text (session ended) |
| First event in new file | `session` | action="started", project |

---

### Story 3: Monitor Privacy Pipeline (P1)

**As a** developer concerned about privacy,
**I want** the Monitor to strip all sensitive data before transmission,
**so that** my code, prompts, and file contents are never sent over the network.

#### Acceptance Scenarios

**Scenario 3.1: File Path Stripping**
- GIVEN an event contains a file path `/home/user/project/src/auth.ts`
- WHEN the event is processed
- THEN only the basename `auth.ts` is transmitted
- AND the full path is never sent

**Scenario 3.2: Code Content Blocking**
- GIVEN an event contains file content, diffs, or code
- WHEN the event is processed
- THEN that content is completely stripped
- AND only metadata (tool name, file basename) is transmitted

**Scenario 3.3: Prompt Blocking**
- GIVEN an event contains user prompts or assistant responses
- WHEN the event is processed
- THEN that content is completely stripped
- AND never transmitted

**Scenario 3.4: Bash Command Handling**
- GIVEN an event contains a Bash tool use
- WHEN the event is processed
- THEN only the `description` field is transmitted (if present)
- AND the actual command is never sent

**Scenario 3.5: Search Query Blocking**
- GIVEN an event contains grep patterns or search queries
- WHEN the event is processed
- THEN the pattern is omitted entirely
- AND only the tool name is transmitted

**Scenario 3.6: Optional Basename Allowlist**
- GIVEN `VIBETEA_BASENAME_ALLOWLIST=".ts,.js,.rs"` is set
- WHEN an event contains a file with extension `.env`
- THEN the basename is replaced with `[filtered]`
- AND only allowed extensions are transmitted as-is

#### Allowed vs. Prohibited Fields

| Allowed (Transmit) | Prohibited (Never Transmit) |
|--------------------|------------------------------|
| Event type | File contents, code, diffs |
| Tool category/name | User prompts |
| Status (started/completed) | Assistant responses |
| File basename | Full file paths |
| Token counts | Bash commands |
| Project name | Search queries, grep patterns |
| Timestamps (ISO-8601) | URLs, API responses |
| Session ID (UUID) | Thinking text, error messages |

---

### Story 4: Monitor Server Connection (P1)

**As a** developer running the Monitor,
**I want** it to maintain a reliable connection to the VibeTea server,
**so that** events are delivered even when the network is unreliable.

#### Acceptance Scenarios

**Scenario 4.1: Initial Connection**
- GIVEN the Monitor starts with valid `VIBETEA_SERVER_URL` and keypair at `VIBETEA_KEY_PATH`
- WHEN the server is reachable
- THEN the Monitor establishes a connection
- AND begins sending signed events

**Scenario 4.1a: Key Generation**
- GIVEN the Monitor runs `vibetea init`
- WHEN no keypair exists at `VIBETEA_KEY_PATH`
- THEN the Monitor generates a new Ed25519 keypair
- AND saves `key.priv` (private) and `key.pub` (public) files
- AND displays the public key for registration with the server

**Scenario 4.2: Connection Loss Recovery**
- GIVEN the Monitor is connected
- WHEN the connection is lost
- THEN the Monitor buffers events locally
- AND attempts reconnection with exponential backoff

**Scenario 4.3: Exponential Backoff**
- GIVEN reconnection is needed
- WHEN attempts fail repeatedly
- THEN delays follow: 1s → 2s → 4s → 8s → 16s → 32s → 60s (max)
- AND each delay has ±25% jitter

**Scenario 4.4: Buffer Capacity**
- GIVEN the server is unreachable for extended time
- WHEN the 1000-event buffer fills
- THEN oldest events are dropped (FIFO eviction)
- AND a warning is logged when buffer reaches 80% capacity

**Scenario 4.5: Buffer Flush on Reconnect**
- GIVEN the Monitor has buffered events
- WHEN connection is re-established
- THEN buffered events are sent in order
- AND new events continue flowing

**Scenario 4.6: Rate Limit Handling**
- GIVEN the server returns 429 Too Many Requests
- WHEN the Monitor receives this response
- THEN it respects the Retry-After header
- AND buffers events during the wait period

#### Technical Requirements

| Parameter | Value |
|-----------|-------|
| Initial backoff delay | 1 second |
| Maximum backoff delay | 60 seconds |
| Backoff multiplier | 2x |
| Jitter | ±25% |
| Maximum retries | Unlimited |
| Buffer capacity | 1000 events (configurable via `VIBETEA_BUFFER_SIZE`) |
| Buffer overflow behavior | FIFO eviction (drop oldest) |

---

### Story 5: Client WebSocket Connection (P2)

**As a** dashboard user,
**I want** the Client to maintain a reliable WebSocket connection,
**so that** I see real-time events without manual refresh.

#### Acceptance Scenarios

**Scenario 5.1: Token from URL**
- GIVEN the user navigates to `/?token=abc123`
- WHEN the page loads
- THEN the Client connects to the server with that token
- AND stores the token in localStorage

**Scenario 5.2: Token from LocalStorage**
- GIVEN a token exists in localStorage
- WHEN the user navigates to `/` (no token in URL)
- THEN the Client connects using the stored token

**Scenario 5.3: Token Input Form**
- GIVEN no token in URL or localStorage
- WHEN the page loads
- THEN a token input form is displayed
- AND the user can enter and submit a token

**Scenario 5.4: Invalid Token**
- GIVEN the user provides an invalid token
- WHEN connection fails with 401
- THEN an error message is displayed
- AND localStorage is cleared
- AND the token input form is shown

**Scenario 5.5: Connection Status Indicator**
- GIVEN the Client UI
- WHEN connection state changes
- THEN a visual indicator shows: Connected (green) / Reconnecting (yellow) / Disconnected (red)

**Scenario 5.6: Auto-Reconnect**
- GIVEN the WebSocket connection is lost
- WHEN the Client detects disconnection
- THEN it attempts reconnection with exponential backoff
- AND displays "Reconnecting..." status

---

### Story 6: Client Live Event Stream (P2)

**As a** developer viewing the dashboard,
**I want** to see a scrolling feed of real-time events,
**so that** I can monitor AI assistant activity as it happens.

#### Acceptance Scenarios

**Scenario 6.1: Event Display**
- GIVEN the Client is connected
- WHEN events arrive
- THEN each event is displayed with: timestamp, event type icon, project name, context (basename)
- AND new events animate in from the top

**Scenario 6.2: Auto-Scroll Active**
- GIVEN the user has not scrolled up
- WHEN new events arrive
- THEN the stream automatically scrolls to show newest events

**Scenario 6.3: Auto-Scroll Paused**
- GIVEN the user scrolls more than 50px from the bottom
- WHEN new events arrive
- THEN auto-scroll is disabled
- AND a "Jump to latest" button appears

**Scenario 6.4: Resume Auto-Scroll (Button)**
- GIVEN auto-scroll is paused
- WHEN the user clicks "Jump to latest"
- THEN the stream scrolls to the bottom
- AND auto-scroll resumes

**Scenario 6.5: Resume Auto-Scroll (Manual)**
- GIVEN auto-scroll is paused
- WHEN the user manually scrolls to within 50px of the bottom
- THEN auto-scroll resumes
- AND the "Jump to latest" button disappears

**Scenario 6.6: Event Type Icons**
- GIVEN events of different types
- WHEN displayed
- THEN each type has a distinct icon: tool (🔧), activity (💬), session (🚀), summary (📋), error (⚠️)

---

### Story 7: Client Activity Heatmap (P2)

**As a** developer wanting to see patterns,
**I want** a GitHub-style contribution heatmap,
**so that** I can visualize my AI coding activity over time.

#### Acceptance Scenarios

**Scenario 7.1: Heatmap Display**
- GIVEN the Client has received events
- WHEN the heatmap is rendered
- THEN it shows a grid where each cell represents one hour
- AND cells are colored by event volume

**Scenario 7.2: Color Scale**
- GIVEN cells with varying event counts
- WHEN rendered
- THEN colors follow: 0 events (#1a1a2e), 1-10 (#2d4a3e), 11-25 (#3d6b4f), 26-50 (#4d8c5f), 51+ (#5dad6f)

**Scenario 7.3: Default View**
- GIVEN the dashboard loads
- WHEN the heatmap is displayed
- THEN it shows the last 7 days by default

**Scenario 7.4: Extended View**
- GIVEN the user clicks "Show 30 days"
- WHEN the view expands
- THEN the heatmap shows the last 30 days

**Scenario 7.5: Timezone Handling**
- GIVEN the user is in a specific timezone
- WHEN the heatmap is displayed
- THEN hours are aligned to the user's local timezone

**Scenario 7.6: Cell Interaction**
- GIVEN the user clicks a heatmap cell
- WHEN the cell is clicked
- THEN the event stream filters to show only events from that hour
- AND the filter can be cleared with a single click

---

### Story 8: Client Session Overview (P2)

**As a** developer with multiple sessions,
**I want** to see cards showing active sessions,
**so that** I can quickly see which projects have active AI assistants.

#### Acceptance Scenarios

**Scenario 8.1: Active Session Card**
- GIVEN a session has received events in the last 5 minutes
- WHEN the session overview is rendered
- THEN a card is displayed showing: project name, session duration, activity indicator

**Scenario 8.2: Activity Indicator**
- GIVEN an active session card
- WHEN events have recently arrived
- THEN a sparkline or pulsing dot shows recent activity frequency

**Scenario 8.3: Inactive Session Display**
- GIVEN a session has no events for 5+ minutes but < 30 minutes
- WHEN the session overview is rendered
- THEN the card shows with "Last active: X minutes ago" label
- AND appears dimmed

**Scenario 8.4: Session Removal**
- GIVEN a session has no events for 30+ minutes
- WHEN the session overview updates
- THEN the session card is removed from display

**Scenario 8.5: Session Click Filter**
- GIVEN a session card is displayed
- WHEN the user clicks it
- THEN the event stream filters to that session only
- AND the filter can be cleared

**Scenario 8.6: Session End Event**
- GIVEN a session receives a summary event (explicit end)
- WHEN the event is processed
- THEN the card shows "Ended" status
- AND follows the 30-minute removal rule

---

### Story 9: Client Statistics Panel (P3)

**As a** developer analyzing my AI usage,
**I want** a statistics panel showing usage metrics,
**so that** I can understand my AI assistant usage patterns.

#### Acceptance Scenarios

**Scenario 9.1: Time Period Selection**
- GIVEN the statistics panel
- WHEN the user selects a time period
- THEN statistics update to reflect: Last hour, Last 24 hours, Last 7 days, or Last 30 days

**Scenario 9.2: Total Events Count**
- GIVEN a selected time period
- WHEN statistics are displayed
- THEN the total event count is shown prominently

**Scenario 9.3: Events by Type Chart**
- GIVEN events in the selected period
- WHEN the breakdown is displayed
- THEN a small bar chart shows count per event type (tool, activity, etc.)

**Scenario 9.4: Most Active Projects**
- GIVEN events in the selected period
- WHEN the project list is displayed
- THEN projects are ranked by event count (top 5)

**Scenario 9.5: Tool Usage Breakdown**
- GIVEN tool events in the selected period
- WHEN the breakdown is displayed
- THEN a donut/pie chart shows tool usage distribution

**Scenario 9.6: Data Disclaimer**
- GIVEN the dashboard loaded at a specific time
- WHEN statistics are displayed
- THEN a disclaimer shows: "Statistics based on events since [page load time]"

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

| Type | Description | Payload Fields |
|------|-------------|----------------|
| `session` | Session lifecycle | `sessionId`, `action` (started/ended), `project` |
| `activity` | General activity | `sessionId`, `project` |
| `tool` | Tool invocation | `sessionId`, `tool`, `status`, `context` |
| `agent` | Agent state changes | `sessionId`, `state` |
| `summary` | Session summary | `sessionId`, `summary` |
| `error` | Error events | `sessionId`, `category` |

---

## Edge Cases

| ID | Scenario | Handling |
|----|----------|----------|
| EC-1 | Sensitive filename (e.g., `api-keys.json`) | Transmit as-is by default; use `VIBETEA_BASENAME_ALLOWLIST` to filter |
| EC-2 | Server unavailable during Monitor startup | Buffer events locally, retry with exponential backoff |
| EC-3 | WebSocket disconnect during event stream | Client auto-reconnects, shows "Reconnecting" status |
| EC-4 | High event volume (>100 events/sec) | Server returns 429, Monitor buffers and retries |
| EC-5 | Multiple simultaneous Claude Code sessions | Each session tracked independently by session ID |
| EC-6 | Buffer fills during extended outage | FIFO eviction (drop oldest), log warning at 80% |
| EC-7 | User scrolls during high-volume events | Auto-scroll pauses, "Jump to latest" button shown |
| EC-8 | Dashboard opened with no events yet | Show empty state with "Waiting for events..." message |

---

## Requirements Summary

### Functional Requirements

| ID | Requirement | Story |
|----|-------------|-------|
| FR-1 | Server accepts events via POST /events with Ed25519 signature auth | Story 1 |
| FR-1a | Server supports unsafe mode to disable auth (`VIBETEA_UNSAFE_NO_AUTH`) | Story 1 |
| FR-2 | Server broadcasts events to WebSocket subscribers | Story 1 |
| FR-3 | Server supports filtering by source, type, project | Story 1 |
| FR-4 | Server rate limits at 100 events/sec per source | Story 1 |
| FR-5 | Monitor watches `~/.claude/projects/**/*.jsonl` | Story 2 |
| FR-6 | Monitor parses Claude Code JSONL events | Story 2 |
| FR-7 | Monitor strips all sensitive data (privacy pipeline) | Story 3 |
| FR-8 | Monitor maintains connection with auto-reconnect | Story 4 |
| FR-8a | Monitor generates Ed25519 keypair via `vibetea init` | Story 4 |
| FR-8b | Monitor signs event batches with private key | Story 4 |
| FR-9 | Monitor buffers up to 1000 events during outage | Story 4 |
| FR-10 | Client connects via WebSocket with auth token | Story 5 |
| FR-11 | Client displays live event stream with auto-scroll | Story 6 |
| FR-12 | Client displays activity heatmap (7/30 day views) | Story 7 |
| FR-13 | Client displays active session cards | Story 8 |
| FR-14 | Client displays statistics panel | Story 9 |

### Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1 | Monitor CPU usage (idle) | < 1% |
| NFR-2 | Monitor memory footprint | < 5 MB |
| NFR-3 | Monitor binary size | < 10 MB |
| NFR-4 | Server concurrent connections | 100+ |
| NFR-5 | Event latency (end-to-end) | < 100 ms |
| NFR-6 | Client time to interactive | < 2 seconds (3G) |

---

## Success Criteria

| ID | Criterion | Measurement |
|----|-----------|-------------|
| SC-1 | Developer sees activity within 5 min of setup | Time from install to first event |
| SC-2 | Events appear within 200ms of occurring | End-to-end latency |
| SC-3 | System runs 24+ hours without intervention | Uptime without crashes/restarts |
| SC-4 | Monitor binary under 10MB | File size |
| SC-5 | Monitor uses minimal resources | < 1% CPU, < 5MB memory |

---

## API Specification

### Server Endpoints

#### POST /events
Ingest endpoint for Monitors.

| Aspect | Specification |
|--------|---------------|
| Auth | Ed25519 signature (see headers below) |
| Headers | `X-Source-ID`: Monitor identifier |
| | `X-Signature`: Base64-encoded Ed25519 signature of request body |
| Content-Type | `application/json` |
| Body | Single event or array of events |
| Success | `202 Accepted` |
| Rate limited | `429 Too Many Requests` with `Retry-After` |
| Unauthorized | `401 Unauthorized` (invalid/missing signature or unknown source) |

**Note**: When `VIBETEA_UNSAFE_NO_AUTH=true`, signature headers are ignored.

#### GET /ws
WebSocket endpoint for Clients.

| Aspect | Specification |
|--------|---------------|
| Auth | Token as query param: `?token=xxx` |
| Filters | `?source=`, `?type=`, `?project=` |
| Messages | JSON events pushed by server |

#### GET /health
Health check endpoint.

| Aspect | Specification |
|--------|---------------|
| Auth | None |
| Response | `200 OK` with `{"status": "ok", "connections": N}` |

---

## Configuration

### Server Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VIBETEA_PUBLIC_KEYS` | Yes* | - | Authorized Monitor public keys (`source1:pubkey1,source2:pubkey2`) |
| `VIBETEA_SUBSCRIBER_TOKEN` | Yes* | - | Auth token for Clients |
| `PORT` | No | 8080 | HTTP server port |
| `VIBETEA_UNSAFE_NO_AUTH` | No | false | Set `true` to disable all authentication (dev only) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VIBETEA_SERVER_URL` | Yes | - | Server URL (e.g., `https://vibetea.fly.dev`) |
| `VIBETEA_SOURCE_ID` | No | hostname | Monitor identifier (must match key registration) |
| `VIBETEA_KEY_PATH` | No | `~/.vibetea` | Directory containing `key.priv` and `key.pub` |
| `VIBETEA_CLAUDE_DIR` | No | `~/.claude` | Claude Code directory |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |
| `VIBETEA_BASENAME_ALLOWLIST` | No | (all) | Comma-separated extensions to allow |

---

## Technology Stack

| Component | Technology |
|-----------|------------|
| Monitor | Rust, notify crate, tokio, ed25519-dalek |
| Server | Rust, axum, tokio, ed25519-dalek |
| Client | React, TypeScript |
| Deployment | Fly.io (Server), CDN (Client) |

---

## Glossary

| Term | Definition |
|------|------------|
| Monitor | Lightweight daemon that watches AI agent activity and forwards normalized events |
| Server | Central event hub that receives events from Monitors and broadcasts to Clients |
| Client | Web-based dashboard that visualizes the real-time event stream |
| Event Envelope | Standard JSON wrapper containing id, source, timestamp, type, payload |
| Privacy Pipeline | Monitor component that strips sensitive data before transmission |
| JSONL | JSON Lines format—one JSON object per line |
| Ed25519 | Elliptic curve signature algorithm used for Monitor authentication |
| Keypair | Monitor's private key (signing) and public key (verification) |
| Subscriber Token | Auth token for Clients to receive events |
| Unsafe Mode | Development mode that disables all authentication (`VIBETEA_UNSAFE_NO_AUTH=true`) |

---

## Out of Scope (v1)

- Event persistence/history
- User accounts or multi-tenancy
- Mobile-specific client
- Cursor, Copilot, or other agent monitors
- Custom event types from Monitors
- Analytics, aggregations, or derived metrics

---

## Appendix: Story Revision History

| Date | Story | Change | Reason |
|------|-------|--------|--------|
| *No revisions yet* | - | - | - |

---

## Appendix: Decision Log Summary

| ID | Decision | See Details |
|----|----------|-------------|
| D1 | Rate limiting: 100/sec per source, 429 response | `archive/DECISIONS.md` |
| D2 | Ed25519 signing for Monitors, static token for Clients, optional unsafe mode | `archive/DECISIONS.md` |
| D3 | Reconnection: 1s→60s backoff, FIFO eviction | `archive/DECISIONS.md` |
| D4 | Basenames transmitted as-is with optional allowlist | `archive/DECISIONS.md` |
| D5 | Client auth via URL param + localStorage | `archive/DECISIONS.md` |
| D6 | Session active = event in last 5 min | `archive/DECISIONS.md` |
| D7 | Heatmap: 5 color levels, absolute thresholds | `archive/DECISIONS.md` |
| D8 | Stats: preset time periods, client-side calc | `archive/DECISIONS.md` |
| D9 | Auto-scroll: 50px threshold, explicit resume only | `archive/DECISIONS.md` |

---

## Appendix: Research Summary

| ID | Finding | See Details |
|----|---------|-------------|
| R1 | Claude Code JSONL format documented | `archive/RESEARCH.md` |
| R2 | History file contains prompts (prohibited) | `archive/RESEARCH.md` |
| R3 | Session lifecycle: file-per-session, summary event on end | `archive/RESEARCH.md` |
