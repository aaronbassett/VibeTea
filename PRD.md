# VibeTea

**Product Requirements Document**

| | |
|----------|------------------|
| Version  | 1.0              |
| Date     | January 2025     |
| Status   | Draft            |
| Author   | Aaron Bassett    |

---

## Executive Summary

VibeTea is a real-time event aggregation and broadcast system for AI coding assistants. It consolidates activity streams from multiple AI agents (Claude Code, Cursor, Copilot, etc.) into a unified WebSocket feed, enabling developers to build dashboards, analytics tools, and integrations that provide visibility into their AI-assisted development workflow.

The system consists of three components: a **Monitor** that watches local AI agent activity and extracts events, a **Server** that aggregates events from multiple sources and rebroadcasts them to subscribers, and a **Client** web application that visualizes the event stream in real-time.

*Privacy is paramount.* VibeTea broadcasts only structural metadata (event types, tool categories, timestamps) and never transmits code, prompts, file contents, or any sensitive information.

---

## Problem Statement

Modern developers increasingly rely on AI coding assistants, often running multiple agents across different projects simultaneously. Currently, there is no unified way to:

- Monitor AI agent activity across multiple tools and sessions
- Visualize coding patterns and AI utilization over time
- Build integrations that react to AI assistant events in real-time
- Understand how AI assistants are being used across a team or organization

Existing solutions like PixelHQ-bridge are tightly coupled to specific visualization clients and don't provide the flexibility needed for custom integrations or multi-source aggregation.

---

## System Architecture

VibeTea follows a hub-and-spoke architecture where the Server acts as a central event bus.

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

---

## Component 1: VibeTea Monitor

The Monitor is a lightweight daemon that runs on a developer's machine, watching AI agent activity and forwarding normalized events to the VibeTea Server.

### Technical Requirements

- Written in Rust for minimal resource footprint and easy distribution as a single static binary
- Watches Claude Code session files (`~/.claude/projects/**/*.jsonl`) using filesystem notifications
- Parses JSONL events and transforms them into the VibeTea event schema
- Maintains persistent WebSocket connection to the VibeTea Server
- Automatic reconnection with exponential backoff on connection loss
- Local event buffering during server unavailability (configurable buffer size)
- Support for multiple simultaneous AI agent sources (Claude Code initially, Cursor and others planned)

### Privacy Pipeline

The Monitor must implement strict privacy controls. The following data is **allowed** to be transmitted:

| Field | Example | Purpose |
|-------|---------|---------|
| Event type | `"tool"`, `"activity"` | Event categorization |
| Tool category | `"file_read"`, `"terminal"` | UI visualization |
| Status | `"started"`, `"completed"` | Animation triggers |
| File basename | `"auth.ts"` | Context display |
| Token counts | `{ input: 5000 }` | Usage metrics |
| Project name | `"my-app"` | Project grouping |
| Timestamps | ISO-8601 | Event ordering |
| Session ID | UUID | Session correlation |

The following data must **never** be transmitted:

- File contents, code, diffs, or any source material
- User prompts or assistant responses
- Full file paths (strip to basename only)
- Bash commands (only the user-provided description field)
- Search queries, URLs, or API responses
- Thinking text, error messages, or tool output

### Configuration

The Monitor should be configurable via environment variables and/or a config file:

| Variable | Default | Description |
|----------|---------|-------------|
| `VIBETEA_SERVER_URL` | Required | WebSocket URL of VibeTea Server |
| `VIBETEA_AUTH_TOKEN` | Required | Authentication bearer token |
| `VIBETEA_SOURCE_ID` | hostname | Unique identifier for this monitor |
| `VIBETEA_CLAUDE_DIR` | `~/.claude` | Claude Code config directory |
| `VIBETEA_BUFFER_SIZE` | 1000 | Events to buffer during disconnect |

---

## Component 2: VibeTea Server

The Server is a lightweight event hub that receives events from Monitors and broadcasts them to subscribed Clients. It performs no persistence and maintains no state beyond active connections.

### Technical Requirements

- Written in Rust using axum for HTTP/WebSocket handling and tokio for async runtime
- Single static binary deployable to Fly.io (or similar) with minimal configuration
- Support for hundreds of concurrent WebSocket connections
- Bearer token authentication for both Monitors (publishers) and Clients (subscribers)
- Health check endpoint for container orchestration
- Graceful shutdown with connection draining

