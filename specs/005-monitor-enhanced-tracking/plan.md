# Implementation Plan: Monitor Enhanced Data Tracking

**Branch**: `005-monitor-enhanced-tracking` | **Date**: 2026-02-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-monitor-enhanced-tracking/spec.md`

## Summary

Extend the VibeTea monitor to track additional data sources from Claude Code: agent spawns from Task tool invocations, skill invocations from history.jsonl, token usage/session metrics/activity patterns/model distribution from stats-cache.json, todo progress with abandonment tracking, file edit history from file-history directory, and project activity from the projects directory structure. All tracking is per-session with global aggregates where appropriate.

## Technical Context

**Language/Version**: Rust 2021 edition (Monitor), TypeScript 5.9.3 (Client)
**Primary Dependencies**:
- Monitor: tokio 1.43, notify 8.0, serde, ed25519-dalek, reqwest, chrono, uuid
- Client: React 19, Zustand, @tanstack/react-virtual
**Storage**: N/A (in-memory event processing, no persistence)
**Testing**: cargo test with --test-threads=1 for env var isolation, Vitest for TypeScript
**Target Platform**: Linux/macOS/Windows for Monitor, Browser for Client
**Project Type**: Multi-component distributed system (Monitor/Server/Client)
**Performance Goals**:
- Stats-cache.json parsing < 100ms (NFR-002)
- Event emission within 1-2 seconds of source file change (SC-001, SC-002)
- File diff operations async to avoid blocking (NFR-003)
**Constraints**:
- Privacy-first: Never transmit code/prompts (NFR-004)
- Max 1000 tracked sessions (NFR-006)
- 200ms debounce on file changes (NFR-005)
**Scale/Scope**:
- Multi-file watching (~5 additional sources beyond current JSONL)
- Per-model token aggregation
- Per-session todo tracking

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| **I. Privacy by Design** | ✅ PASS | Events contain only metadata (tool names, counts, timestamps). No code content transmitted. FR-031 requires privacy compliance. |
| **II. Unix Philosophy** | ✅ PASS | Monitor watches files and emits events. Server broadcasts. Client displays. Clear separation maintained. |
| **III. Keep It Simple** | ✅ PASS | Using existing patterns (file watcher, event types). No new frameworks. |
| **IV. Event-Driven** | ✅ PASS | All new tracking uses file watch events → parsed events → broadcast. |
| **V. Test What Matters** | ✅ PASS | Integration tests for file watching, unit tests for parsers. Privacy tests required. |
| **VI. Fail Fast & Loud** | ✅ PASS | NFR-007 requires graceful JSON parse failure handling with retry. |
| **VII. Modularity** | ✅ PASS | New parsers as separate modules. Event types extend existing enum. |

## Project Structure

### Documentation (this feature)

```text
specs/005-monitor-enhanced-tracking/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (event schemas)
└── tasks.md             # Phase 2 output (/sdd:tasks command)
```

### Source Code (repository root)

```text
monitor/src/
├── main.rs                    # Entry point, CLI commands
├── lib.rs                     # Module exports
├── config.rs                  # Environment configuration (existing)
├── watcher.rs                 # File system watcher (existing, extend)
├── parser.rs                  # JSONL parser (existing, for session JSONL)
├── privacy.rs                 # Privacy pipeline (existing)
├── crypto.rs                  # Ed25519 signing (existing)
├── sender.rs                  # HTTP sender (existing)
├── types.rs                   # Event types (extend with new events)
├── error.rs                   # Error types (extend)
├── trackers/                  # NEW: Dedicated tracking modules
│   ├── mod.rs                 # Tracker module exports
│   ├── agent_tracker.rs       # FR-001-003: Task tool parsing from session JSONL
│   ├── skill_tracker.rs       # FR-004-006: history.jsonl watching
│   ├── stats_tracker.rs       # FR-007-015: stats-cache.json watching
│   ├── todo_tracker.rs        # FR-016-021: todos/*.json watching
│   ├── file_history_tracker.rs # FR-022-027: file-history diff tracking
│   └── project_tracker.rs     # FR-028-030: projects/ activity tracking
└── tests/
    └── enhanced_tracking_test.rs  # Integration tests

server/src/
├── types.rs                   # Event types (extend to match monitor)
└── (other files unchanged)

client/src/
├── types/
│   └── events.ts              # Event types (extend to match)
├── components/
│   └── (potential new components for new data display)
└── hooks/
    └── useEventStore.ts       # Store updates for new event types
