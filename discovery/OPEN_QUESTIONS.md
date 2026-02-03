# Open Questions: vibetea

**Updated**: 2026-02-02 13:00 UTC

---

## 🔴 Blocking Questions

*None remaining—all blocking questions resolved.*

---

## 🟡 Clarifying Questions

*None remaining—all clarifying questions resolved.*

---

## 🔵 Research Pending

*All research questions resolved. See `archive/RESEARCH.md` for findings.*

---

## 🟠 Watching

May affect graduated stories later.

*No watching items yet—will be populated as stories graduate.*

---

## Resolved Questions

| ID | Question | Resolution | Date | Stories Updated |
|----|----------|------------|------|-----------------|
| Q1 | Server rate limiting specifics | 100 events/sec per source, 200 burst, 1000 global. 429 on exceed. | 2026-02-02 | Server: Event Hub Core |
| Q2 | Authentication model | Ed25519 signing for Monitors, static token for Clients, `VIBETEA_UNSAFE_NO_AUTH=true` to disable | 2026-02-02 | Server, Monitor, Client |
| Q3 | Claude Code JSONL event format | Documented in R1. Types: system, user, assistant, progress, file-history-snapshot, summary | 2026-02-02 | Monitor: Claude Code Watcher |
| Q4 | Claude Code event types | 6 types documented. Map to VibeTea schema. | 2026-02-02 | Monitor: Claude Code Watcher |
| Q5 | Claude Code session file lifecycle | New file per session, events appended, summary event on end | 2026-02-02 | Monitor: Claude Code Watcher |
| Q6 | Sensitive filename handling | Transmit as-is by default; optional VIBETEA_BASENAME_ALLOWLIST env var | 2026-02-02 | Monitor: Privacy Pipeline |
| Q7 | Monitor reconnection behavior | 1s initial, 60s max, 2x backoff, ±25% jitter, unlimited retries, FIFO eviction | 2026-02-02 | Monitor: Server Connection |
| Q8 | Client authentication UX | URL query param + localStorage persistence + form fallback | 2026-02-02 | Client: WebSocket Connection |
| Q10 | Heatmap time granularity | 1 hour cells, user's local timezone, 7/30 day views | 2026-02-02 | Client: Activity Heatmap |
| Q11 | Heatmap color scale | 5 levels with absolute thresholds (0, 1-10, 11-25, 26-50, 51+) | 2026-02-02 | Client: Activity Heatmap |
| Q12 | Session "active" definition | Active if event in last 5 min OR no summary event. Remove from display after 30 min inactive. | 2026-02-02 | Client: Session Overview |
| Q13 | Statistics time periods | Preset periods (1h, 24h, 7d, 30d), client-side calculation with disclaimer | 2026-02-02 | Client: Statistics Panel |
| Q14 | Claude Code file location variations | `~/.claude/projects/<slugified-path>/<uuid>.jsonl` confirmed | 2026-02-02 | Monitor: Claude Code Watcher |
| Q9 | Event stream auto-scroll behavior | Disable at 50px from bottom, resume on scroll back or "Jump to latest" button, no timeout | 2026-02-02 | Client: Live Event Stream |
| Q15 | WebSocket library selection | Use axum's built-in WebSocket support (well-integrated, sufficient) | 2026-02-02 | Server: Event Hub Core |