### API Specification

#### POST /events

Ingest endpoint for Monitors to push events. Accepts single events or batches.

**Authentication:** Bearer token in Authorization header

#### GET /ws

WebSocket endpoint for Clients to subscribe to the event stream.

**Authentication:** Bearer token as query parameter (`?token=xxx`)

**Optional Filters:**

- `?source=<source_id>` — Filter events from a specific Monitor
- `?type=<event_type>` — Filter by event type (tool, activity, session, etc.)
- `?project=<project_name>` — Filter by project name

#### GET /health

Health check endpoint returning 200 OK with connection statistics.

### Event Schema

All events broadcast to Clients follow this envelope structure:

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

**Event Types:** `session`, `activity`, `tool`, `agent`, `summary`, `error`

---

## Component 3: VibeTea Client (Dashboard)

The Client is a web-based dashboard that visualizes the real-time event stream. It provides at-a-glance visibility into AI coding assistant activity, similar to GitHub's contribution activity view.

### Technical Requirements

- Single-page application built with React and TypeScript
- Real-time updates via WebSocket connection to VibeTea Server
- Responsive design supporting desktop and tablet viewports
- Dark mode support (default) with optional light mode toggle
- Static deployment to CDN (Vercel, Cloudflare Pages, or similar)

### Dashboard Layout

The dashboard should present information in a clean, glanceable format:

#### 1. Activity Heatmap

A GitHub-style contribution grid showing event density over time. Each cell represents one hour, colored by event volume (darker = more activity). The grid should display the last 7 days by default with options to expand to 30 days.

#### 2. Live Event Stream

A scrolling feed of recent events, displaying timestamp, event type (with icon), project name, and context (e.g., file basename or tool category). New events should animate in from the top. The stream should auto-scroll unless the user has scrolled up to review history.

#### 3. Session Overview

Cards showing currently active sessions, each displaying the project name, session duration, and a mini activity indicator (sparkline or dots showing recent event frequency).

#### 4. Statistics Panel

Summary metrics for the selected time period:

- Total events
- Events by type (tool, activity, etc.) — small bar chart
- Most active projects — ranked list
- Tool usage breakdown — pie or donut chart

### Interaction Patterns

- Clicking a session card should filter the event stream to that session
- Clicking a cell in the heatmap should filter to events from that hour
- All filters should be clearable with a single click
- Connection status indicator showing WebSocket health (connected/reconnecting/disconnected)

---

## Non-Functional Requirements

### Performance

- Monitor: < 1% CPU usage when idle, < 5MB memory footprint
- Server: Handle 100+ concurrent WebSocket connections on a single Fly.io shared-cpu-1x instance
- Client: Time to interactive < 2 seconds on 3G connection
- Event latency: < 100ms from Monitor detection to Client display under normal conditions

### Reliability

- Monitor should recover gracefully from Server unavailability
- Server should handle client disconnects without affecting other clients
- Client should automatically reconnect with exponential backoff

### Security

- All connections must use TLS (WSS for WebSockets, HTTPS for REST)
- Bearer token authentication on all endpoints
- No sensitive data transmitted (enforced by Monitor privacy pipeline)
- Rate limiting on event ingestion endpoint to prevent abuse

---

## Out of Scope (v1)

The following features are explicitly out of scope for the initial release:

- Event persistence/history — VibeTea is a real-time broadcast system only
- User accounts or multi-tenancy — single-token authentication is sufficient
- Mobile-specific client — responsive web will serve mobile use cases
- Cursor, Copilot, or other agent monitors — Claude Code only for v1
- Custom event types or schemas from Monitors — fixed schema only
- Analytics, aggregations, or derived metrics — raw events only

---

## Success Criteria

The project will be considered successful when:

1. A developer can install the Monitor, connect to the hosted Server, and see their Claude Code activity appear in the Dashboard within 5 minutes
2. Events appear in the Dashboard within 200ms of occurring in Claude Code
3. The system runs reliably for 24+ hours of continuous use without intervention
4. The Monitor binary is under 10MB and uses minimal system resources

---

## Suggested Implementation Order

1. **Server** — Get the event hub running on Fly.io first. This unblocks everything else.
2. **Monitor** — Build the Claude Code watcher and connect it to the Server.
3. **Client** — Build the Dashboard once there's real event data to display.