```

**Structure Decision**: Extend existing monitor module with new `trackers/` subdirectory for dedicated tracking modules. This follows the single-responsibility principle while keeping related tracking logic together.

## Complexity Tracking

No constitution violations requiring justification. The feature extends existing patterns:
- File watching uses existing `notify` infrastructure
- Event types extend existing `EventPayload` enum
- Privacy filtering follows established pipeline

## Learnings from Previous Retros

### From Phase 8-10 Retros (Most Relevant)

| Learning | Application to This Feature |
|----------|---------------------------|
| **Pure render behavior** (P10) | Stats aggregation should use event timestamps as reference, not Date.now() |
| **Memoization with useMemo** (P9) | Client-side aggregation of token usage should be memoized |
| **Zustand selective subscriptions** (P8) | New event types should have dedicated selectors |
| **TypeScript type assertions** (P8) | Event payload switching needs explicit type casts |
| **Debouncing** (P8) | 200ms debounce on stats-cache.json (NFR-005) |
| **Graceful JSON parse failures** (P8) | Retry with 100ms delay on parse error (edge case spec) |

### Key Patterns to Reuse

1. **File watcher position tracking** - Already in watcher.rs for JSONL tailing
2. **Privacy pipeline** - All new events route through PrivacyPipeline
3. **Event buffering** - Sender already handles buffering and retry
4. **Event type icons** - Unicode escape sequences for client display

---

## Phase 0: Research

### Research Questions

1. **stats-cache.json structure**: What is the exact JSON schema? What fields contain per-model token counts, session metrics, hourCounts, and modelUsage?

2. **history.jsonl format**: What fields are available? How to extract sessionId and project from each entry?

3. **todo file naming**: Confirm the pattern `<session-uuid>-agent-<session-uuid>.json` and content structure.

4. **file-history versioning**: Confirm the `<hash>@vN` naming pattern. What diff library to use in Rust?

5. **Session JSONL Task tool**: What does a Task tool_use event look like? How is subagent_type encoded in the parameters?

6. **projects/ directory structure**: How to determine if a session is active (no summary event)?

7. **inotify limits**: What are typical max_user_watches limits? How to detect approaching limits?

### Research Approach

Use parallel exploration agents to investigate Claude Code's ~/.claude directory structure:
- Examine actual stats-cache.json, history.jsonl, todos/*.json files
- Analyze file-history/ naming patterns
- Parse session JSONL for Task tool invocations

**Output**: research.md with all questions resolved

---

## Phase 1: Design & Contracts

### Prerequisites
- Phase 0 research.md complete
- All NEEDS CLARIFICATION items resolved

### Deliverables

1. **data-model.md**: Entity definitions for all new event types
   - AgentSpawnEvent
   - SkillInvocationEvent
   - TokenUsageEvent
   - SessionMetricsEvent
   - ActivityPatternEvent
   - ModelDistributionEvent
   - TodoProgressEvent
   - FileChangeEvent
   - ProjectActivityEvent

2. **contracts/**: JSON schemas for new event types (OpenAPI format)

3. **quickstart.md**: Setup instructions for testing enhanced tracking

4. **Agent context update**: Update CLAUDE.md with new environment variables and commands

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Separate tracker modules** | Single responsibility, easier testing |
| **Global aggregates for stats-cache** | stats-cache.json is already global; no per-session breakdown available |
| **Per-session for todos/file-history** | Filenames contain session IDs |
| **Debounced file watching** | stats-cache.json updates frequently; avoid overwhelming event stream |
| **Async diff operations** | Prevent blocking event loop (NFR-003) |

---

## Phase 2: Local Development Environment

### Prerequisites
- Tech stack determined from STACK.md

### Validation Checklist

- [ ] `cargo clippy` passes on monitor crate
- [ ] `cargo fmt --check` passes
- [ ] `cargo test -p vibetea-monitor --test-threads=1` passes
- [ ] Existing test suite still passes after structure changes
- [ ] Privacy tests continue to verify Constitution I compliance

### Development Setup

1. **Test files**: Create mock versions of:
   - `~/.claude/stats-cache.json`
   - `~/.claude/history.jsonl`
   - `~/.claude/todos/*.json`
   - `~/.claude/file-history/<session>/<hash>@vN`

2. **Environment variables** (new):
   - No new required env vars (uses existing VIBETEA_CLAUDE_DIR)

3. **Justfile updates**: Add commands for running tracker-specific tests

---

## Stop Point

This plan covers Phases 0-2. Implementation continues with `/sdd:tasks` after Phase 1 design artifacts are complete.

**Generated Artifacts**:
- ✅ plan.md (this file)
- ✅ research.md (Phase 0) - All 7 research questions resolved
- ✅ data-model.md (Phase 1) - 9 event type definitions
- ✅ contracts/enhanced-events.json (Phase 1) - JSON Schema for all events
- ✅ quickstart.md (Phase 1) - Development setup and test data

**Branch**: `005-monitor-enhanced-tracking`
**Spec**: `/specs/005-monitor-enhanced-tracking/spec.md`

---

## Phase 0 Research Summary

All research questions have been resolved. Key findings:

| Question | Finding |
|----------|---------|
| stats-cache.json | Contains `modelUsage` map with per-model token counts, `hourCounts` for activity patterns, global metrics |
| history.jsonl | Format: `{display, timestamp, sessionId, project}` - skills start with `/` |
| todo file naming | Pattern: `{session-uuid}-agent-{session-uuid}.json` |
| file-history versioning | Pattern: `{16-char-hex}@vN` with sequential versions |
| Task tool structure | `tool_use.input.subagent_type` contains agent type |
| Session activity | Active if last line is NOT `{"type": "summary"}` |
| inotify limits | 495440 watches available; ~100 needed |

See [research.md](./research.md) for full details.

---

## Phase 1 Design Summary

### Event Types Added

1. **AgentSpawnEvent** - Task tool agent spawns
2. **SkillInvocationEvent** - Skill/command invocations
3. **TokenUsageEvent** - Per-model token consumption
4. **SessionMetricsEvent** - Global session metrics
5. **ActivityPatternEvent** - Hourly activity distribution
6. **ModelDistributionEvent** - Model usage breakdown
7. **TodoProgressEvent** - Todo list progress per session
8. **FileChangeEvent** - File edit statistics
9. **ProjectActivityEvent** - Project session activity

### Privacy Compliance

All events verified to contain only metadata:
- UUIDs, timestamps, counts
- Tool/skill names (not content)
- File hashes (not paths or content)
- Model identifiers (public info)

See [data-model.md](./data-model.md) for full entity definitions.

---

## Next Steps

Run `/sdd:tasks` to generate implementation tasks based on this plan.
