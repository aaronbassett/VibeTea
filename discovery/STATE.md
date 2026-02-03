# Discovery State: vibetea

**Updated**: 2026-02-02 13:00 UTC
**Iteration**: 2
**Phase**: Complete (Ready for Implementation)

---

## Problem Understanding

### Problem Statement
Modern developers increasingly rely on AI coding assistants, often running multiple agents (Claude Code, Cursor, Copilot) across different projects simultaneously. Currently, there is no unified way to monitor AI agent activity across tools, visualize coding patterns and AI utilization over time, build integrations that react to AI assistant events in real-time, or understand how AI assistants are being used across a team. VibeTea solves this by providing a real-time event aggregation and broadcast system that consolidates activity streams into a unified WebSocket feed while maintaining strict privacy controls—transmitting only structural metadata, never code or sensitive content.

### Personas
| Persona | Description | Primary Goals |
|---------|-------------|---------------|
| Solo Developer | Individual developer using AI coding assistants daily | Wants visibility into their AI-assisted workflow, patterns, and utilization |
| Team Lead | Technical lead overseeing multiple developers | Wants to understand how AI assistants are being used across the team |
| Integration Builder | Developer building custom tooling | Wants a reliable event stream to build dashboards, analytics, or automations |

### Current State vs. Desired State
**Today (without feature)**:
- No visibility into AI agent activity across tools
- No way to see patterns or trends in AI utilization
- Building integrations requires coupling to specific visualization clients
- Existing solutions (like PixelHQ-bridge) lack flexibility for custom integrations

**Tomorrow (with feature)**:
- Unified real-time view of all AI coding assistant activity
- Glanceable dashboard showing activity heatmaps, live events, and session status
- Open WebSocket API for custom integrations
- Privacy-preserving: only metadata transmitted, never code or prompts

### Constraints
- **Technical**: Monitor must be Rust (single static binary, <10MB, <5MB memory)
- **Technical**: Server must be Rust with axum/tokio, deployable to Fly.io
- **Technical**: Client must be React/TypeScript SPA, deployable to CDN
- **Privacy**: Strict privacy pipeline—never transmit code, prompts, file contents, full paths
- **Scope (v1)**: Claude Code only (no Cursor/Copilot), no persistence, no multi-tenancy
- **Performance**: <100ms event latency end-to-end, <2s time-to-interactive for client

---

## Story Landscape

### Story Status Overview
| # | Story | Priority | Status | Confidence | Blocked By |
|---|-------|----------|--------|------------|------------|
| 1 | Server: Event Hub Core | P1 | ✅ Complete | 95% | - |
| 2 | Monitor: Claude Code Watcher | P1 | ✅ Complete | 90% | - |
| 3 | Monitor: Privacy Pipeline | P1 | ✅ Complete | 95% | - |
| 4 | Monitor: Server Connection | P1 | ✅ Complete | 95% | - |
| 5 | Client: WebSocket Connection | P2 | ✅ Complete | 90% | - |
| 6 | Client: Live Event Stream | P2 | ✅ Complete | 90% | - |
| 7 | Client: Activity Heatmap | P2 | ✅ Complete | 90% | - |
| 8 | Client: Session Overview | P2 | ✅ Complete | 90% | - |
| 9 | Client: Statistics Panel | P3 | ✅ Complete | 85% | - |

### Story Dependencies
```
[Server Core #1] ←── [Monitor Connection #4] ←── [Monitor Privacy #3] ←── [Monitor Watcher #2]
      │
      └──────────── [Client WebSocket #5] ←── [Client Event Stream #6]
                          │                          │
                          │                          └── [Client Heatmap #7]
                          │                          └── [Client Sessions #8]
                          │                          └── [Client Stats #9]
```

### Proto-Stories / Emerging Themes
*All themes now crystallized into stories. No remaining proto-stories.*

---

## Completed Stories Summary

| # | Story | Priority | Key Decisions | Revision Risk |
|---|-------|----------|---------------|---------------|
| 1 | Server: Event Hub Core | P1 | D1 (rate limits), D2 (Ed25519 auth) | Low |
| 2 | Monitor: Claude Code Watcher | P1 | R1 (JSONL format), R3 (session lifecycle) | Low |
| 3 | Monitor: Privacy Pipeline | P1 | D4 (basename allowlist) | Low |
| 4 | Monitor: Server Connection | P1 | D2 (Ed25519 signing), D3 (reconnection) | Low |
| 5 | Client: WebSocket Connection | P2 | D5 (token UX) | Low |
| 6 | Client: Live Event Stream | P2 | Q9 (auto-scroll: 50px threshold) | Low |
| 7 | Client: Activity Heatmap | P2 | D7 (5 color levels, absolute thresholds) | Low |
| 8 | Client: Session Overview | P2 | D6 (5 min active, 30 min removal) | Low |
| 9 | Client: Statistics Panel | P3 | D8 (preset periods, client-side calc) | Low |

*Full stories in SPEC.md*

---

## In-Progress Story Detail

*All stories complete. See SPEC.md for full specifications.*

---

## Watching List

*Items that might affect graduated stories:*

[Will be populated as graduated stories accumulate]

---

## Glossary

- **Monitor**: Lightweight daemon that watches AI agent activity and forwards normalized events
- **Server**: Central event hub that receives events from Monitors and broadcasts to Clients
- **Client**: Web-based dashboard that visualizes the real-time event stream
- **Event Envelope**: Standard wrapper for all events containing id, source, timestamp, type, payload
- **Privacy Pipeline**: Filter in Monitor that strips sensitive data before transmission
- **JSONL**: JSON Lines format used by Claude Code for session files
- **Ed25519**: Elliptic curve signature algorithm used for Monitor authentication
- **Keypair**: Monitor's private key (signing) and public key (verification)
- **Subscriber Token**: Auth token for Clients to receive events
- **Unsafe Mode**: Development mode that disables all authentication (`VIBETEA_UNSAFE_NO_AUTH=true`)

---

## Next Actions

1. ✅ Research Claude Code JSONL format — **DONE** (see R1)
2. ✅ Define rate limiting parameters — **DONE** (see D1)
3. ✅ Define auth model — **DONE** (see D2, revised to Ed25519)
4. ✅ Define reconnection parameters — **DONE** (see D3)
5. ✅ Resolve Q9 (auto-scroll behavior) — **DONE**
6. ✅ Resolve Q15 (WebSocket library) — **DONE**
7. ✅ Graduate all stories to SPEC.md — **DONE**

**Specification complete. Ready for implementation.**
