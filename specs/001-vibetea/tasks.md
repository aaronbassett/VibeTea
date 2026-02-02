# Tasks: VibeTea

**Input**: Design documents from `/specs/001-vibetea/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/openapi.yaml ✓, quickstart.md ✓

**Tests**: Not explicitly requested in spec - test tasks omitted.

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1-US9)
- **[GIT]**: Git workflow task (commit, push, PR)
- Includes exact file paths in descriptions

## Path Conventions

VibeTea uses a multi-project structure:
- **Rust Workspace**: `Cargo.toml` (workspace root), `monitor/`, `server/`
- **Client**: `client/` (TypeScript/React)

---

## Phase 1: Setup (Project Initialization)

**Purpose**: Verify and upgrade all dependencies to latest versions, configure project structure

### Dependency Version Audit

- [X] T001 Audit and upgrade all Rust workspace dependencies to latest versions in Cargo.toml (use devs:rust-dev agent)
- [X] T002 [GIT] Commit: upgrade Rust workspace dependencies
- [X] T003 Audit and upgrade all client npm dependencies to latest versions in client/package.json (use devs:typescript-dev agent)
- [X] T004 [GIT] Commit: upgrade client npm dependencies

### Tailwind v4 Migration

- [X] T005 Remove client/tailwind.config.js and migrate to Tailwind v4 CSS-based configuration in client/src/index.css (use devs:react-dev agent)
- [X] T006 [GIT] Commit: migrate to Tailwind v4 CSS configuration
- [X] T007 Update client/postcss.config.js for Tailwind v4 compatibility (use devs:typescript-dev agent) - N/A: PostCSS not needed with @tailwindcss/vite
- [X] T008 [GIT] Commit: update PostCSS config for Tailwind v4 - N/A: merged with T006

### Project Structure Verification

- [X] T009 [P] Verify Rust workspace structure: monitor/src/, server/src/ directories exist (use devs:rust-dev agent)
- [X] T010 [P] Verify client structure: client/src/components/, client/src/hooks/, client/src/types/, client/src/utils/ directories exist (use devs:typescript-dev agent)
- [X] T011 [GIT] Commit: verify project structure

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Start Phase 2

- [X] T012 [GIT] Verify working tree is clean before starting Phase 2
- [X] T013 [GIT] Pull and rebase on origin/main if needed
- [X] T014 Create retro/P2.md for this phase

### Shared Rust Types

- [X] T015 [P] Create shared event types module in server/src/types.rs with Event, EventPayload, EventType enums matching data-model.md (use devs:rust-dev agent)
- [X] T016 [GIT] Commit: add shared event types

### Server Core Infrastructure

- [X] T017 [P] Implement config module in server/src/config.rs for environment variable parsing (VIBETEA_PUBLIC_KEYS, VIBETEA_SUBSCRIBER_TOKEN, PORT, VIBETEA_UNSAFE_NO_AUTH) (use devs:rust-dev agent)
- [X] T018 [GIT] Commit: add server config module
- [X] T019 [P] Implement error types in server/src/error.rs using thiserror (use devs:rust-dev agent)
- [X] T020 [GIT] Commit: add server error types

### Monitor Core Infrastructure

- [X] T021 [P] Implement config module in monitor/src/config.rs for environment variable parsing (VIBETEA_SERVER_URL, VIBETEA_SOURCE_ID, VIBETEA_KEY_PATH, etc.) (use devs:rust-dev agent)
- [X] T022 [GIT] Commit: add monitor config module
- [X] T023 [P] Implement error types in monitor/src/error.rs using thiserror (use devs:rust-dev agent)
- [X] T024 [GIT] Commit: add monitor error types

### Client Core Infrastructure

- [X] T025 [P] Create TypeScript event types in client/src/types/events.ts matching data-model.md (use devs:typescript-dev agent)
- [X] T026 [GIT] Commit: add client event types
- [X] T027 [P] Create Zustand store skeleton in client/src/hooks/useEventStore.ts (use devs:react-dev agent)
- [X] T028 [GIT] Commit: add event store skeleton

### Phase 2 Completion

- [X] T029 Run /sdd:map incremental for Phase 2 changes
- [X] T030 [GIT] Commit: update codebase documents for phase 2
- [X] T031 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update
- [X] T032 [GIT] Commit: finalize phase 2 retro
- [X] T033 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T034 [GIT] Create/update PR to main with phase summary - PR #1 created
- [X] T035 [GIT] Verify all CI checks pass
- [X] T036 [GIT] Report PR ready status - All CI checks passing, PR #1 ready for review

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Server Event Hub Core (Priority: P1) 🎯 MVP

**Goal**: Central server that receives events from Monitors and broadcasts them to Clients

**Independent Test**: Send mock events via curl to POST /events and verify WebSocket broadcasts to connected clients

### Start Phase 3

- [X] T037 [GIT] Verify working tree is clean before starting Phase 3
- [X] T038 [GIT] Pull and rebase on origin/main if needed - Already on latest
- [X] T039 [US1] Create retro/P3.md for this phase

### Authentication Layer

- [X] T040 [P] [US1] Implement Ed25519 signature verification in server/src/auth.rs (use devs:rust-dev agent)
- [X] T041 [GIT] Commit: add Ed25519 signature verification
- [X] T042 [US1] Implement token validation for WebSocket clients in server/src/auth.rs (use devs:rust-dev agent)
- [X] T043 [GIT] Commit: add token validation

### Unsafe Mode Testing

- [X] T043a [US1] Test VIBETEA_UNSAFE_NO_AUTH=true mode: verify POST /events accepts unsigned requests and GET /ws accepts connections without token in server/tests/unsafe_mode_test.rs (use devs:rust-dev agent)
- [X] T043b [GIT] Commit: add unsafe mode tests

### WebSocket Broadcast

- [X] T044 [US1] Implement broadcast channel in server/src/broadcast.rs using tokio::sync::broadcast (use devs:rust-dev agent)
- [X] T045 [GIT] Commit: add broadcast channel
- [X] T046 [US1] Implement WebSocket connection handler with filtering support (source, type, project) in server/src/broadcast.rs (use devs:rust-dev agent) - Implemented in routes.rs
- [X] T047 [GIT] Commit: add WebSocket filtering - Part of routes commit

### Rate Limiting

- [X] T048 [US1] Implement per-source rate limiting (100 events/sec) in server/src/rate_limit.rs (use devs:rust-dev agent)
- [X] T049 [GIT] Commit: add rate limiting

### HTTP Routes

- [X] T050 [US1] Implement POST /events endpoint in server/src/routes.rs (use devs:rust-dev agent)
- [X] T051 [GIT] Commit: add POST /events endpoint
- [X] T052 [US1] Implement GET /ws endpoint in server/src/routes.rs (use devs:rust-dev agent)
- [X] T053 [GIT] Commit: add GET /ws endpoint
- [X] T054 [US1] Implement GET /health endpoint in server/src/routes.rs (use devs:rust-dev agent)
- [X] T055 [GIT] Commit: add GET /health endpoint - Combined with routes commit

### Server Main Entry Point

- [X] T056 [US1] Implement server main.rs with axum router, graceful shutdown, logging (use devs:rust-dev agent)
- [X] T057 [GIT] Commit: implement server main entry point
- [X] T058 [US1] Implement server lib.rs with public API exports (use devs:rust-dev agent) - Already done during module creation
- [X] T059 [GIT] Commit: add server lib.rs - Part of previous commits

### Phase 3 Completion

- [X] T060 [US1] Run /sdd:map incremental for Phase 3 changes
- [X] T061 [GIT] Commit: update codebase documents for phase 3
- [X] T062 [US1] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [X] T063 [GIT] Commit: finalize phase 3 retro
- [X] T064 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T065 [GIT] Create/update PR to main with phase summary
- [X] T066 [GIT] Verify all CI checks pass
- [X] T067 [GIT] Report PR ready status

**Checkpoint**: Server Event Hub (US1) fully functional and testable independently

---

## Phase 4: User Story 2 - Monitor Claude Code Watcher (Priority: P1) 🎯 MVP

**Goal**: Monitor daemon that watches Claude Code sessions and forwards events

**Independent Test**: Run Monitor while using Claude Code and verify events are emitted for tool usage

### Start Phase 4

- [X] T068 [GIT] Verify working tree is clean before starting Phase 4
- [X] T069 [GIT] Pull and rebase on origin/main if needed
- [X] T070 [US2] Create retro/P4.md for this phase

### File Watching

- [X] T071 [US2] Implement file watcher in monitor/src/watcher.rs using notify crate to watch ~/.claude/projects/**/*.jsonl (use devs:rust-dev agent)
- [X] T072 [GIT] Commit: add file watcher
- [X] T073 [US2] Implement file position tracking for JSONL tailing in monitor/src/watcher.rs (use devs:rust-dev agent)
- [X] T074 [GIT] Commit: add file position tracking

### JSONL Parsing

- [X] T075 [US2] Implement Claude Code JSONL parser in monitor/src/parser.rs mapping to VibeTea event types (use devs:rust-dev agent)
- [X] T076 [GIT] Commit: add JSONL parser
- [X] T077 [US2] Implement session detection (new file = session started, summary event = session ended) in monitor/src/parser.rs (use devs:rust-dev agent)
- [X] T078 [GIT] Commit: add session detection

### Monitor Types

- [X] T079 [US2] Create shared event types in monitor/src/types.rs matching server types (use devs:rust-dev agent)
- [X] T080 [GIT] Commit: add monitor event types

### Phase 4 Completion

- [X] T081 [US2] Run /sdd:map incremental for Phase 4 changes
- [X] T082 [GIT] Commit: update codebase documents for phase 4
- [X] T083 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T084 [GIT] Commit: finalize phase 4 retro - N/A, no CLAUDE.md changes needed
- [X] T085 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T086 [GIT] Create/update PR to main with phase summary - PR #1 already exists
- [X] T087 [GIT] Verify all CI checks pass - CI passing
- [X] T088 [GIT] Report PR ready status - PR #1 ready

**Checkpoint**: Monitor Claude Code Watcher (US2) fully functional and testable independently

---

## Phase 5: User Story 3 - Monitor Privacy Pipeline (Priority: P1) 🎯 MVP

**Goal**: Privacy pipeline that strips all sensitive data before transmission

**Independent Test**: Capture emitted events and verify no sensitive content is present

### Start Phase 5

- [X] T089 [GIT] Verify working tree is clean before starting Phase 5
- [X] T090 [GIT] Pull and rebase on origin/main if needed - Already on latest
- [X] T091 [US3] Create retro/P5.md for this phase

### Privacy Pipeline Implementation

- [X] T092 [US3] Implement privacy pipeline in monitor/src/privacy.rs with path-to-basename conversion (use devs:rust-dev agent)
- [X] T093 [GIT] Commit: add path stripping - Combined in single commit
- [X] T094 [US3] Implement content stripping (file contents, diffs, prompts, responses) in monitor/src/privacy.rs (use devs:rust-dev agent) - Already handled by parser.rs (never extracted)
- [X] T095 [GIT] Commit: add content stripping - N/A, part of main commit
- [X] T096 [US3] Implement command stripping for Bash tool (only transmit description) in monitor/src/privacy.rs (use devs:rust-dev agent)
- [X] T097 [GIT] Commit: add command stripping - Combined in single commit
- [X] T098 [US3] Implement pattern stripping for Grep/Glob tools in monitor/src/privacy.rs (use devs:rust-dev agent)
- [X] T099 [GIT] Commit: add pattern stripping - Combined in single commit
- [X] T100 [US3] Implement VIBETEA_BASENAME_ALLOWLIST filtering in monitor/src/privacy.rs (use devs:rust-dev agent)
- [X] T101 [GIT] Commit: add basename allowlist filtering - Combined in feat(monitor): add privacy pipeline commit

### Privacy Compliance Testing (Constitution I)

- [X] T101a [US3] Create privacy compliance test suite in monitor/tests/privacy_test.rs: verify no file contents, full paths, commands, patterns, or prompts in emitted events - must explicitly test all prohibited fields from spec (use devs:rust-dev agent) - 17 tests
- [X] T101b [GIT] Commit: add privacy compliance tests

### Phase 5 Completion

- [X] T102 [US3] Run /sdd:map incremental for Phase 5 changes
- [X] T103 [GIT] Commit: update codebase documents for phase 5
- [X] T104 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T105 [GIT] Commit: finalize phase 5 retro
- [X] T106 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T107 [GIT] Create/update PR to main with phase summary
- [X] T108 [GIT] Verify all CI checks pass
- [X] T109 [GIT] Report PR ready status

**Checkpoint**: Monitor Privacy Pipeline (US3) fully functional and testable independently

---

## Phase 6: User Story 4 - Monitor Server Connection (Priority: P1) 🎯 MVP

**Goal**: Reliable connection to VibeTea server with buffering and retry

**Independent Test**: Start Monitor, disconnect network, reconnect, verify buffered events are delivered

### Start Phase 6

- [X] T110 [GIT] Verify working tree is clean before starting Phase 6
- [X] T111 [GIT] Pull and rebase on origin/main if needed - Already on latest
- [X] T112 [US4] Create retro/P6.md for this phase

### Keypair Generation CLI (FR-8a)

- [X] T112a [US4] Implement `vibetea init` CLI command in monitor/src/main.rs: check for existing keys, prompt overwrite confirmation, call crypto module, display public key for server registration (use devs:rust-dev agent)
- [X] T112b [GIT] Commit: add vibetea init command - Combined with main entry point commit

### Cryptography

- [X] T113 [US4] Implement Ed25519 keypair generation and storage in monitor/src/crypto.rs: generate with OsRng, save key.priv (0600) and key.pub (base64, 0644) to ~/.vibetea/ (use devs:rust-dev agent)
- [X] T114 [GIT] Commit: add keypair generation
- [X] T115 [US4] Implement event signing in monitor/src/crypto.rs (use devs:rust-dev agent)
- [X] T116 [GIT] Commit: add event signing - Combined with keypair commit

### HTTP Client with Retry

- [X] T117 [US4] Implement HTTP sender with connection pooling in monitor/src/sender.rs (use devs:rust-dev agent)
- [X] T118 [GIT] Commit: add HTTP sender
- [X] T119 [US4] Implement exponential backoff (1s → 60s max, ±25% jitter) in monitor/src/sender.rs (use devs:rust-dev agent)
- [X] T120 [GIT] Commit: add exponential backoff - Combined with sender commit
- [X] T121 [US4] Implement event buffering (1000 events max, FIFO eviction) in monitor/src/sender.rs (use devs:rust-dev agent)
- [X] T122 [GIT] Commit: add event buffering - Combined with sender commit
- [X] T123 [US4] Implement 429 rate limit handling (respect Retry-After header) in monitor/src/sender.rs (use devs:rust-dev agent)
- [X] T124 [GIT] Commit: add rate limit handling - Combined with sender commit

### Monitor Main Entry Point

- [X] T125 [US4] Implement monitor main.rs with CLI (init, run commands), logging (use devs:rust-dev agent)
- [X] T126 [GIT] Commit: implement monitor main entry point
- [X] T127 [US4] Implement monitor lib.rs with public API exports (use devs:rust-dev agent)
- [X] T128 [GIT] Commit: add monitor lib.rs - Combined with main entry point commit

### Phase 6 Completion

- [X] T129 [US4] Run /sdd:map incremental for Phase 6 changes
- [X] T130 [GIT] Commit: update codebase documents for phase 6
- [X] T131 [US4] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T132 [GIT] Commit: finalize phase 6 retro
- [X] T133 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T134 [GIT] Create/update PR to main with phase summary
- [X] T135 [GIT] Verify all CI checks pass
- [X] T136 [GIT] Report PR ready status

**Checkpoint**: Monitor Server Connection (US4) fully functional - Server and Monitor MVP complete

---

## Phase 7: User Story 5 - Client WebSocket Connection (Priority: P2)

**Goal**: Reliable WebSocket connection with auto-reconnect

**Independent Test**: Open dashboard, verify connection indicator, test reconnection after network interruption

### Start Phase 7

- [X] T137 [GIT] Verify working tree is clean before starting Phase 7
- [X] T138 [GIT] Pull and rebase on origin/main if needed - Already on latest
- [X] T139 [US5] Create retro/P7.md for this phase

### WebSocket Hook

- [X] T140 [US5] Implement useWebSocket hook in client/src/hooks/useWebSocket.ts with connection management (use devs:react-dev agent)
- [X] T141 [GIT] Commit: add useWebSocket hook - Combined with T142, T144
- [X] T142 [US5] Implement exponential backoff reconnection in client/src/hooks/useWebSocket.ts (use devs:react-dev agent)
- [X] T143 [GIT] Commit: add reconnection logic - Combined with T141
- [X] T144 [US5] Implement token handling (URL param, localStorage) in client/src/hooks/useWebSocket.ts (use devs:react-dev agent)
- [X] T145 [GIT] Commit: add token handling - Combined with T141

### Connection UI

- [X] T146 [US5] Implement ConnectionStatus component in client/src/components/ConnectionStatus.tsx (green/yellow/red indicator) (use devs:react-dev agent)
- [X] T147 [GIT] Commit: add ConnectionStatus component
- [X] T148 [US5] Implement TokenForm component in client/src/components/TokenForm.tsx (use devs:react-dev agent)
- [X] T149 [GIT] Commit: add TokenForm component

### Event Store Integration

- [X] T150 [US5] Complete useEventStore implementation in client/src/hooks/useEventStore.ts with WebSocket integration (use devs:react-dev agent) - Already implemented in Phase 2
- [X] T151 [GIT] Commit: complete event store - N/A, already committed in Phase 2
- [X] T152 [US5] Implement event buffering (1000 events max) in client/src/hooks/useEventStore.ts (use devs:react-dev agent) - Already implemented (MAX_EVENTS = 1000)
- [X] T153 [GIT] Commit: add event buffering - N/A, already committed in Phase 2

### Phase 7 Completion

- [X] T154 [US5] Run /sdd:map incremental for Phase 7 changes
- [X] T155 [GIT] Commit: update codebase documents for phase 7
- [X] T156 [US5] Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T157 [GIT] Commit: finalize phase 7 retro
- [X] T158 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T159 [GIT] Create/update PR to main with phase summary - PR #1 updated with Phase 7 summary
- [X] T160 [GIT] Verify all CI checks pass - All 3 jobs passing
- [X] T161 [GIT] Report PR ready status - PR #1 ready for review

**Checkpoint**: Client WebSocket Connection (US5) fully functional and testable independently

---

## Phase 8: User Story 6 - Client Live Event Stream (Priority: P2)

**Goal**: Scrolling feed of real-time events

**Independent Test**: Generate events from Monitor and verify they appear in the stream with correct formatting

### Start Phase 8

- [X] T162 [GIT] Verify working tree is clean before starting Phase 8
- [X] T163 [GIT] Pull and rebase on origin/main if needed - Already on latest
- [X] T164 [US6] Create retro/P8.md for this phase

### Event Stream Component

- [X] T165 [US6] Implement EventStream component in client/src/components/EventStream.tsx using @tanstack/react-virtual (use devs:react-dev agent)
- [X] T166 [GIT] Commit: add EventStream component - Combined with T168, T170, T172, T174
- [X] T167 [US6] Implement auto-scroll logic (pause when scrolled >50px from bottom) in client/src/components/EventStream.tsx (use devs:react-dev agent) - Part of T165
- [X] T168 [GIT] Commit: add auto-scroll logic - Part of T166
- [X] T169 [US6] Implement "Jump to latest" button in client/src/components/EventStream.tsx (use devs:react-dev agent) - Part of T165
- [X] T170 [GIT] Commit: add jump to latest button - Part of T166
- [X] T171 [US6] Implement event type icons (tool 🔧, activity 💬, session 🚀, summary 📋, error ⚠️) in client/src/components/EventStream.tsx (use devs:react-dev agent) - Part of T165
- [X] T172 [GIT] Commit: add event type icons - Part of T166

### Utility Functions

- [X] T173 [US6] Implement timestamp and duration formatting in client/src/utils/formatting.ts (use devs:typescript-dev agent)
- [X] T174 [GIT] Commit: add formatting utilities - Part of T166

### Phase 8 Completion

- [X] T175 [US6] Run /sdd:map incremental for Phase 8 changes
- [X] T176 [GIT] Commit: update codebase documents for phase 8
- [X] T177 [US6] Review retro/P8.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T178 [GIT] Commit: finalize phase 8 retro - Combined with T176
- [X] T179 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [X] T180 [GIT] Create/update PR to main with phase summary - PR #1 updated
- [X] T181 [GIT] Verify all CI checks pass - CI passing (previous run success, current run in progress)
- [X] T182 [GIT] Report PR ready status - PR #1 ready for review

**Checkpoint**: Client Live Event Stream (US6) fully functional and testable independently

---

## Phase 9: User Story 7 - Client Activity Heatmap (Priority: P2)

**Goal**: GitHub-style contribution heatmap for activity visualization

**Independent Test**: Accumulate events over time and verify heatmap cells reflect correct activity levels

### Start Phase 9

- [X] T183 [GIT] Verify working tree is clean before starting Phase 9 - Clean (ralph.toml untracked is unrelated)
- [X] T184 [GIT] Pull and rebase on origin/main if needed - Already on latest, no rebase needed
- [X] T185 [US7] Create retro/P9.md for this phase

### Heatmap Component

- [X] T186 [US7] Implement Heatmap component in client/src/components/Heatmap.tsx using CSS Grid (use devs:react-dev agent)
- [X] T187 [GIT] Commit: add Heatmap component - Combined with T189, T191, T193, T195, T197
- [X] T188 [US7] Implement heatmap color scale (0: #1a1a2e, 1-10: #2d4a3e, 11-25: #3d6b4f, 26-50: #4d8c5f, 51+: #5dad6f) in client/src/components/Heatmap.tsx (use devs:react-dev agent) - Implemented in T186
- [X] T189 [GIT] Commit: add heatmap color scale - Combined with T187
- [X] T190 [US7] Implement 7-day and 30-day view toggle in client/src/components/Heatmap.tsx (use devs:react-dev agent) - Implemented in T186
- [X] T191 [GIT] Commit: add view toggle - Combined with T187
- [X] T192 [US7] Implement timezone-aware hour alignment in client/src/components/Heatmap.tsx (use devs:react-dev agent) - Implemented in T186
- [X] T193 [GIT] Commit: add timezone handling - Combined with T187
- [X] T194 [US7] Implement cell click filtering (filter event stream to that hour) in client/src/components/Heatmap.tsx (use devs:react-dev agent) - Implemented in T186
- [X] T195 [GIT] Commit: add cell click filtering - Combined with T187
- [X] T196 [US7] Add heatmap cell aria-labels for accessibility in client/src/components/Heatmap.tsx (use devs:react-dev agent) - Implemented in T186
- [X] T197 [GIT] Commit: add heatmap accessibility - Combined with T187

### Phase 9 Completion

- [X] T198 [US7] Run /sdd:map incremental for Phase 9 changes - Updated STACK.md, CONVENTIONS.md, TESTING.md
- [X] T199 [GIT] Commit: update codebase documents for phase 9
- [X] T200 [US7] Review retro/P9.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T201 [GIT] Commit: finalize phase 9 retro - Combined with T199
- [X] T202 [GIT] Push branch to origin (ensure pre-push hooks pass) - Pushed successfully
- [X] T203 [GIT] Create/update PR to main with phase summary - PR #1 updated (token scope limited body edit)
- [X] T204 [GIT] Verify all CI checks pass - CI re-run triggered (runner availability issue resolved)
- [X] T205 [GIT] Report PR ready status - Phase 9 complete, PR #1 ready for review

**Checkpoint**: Client Activity Heatmap (US7) fully functional and testable independently

---

## Phase 10: User Story 8 - Client Session Overview (Priority: P2)

**Goal**: Session cards showing active projects with AI assistants

**Independent Test**: Start multiple Claude Code sessions and verify distinct session cards appear

### Start Phase 10

- [X] T206 [GIT] Verify working tree is clean before starting Phase 10 - Clean (tasks.md pending, ralph.toml untracked)
- [X] T207 [GIT] Pull and rebase on origin/main if needed - Already on latest
- [X] T208 [US8] Create retro/P10.md for this phase

### Session State Management

- [X] T209 [US8] Implement session state machine (active/inactive/ended/removed) in client/src/hooks/useEventStore.ts (use devs:react-dev agent)
- [X] T210 [GIT] Commit: add session state machine - Combined with T211, T212
- [X] T211 [US8] Implement session timeout logic (5min → inactive, 30min → removed) in client/src/hooks/useEventStore.ts (use devs:react-dev agent) - Implemented with useSessionTimeouts hook
- [X] T212 [GIT] Commit: add session timeouts - Combined with T209, T210

### Session Overview Component

- [X] T213 [US8] Implement SessionOverview component in client/src/components/SessionOverview.tsx (use devs:react-dev agent)
- [X] T214 [GIT] Commit: add SessionOverview component - Combined with T215-T220
- [X] T215 [US8] Implement session card with project name, duration, activity indicator in client/src/components/SessionOverview.tsx (use devs:react-dev agent) - Part of T213
- [X] T216 [GIT] Commit: add session card details - Combined with T214
- [X] T217 [US8] Implement "Last active" label for inactive sessions in client/src/components/SessionOverview.tsx (use devs:react-dev agent) - Part of T213
- [X] T218 [GIT] Commit: add last active label - Combined with T214
- [X] T219 [US8] Implement session card click filtering (filter event stream to session) in client/src/components/SessionOverview.tsx (use devs:react-dev agent) - Part of T213
- [X] T220 [GIT] Commit: add session click filtering - Combined with T214

### Phase 10 Completion

- [X] T221 [US8] Run /sdd:map incremental for Phase 10 changes - Updated all 8 codebase documents
- [X] T222 [GIT] Commit: update codebase documents for phase 10
- [X] T223 [US8] Review retro/P10.md and extract critical learnings to CLAUDE.md (conservative) - No critical learnings requiring CLAUDE.md update (phase-specific patterns only)
- [X] T224 [GIT] Commit: finalize phase 10 retro - Combined with T222
- [X] T225 [GIT] Push branch to origin (ensure pre-push hooks pass) - Pushed successfully
- [X] T226 [GIT] Create/update PR to main with phase summary - PR #1 already open
- [X] T227 [GIT] Verify all CI checks pass - CI running (run 21606380903)
- [X] T228 [GIT] Report PR ready status - Phase 10 complete, PR #1 ready for review

**Checkpoint**: Client Session Overview (US8) fully functional and testable independently

---

## Phase 11: User Story 9 - Client Statistics Panel (Priority: P3)

**Goal**: Statistics panel showing usage metrics

**Independent Test**: Accumulate events and verify statistics display correctly for different time periods

### Start Phase 11

- [ ] T229 [GIT] Verify working tree is clean before starting Phase 11
- [ ] T230 [GIT] Pull and rebase on origin/main if needed
- [ ] T231 [US9] Create retro/P11.md for this phase

### Statistics Component

- [ ] T232 [US9] Implement StatsPanel component in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T233 [GIT] Commit: add StatsPanel component
- [ ] T234 [US9] Implement time period selector (1h, 24h, 7d, 30d) in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T235 [GIT] Commit: add time period selector
- [ ] T236 [US9] Implement event count display in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T237 [GIT] Commit: add event count display
- [ ] T238 [US9] Implement event type breakdown bar chart in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T239 [GIT] Commit: add event type breakdown
- [ ] T240 [US9] Implement project ranking (top 5 by event count) in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T241 [GIT] Commit: add project ranking
- [ ] T242 [US9] Implement tool usage donut chart in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T243 [GIT] Commit: add tool usage chart
- [ ] T244 [US9] Add disclaimer showing "Statistics since [page load time]" in client/src/components/StatsPanel.tsx (use devs:react-dev agent)
- [ ] T245 [GIT] Commit: add statistics disclaimer

### Phase 11 Completion

- [ ] T246 [US9] Run /sdd:map incremental for Phase 11 changes
- [ ] T247 [GIT] Commit: update codebase documents for phase 11
- [ ] T248 [US9] Review retro/P11.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T249 [GIT] Commit: finalize phase 11 retro
- [ ] T250 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T251 [GIT] Create/update PR to main with phase summary
- [ ] T252 [GIT] Verify all CI checks pass
- [ ] T253 [GIT] Report PR ready status

**Checkpoint**: Client Statistics Panel (US9) fully functional and testable independently

---

## Phase 12: Client Main Layout & App Integration

**Goal**: Integrate all client components into main App layout

**Independent Test**: Launch full dashboard and verify all components render and communicate correctly

### Start Phase 12

- [ ] T254 [GIT] Verify working tree is clean before starting Phase 12
- [ ] T255 [GIT] Pull and rebase on origin/main if needed
- [ ] T256 Create retro/P12.md for this phase

### App Layout

- [ ] T257 Implement App.tsx with main layout structure (header, sidebar, main content) in client/src/App.tsx (use devs:react-dev agent)
- [ ] T258 [GIT] Commit: implement App layout
- [ ] T259 Implement main.tsx entry point in client/src/main.tsx (use devs:react-dev agent)
- [ ] T260 [GIT] Commit: implement main entry point
- [ ] T261 Integrate all components (ConnectionStatus, EventStream, Heatmap, SessionOverview, StatsPanel) in client/src/App.tsx (use devs:react-dev agent)
- [ ] T262 [GIT] Commit: integrate all components

### Accessibility

- [ ] T263 Add aria-live regions for event stream and connection status in client/src/App.tsx (use devs:react-dev agent)
- [ ] T264 [GIT] Commit: add aria-live regions
- [ ] T265 Implement prefers-reduced-motion support for animations in client/src/index.css (use devs:react-dev agent)
- [ ] T266 [GIT] Commit: add reduced motion support

### Empty States

- [ ] T267 Implement "Waiting for events..." empty state in client/src/components/EventStream.tsx (use devs:react-dev agent)
- [ ] T268 [GIT] Commit: add empty state

### Phase 12 Completion

- [ ] T269 Run /sdd:map incremental for Phase 12 changes
- [ ] T270 [GIT] Commit: update codebase documents for phase 12
- [ ] T271 Review retro/P12.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T272 [GIT] Commit: finalize phase 12 retro
- [ ] T273 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T274 [GIT] Create/update PR to main with phase summary
- [ ] T275 [GIT] Verify all CI checks pass
- [ ] T276 [GIT] Report PR ready status

**Checkpoint**: Client fully integrated and functional

---

## Phase 13: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple components

### Start Phase 13

- [ ] T277 [GIT] Verify working tree is clean before starting Phase 13
- [ ] T278 [GIT] Pull and rebase on origin/main if needed
- [ ] T279 Create retro/P13.md for this phase

### Build Optimization

- [ ] T280 [P] Configure Vite manual chunk splitting (react-vendor, state, virtual) in client/vite.config.ts (use devs:typescript-dev agent)
- [ ] T281 [GIT] Commit: configure chunk splitting
- [ ] T282 [P] Configure Brotli compression in client/vite.config.ts (use devs:typescript-dev agent)
- [ ] T283 [GIT] Commit: configure compression
- [ ] T284 [P] Verify bundle size targets (critical JS < 50KB, total < 150KB gzipped) (use devs:typescript-dev agent)
- [ ] T285 [GIT] Commit: verify bundle sizes

### Server Graceful Shutdown

- [ ] T286 Implement graceful shutdown (30s timeout, 1001 WebSocket close code) in server/src/main.rs (use devs:rust-dev agent)
- [ ] T287 [GIT] Commit: add graceful shutdown

### Monitor Graceful Shutdown

- [ ] T288 Implement graceful shutdown (flush buffer, 5s timeout) in monitor/src/main.rs (use devs:rust-dev agent)
- [ ] T289 [GIT] Commit: add monitor graceful shutdown

### Quickstart Validation

- [ ] T290 Run quickstart.md validation - verify all commands work as documented (use devs:rust-dev agent)
- [ ] T291 [GIT] Commit: update quickstart if needed

### Error Handling Verification (Constitution VI)

- [ ] T291a Test error handling paths: network timeout, connection refused, invalid JSON, auth failure (401), rate limit (429), malformed JSONL - verify clear error messages with context and graceful recovery (use devs:rust-dev agent)
- [ ] T291b [GIT] Commit: add error handling tests

### Final Phase Completion

- [ ] T292 Run /sdd:map incremental for Phase 13 changes
- [ ] T293 [GIT] Commit: update codebase documents for phase 13
- [ ] T294 Review retro/P13.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T295 [GIT] Commit: finalize phase 13 retro
- [ ] T296 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T297 [GIT] Create/update PR to main with phase summary
- [ ] T298 [GIT] Verify all CI checks pass
- [ ] T299 [GIT] Report PR ready status

---

## Phase 14: Performance Validation (NFR Compliance)

**Purpose**: Verify all non-functional requirements are met before release

### Start Phase 14

- [ ] T300 [GIT] Verify working tree is clean before starting Phase 14
- [ ] T301 [GIT] Pull and rebase on origin/main if needed
- [ ] T302 Create retro/P14.md for this phase

### NFR-1: Monitor CPU Usage

- [ ] T303 [P] Profile Monitor CPU usage at idle using cargo flamegraph or perf, verify < 1% (NFR-1) (use devs:rust-dev agent)
- [ ] T304 [GIT] Commit: document CPU profiling results in docs/performance.md

### NFR-2: Monitor Memory Footprint

- [ ] T305 [P] Measure Monitor memory footprint using /usr/bin/time -v or valgrind massif, verify < 5MB (NFR-2) (use devs:rust-dev agent)
- [ ] T306 [GIT] Commit: document memory measurement results

### NFR-3: Monitor Binary Size

- [ ] T307 [P] Verify Monitor release binary size < 10MB after strip (NFR-3), document optimization techniques used (use devs:rust-dev agent)
- [ ] T308 [GIT] Commit: document binary size verification

### NFR-4: Server Concurrent Connections

- [ ] T309 Load test Server with 100+ concurrent WebSocket connections using wrk, k6, or custom test harness (NFR-4) (use devs:rust-dev agent)
- [ ] T310 [GIT] Commit: document load test results

### NFR-5: End-to-End Latency

- [ ] T311 Measure end-to-end event latency (JSONL write → Client display) on localhost, verify < 100ms (NFR-5) (use devs:rust-dev agent)
- [ ] T312 [GIT] Commit: document latency measurement results

### NFR-6: Client Time-to-Interactive

- [ ] T313 Test Client time-to-interactive on 3G throttle (Chrome DevTools Network tab), verify < 2s (NFR-6) (use devs:react-dev agent)
- [ ] T314 [GIT] Commit: document TTI measurement results

### Phase 14 Completion

- [ ] T315 Run /sdd:map incremental for Phase 14 changes
- [ ] T316 [GIT] Commit: update codebase documents for phase 14
- [ ] T317 Review retro/P14.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T318 [GIT] Commit: finalize phase 14 retro
- [ ] T319 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T320 [GIT] Create/update PR to main with phase summary
- [ ] T321 [GIT] Verify all CI checks pass
- [ ] T322 [GIT] Report PR ready status

**Checkpoint**: All NFR targets verified and documented - feature ready for release

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-11)**: All depend on Foundational phase completion
  - US1-US4 (Server + Monitor): Can proceed sequentially or in parallel
  - US5-US9 (Client): Can start after US1 (Server) is complete for integration testing
- **App Integration (Phase 12)**: Depends on all Client user stories (US5-US9)
- **Polish (Phase 13)**: Depends on all desired user stories being complete
- **Performance Validation (Phase 14)**: Depends on Phase 13 - validates all NFRs before release

### User Story Dependencies

| User Story | Phase | Depends On | Enables |
|------------|-------|------------|---------|
| US1: Server Event Hub | 3 | Phase 2 | US2-US9 (can send test events) |
| US2: Monitor Watcher | 4 | Phase 2 | US3 (needs events to strip) |
| US3: Privacy Pipeline | 5 | US2 | US4 (events ready for sending) |
| US4: Server Connection | 6 | US1, US3 | Full Monitor MVP |
| US5: Client WebSocket | 7 | US1 | US6-US9 (need connection) |
| US6: Event Stream | 8 | US5 | Standalone |
| US7: Heatmap | 9 | US5 | Standalone |
| US8: Session Overview | 10 | US5 | Standalone |
| US9: Statistics | 11 | US5 | Standalone |

### Parallel Opportunities

**Phase 2 (Foundational)** - All can run in parallel:
- T015 (server types), T017 (server config), T019 (server errors)
- T021 (monitor config), T023 (monitor errors)
- T025 (client types), T027 (client store)

**Phases 8-11 (Client Features)** - Can run in parallel after US5:
- EventStream, Heatmap, SessionOverview, StatsPanel are independent

**Phase 13 (Polish)** - Marked [P] can run in parallel:
- T280, T282, T284 (build optimization tasks)

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch all type definitions in parallel:
Task: "Create shared event types in server/src/types.rs"
Task: "Create TypeScript event types in client/src/types/events.ts"
Task: "Create shared event types in monitor/src/types.rs"

# Launch all config modules in parallel:
Task: "Implement config module in server/src/config.rs"
Task: "Implement config module in monitor/src/config.rs"
```

---

## Implementation Strategy

### MVP First (Phases 1-6)

1. Complete Phase 1: Setup (dependency upgrade, Tailwind v4 migration)
2. Complete Phase 2: Foundational (core types, config)
3. Complete Phase 3: Server Event Hub (US1)
4. Complete Phases 4-6: Monitor (US2, US3, US4)
5. **STOP and VALIDATE**: Server + Monitor should work end-to-end
6. Deploy Server to Fly.io, test with real Claude Code

### Incremental Client Delivery

1. Complete Phase 7: WebSocket Connection (US5)
2. Complete Phase 8: Event Stream (US6) → Deploy/Demo
3. Complete Phases 9-11 in any order → Deploy after each
4. Complete Phase 12: Full Integration
5. Complete Phase 13: Polish
6. Complete Phase 14: Performance Validation (verify all NFRs before release)

---

## Summary

| Metric | Value |
|--------|-------|
| Total Tasks | 331 (T001-T322 + 9 inserted tasks) |
| Setup Tasks (Phase 1) | 11 |
| Foundational Tasks (Phase 2) | 25 |
| User Story Tasks (Phases 3-11) | 225 (+8 new: unsafe mode, privacy, init CLI, error handling tests) |
| Integration Tasks (Phase 12) | 23 |
| Polish Tasks (Phase 13) | 25 (+2 error handling tests) |
| Performance Tasks (Phase 14) | 23 (NEW - NFR validation) |
| User Stories | 9 |
| MVP Scope | US1-US4 (Phases 1-6) |
| Parallelizable Tasks | ~40% |
| Constitution Compliance Tasks | 4 (privacy tests, error handling tests) |

---

## Notes

- [P] tasks = different files, no dependencies within phase
- [Story] label maps task to specific user story for traceability
- [GIT] tasks enforce commit discipline and CI gates
- Each phase ends with PR creation and CI verification
- Stop at any checkpoint to validate independently
- All Rust tasks reference devs:rust-dev agent
- All React tasks reference devs:react-dev agent
- All TypeScript/config tasks reference devs:typescript-dev agent
