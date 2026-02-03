# Tasks: Monitor Enhanced Data Tracking

**Input**: Design documents from `/specs/005-monitor-enhanced-tracking/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, quickstart.md
**Branch**: `005-monitor-enhanced-tracking`

**Tests**: Tests included where appropriate for tracker modules and privacy compliance.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- **[GIT]**: Git workflow task
- Include exact file paths in descriptions

## Path Conventions

- **Monitor**: `monitor/src/` (Rust)
- **Server**: `server/src/` (Rust)
- **Client**: `client/src/` (TypeScript)
- Trackers: `monitor/src/trackers/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure

### Phase Start
- [x] T001 [GIT] Verify on feature branch 005-monitor-enhanced-tracking and working tree is clean

### Implementation
- [x] T002 Create trackers module directory at monitor/src/trackers/
- [x] T003 [GIT] Commit: create trackers module directory
- [x] T004 [P] Create trackers/mod.rs with module exports in monitor/src/trackers/mod.rs
- [x] T005 [P] Add trackers module to lib.rs in monitor/src/lib.rs
- [x] T006 [GIT] Commit: add trackers module structure
- [x] T007 [P] Extend EventPayload enum with new event types in monitor/src/types.rs (use devs:rust-dev agent)
- [x] T008 [P] Extend server EventPayload enum to match monitor in server/src/types.rs (use devs:rust-dev agent)
- [x] T009 [GIT] Commit: add new event types to monitor and server
- [x] T010 [P] Extend client event types in client/src/types/events.ts (use devs:typescript-dev agent)
- [x] T011 [GIT] Commit: add new event types to client

### Phase Completion
- [x] T012 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T013 [GIT] Create/update PR to main with phase summary
- [x] T014 [GIT] Verify all CI checks pass
- [x] T015 [GIT] Report PR ready status

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared parsing utilities and debounce infrastructure that all trackers need

**CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [x] T016 [GIT] Verify working tree is clean before starting Phase 2
- [x] T017 [GIT] Pull and rebase on origin/main if needed
- [x] T018 Create retro/P2.md for this phase

### Implementation
- [x] T019 [GIT] Commit: initialize phase 2 retro
- [x] T020 Add debounce utility module to monitor/src/utils/debounce.rs (use devs:rust-dev agent)
- [x] T020A [P] Implement session state limit enforcement (max 1000 sessions) in monitor/src/config.rs (use devs:rust-dev agent)
  - Add MAX_TRACKED_SESSIONS config constant (default 1000)
  - Implement LRU session eviction when limit reached
  - Add metrics tracking for session limit warnings
- [x] T021 [GIT] Commit: add debounce utility
- [x] T022 [P] Add shell-like tokenizer for skill name extraction in monitor/src/utils/tokenize.rs (use devs:rust-dev agent)
- [x] T023 [P] Add session filename parser utility in monitor/src/utils/session_filename.rs (use devs:rust-dev agent)
- [x] T024 [GIT] Commit: add tokenizer and session filename utilities
- [x] T025 Create utils module with mod.rs in monitor/src/utils/mod.rs
- [x] T026 [GIT] Commit: create utils module
- [x] T027 Add utils module to lib.rs in monitor/src/lib.rs
- [x] T028 [GIT] Commit: add utils module to lib.rs
- [x] T029 Run codebase mapping for Phase 2 changes (/sdd:map incremental)
- [x] T030 [GIT] Commit: update codebase documents for phase 2
- [x] T031 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T032 [GIT] Commit: finalize phase 2 retro

### Phase Completion
- [x] T033 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T034 [GIT] Create/update PR to main with phase summary
- [x] T035 [GIT] Verify all CI checks pass
- [x] T036 [GIT] Report PR ready status

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Token Usage by Model (Priority: P1) MVP

**Goal**: Track per-model token consumption from stats-cache.json

**Independent Test**: Start Claude Code session, perform actions, verify monitor displays global token counts broken down by model.

### Phase Start
- [x] T037 [GIT] Verify working tree is clean before starting Phase 3
- [x] T038 [GIT] Pull and rebase on origin/main if needed
- [x] T039 [US1] Create retro/P3.md for this phase

### Tests for User Story 1
- [x] T040 [GIT] Commit: initialize phase 3 retro
- [x] T041 [P] [US1] Unit test for stats_tracker parsing in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T042 [P] [US1] Unit test for TokenUsageEvent emission in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T042A [P] [US1] Unit test for 200ms debounce timing behavior in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
  - Mock file watcher events at <100ms intervals
  - Verify only 1 event emitted per 200ms window
- [x] T043 [GIT] Commit: add stats_tracker tests (including debounce timing)

### Implementation for User Story 1
- [x] T044 [US1] Implement StatsCache struct for JSON parsing in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T045 [GIT] Commit: add StatsCache struct
- [x] T046 [US1] Implement stats_tracker module with file watching in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T047 [GIT] Commit: implement stats_tracker file watching
- [x] T048 [US1] Implement TokenUsageEvent emission per model in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T049 [GIT] Commit: implement TokenUsageEvent emission
- [x] T050 [US1] Add 200ms debounce for stats-cache.json changes in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T051 [GIT] Commit: add debounce to stats_tracker
- [x] T052 [US1] Add JSON parse failure retry with 100ms delay in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [x] T053 [GIT] Commit: add JSON parse retry logic
- [x] T054 [US1] Wire stats_tracker to watcher in monitor/src/watcher.rs (use devs:rust-dev agent)
- [x] T055 [GIT] Commit: wire stats_tracker to watcher
- [x] T056 [US1] Add client store handler for token_usage events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [x] T057 [GIT] Commit: add client token_usage handler
- [x] T058 [US1] Run /sdd:map incremental for Phase 3 changes
- [x] T059 [GIT] Commit: update codebase documents for phase 3
- [x] T060 [US1] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T061 [GIT] Commit: finalize phase 3 retro

### Phase Completion
- [x] T062 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T063 [GIT] Create/update PR to main with phase summary
- [x] T064 [GIT] Verify all CI checks pass
- [x] T065 [GIT] Report PR ready status

**Checkpoint**: Token usage tracking fully functional and testable independently

---

## Phase 4: User Story 2 - Track Agent Spawns (Priority: P1)

**Goal**: Track Task tool agent spawns from session JSONL files

**Independent Test**: Ask Claude to perform a task triggering agent spawns, verify monitor captures subagent_type for each Task tool invocation.

### Phase Start
- [x] T066 [GIT] Verify working tree is clean before starting Phase 4
- [x] T067 [GIT] Pull and rebase on origin/main if needed
- [x] T068 [US2] Create retro/P4.md for this phase

### Tests for User Story 2
- [ ] T069 [GIT] Commit: initialize phase 4 retro
- [ ] T070 [P] [US2] Unit test for Task tool_use parsing in monitor/src/trackers/agent_tracker.rs (use devs:rust-dev agent)
- [ ] T071 [P] [US2] Unit test for AgentSpawnEvent emission in monitor/src/trackers/agent_tracker.rs (use devs:rust-dev agent)
- [ ] T072 [GIT] Commit: add agent_tracker tests

### Implementation for User Story 2
- [ ] T073 [US2] Implement TaskToolUse struct for JSON parsing in monitor/src/trackers/agent_tracker.rs (use devs:rust-dev agent)
- [ ] T074 [GIT] Commit: add TaskToolUse struct
- [ ] T075 [US2] Implement agent_tracker module in monitor/src/trackers/agent_tracker.rs (use devs:rust-dev agent)
- [ ] T076 [GIT] Commit: implement agent_tracker module
- [ ] T077 [US2] Integrate agent_tracker with existing JSONL parsing in monitor/src/parser.rs (use devs:rust-dev agent)
- [ ] T078 [GIT] Commit: integrate agent_tracker with parser
- [ ] T079 [US2] Add client store handler for agent_spawn events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T080 [GIT] Commit: add client agent_spawn handler
- [ ] T081 [US2] Run /sdd:map incremental for Phase 4 changes
- [ ] T082 [GIT] Commit: update codebase documents for phase 4
- [ ] T083 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T084 [GIT] Commit: finalize phase 4 retro

### Phase Completion
- [ ] T085 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T086 [GIT] Create/update PR to main with phase summary
- [ ] T087 [GIT] Verify all CI checks pass
- [ ] T088 [GIT] Report PR ready status

**Checkpoint**: Agent spawn tracking fully functional and testable independently

---

## Phase 5: User Story 3 - Monitor Skill Invocations (Priority: P1)

**Goal**: Track skill/slash command invocations from history.jsonl

**Independent Test**: Invoke several slash commands, verify monitor captures each with timestamp and command name.

### Phase Start
- [ ] T089 [GIT] Verify working tree is clean before starting Phase 5
- [ ] T090 [GIT] Pull and rebase on origin/main if needed
- [ ] T091 [US3] Create retro/P5.md for this phase

### Tests for User Story 3
- [ ] T092 [GIT] Commit: initialize phase 5 retro
- [ ] T093 [P] [US3] Unit test for history.jsonl parsing in monitor/src/trackers/skill_tracker.rs (use devs:rust-dev agent)
- [ ] T094 [P] [US3] Unit test for shell-like tokenization in monitor/src/utils/tokenize.rs (use devs:rust-dev agent)
- [ ] T094A [P] [US3] Unit test for quoted string handling in shell-like tokenizer (use devs:rust-dev agent)
  - Test: `/commit -m "fix: update docs"` → `commit`
  - Test: `/"my skill" arg1` → `"my skill"`
  - Test: `/sdd:plan` → `sdd:plan`
- [ ] T095 [GIT] Commit: add skill_tracker tests (including quoted command names)

### Implementation for User Story 3
- [ ] T096 [US3] Implement HistoryEntry struct for JSON parsing in monitor/src/trackers/skill_tracker.rs (use devs:rust-dev agent)
- [ ] T097 [GIT] Commit: add HistoryEntry struct
- [ ] T098 [US3] Implement skill_tracker module with file watching in monitor/src/trackers/skill_tracker.rs (use devs:rust-dev agent)
- [ ] T099 [GIT] Commit: implement skill_tracker module
- [ ] T100 [US3] Implement skill name extraction using shell-like tokenizer in monitor/src/trackers/skill_tracker.rs (use devs:rust-dev agent)
- [ ] T101 [GIT] Commit: implement skill name extraction
- [ ] T102 [US3] Wire skill_tracker to watcher in monitor/src/watcher.rs (use devs:rust-dev agent)
- [ ] T103 [GIT] Commit: wire skill_tracker to watcher
- [ ] T104 [US3] Add client store handler for skill_invocation events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T105 [GIT] Commit: add client skill_invocation handler
- [ ] T106 [US3] Run /sdd:map incremental for Phase 5 changes
- [ ] T107 [GIT] Commit: update codebase documents for phase 5
- [ ] T108 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T109 [GIT] Commit: finalize phase 5 retro

### Phase Completion
- [ ] T110 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T111 [GIT] Create/update PR to main with phase summary
- [ ] T112 [GIT] Verify all CI checks pass
- [ ] T113 [GIT] Report PR ready status

**Checkpoint**: Skill invocation tracking fully functional and testable independently

---

## Phase 6: User Story 4 - Track Todo Progress with Abandonment (Priority: P2)

**Goal**: Track todo list progress per session with abandoned task detection

**Independent Test**: Create tasks, complete some, leave others pending, end session, verify correct categorization.

### Phase Start
- [ ] T114 [GIT] Verify working tree is clean before starting Phase 6
- [ ] T115 [GIT] Pull and rebase on origin/main if needed
- [ ] T116 [US4] Create retro/P6.md for this phase

### Tests for User Story 4
- [ ] T117 [GIT] Commit: initialize phase 6 retro
- [ ] T118 [P] [US4] Unit test for todo filename parsing in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T119 [P] [US4] Unit test for todo status counting in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T120 [P] [US4] Unit test for abandonment detection via summary event in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T121 [GIT] Commit: add todo_tracker tests

### Implementation for User Story 4
- [ ] T122 [US4] Implement TodoEntry and TodoFile structs in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T123 [GIT] Commit: add TodoEntry and TodoFile structs
- [ ] T124 [US4] Implement todo_tracker module with file watching in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T125 [GIT] Commit: implement todo_tracker file watching
- [ ] T126 [US4] Implement session-todo correlation via filename parsing in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T127 [GIT] Commit: implement session-todo correlation
- [ ] T128 [US4] Implement abandonment detection via summary event correlation in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T129 [GIT] Commit: implement abandonment detection
- [ ] T130 [US4] Wire todo_tracker to watcher in monitor/src/watcher.rs (use devs:rust-dev agent)
- [ ] T131 [GIT] Commit: wire todo_tracker to watcher
- [ ] T132 [US4] Add 100ms debounce for todo file changes in monitor/src/trackers/todo_tracker.rs (use devs:rust-dev agent)
- [ ] T133 [GIT] Commit: add debounce to todo_tracker
- [ ] T134 [US4] Add client store handler for todo_progress events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T135 [GIT] Commit: add client todo_progress handler
- [ ] T136 [US4] Run /sdd:map incremental for Phase 6 changes
- [ ] T137 [GIT] Commit: update codebase documents for phase 6
- [ ] T138 [US4] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T139 [GIT] Commit: finalize phase 6 retro

### Phase Completion
- [ ] T140 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T141 [GIT] Create/update PR to main with phase summary
- [ ] T142 [GIT] Verify all CI checks pass
- [ ] T143 [GIT] Report PR ready status

**Checkpoint**: Todo progress tracking with abandonment fully functional

---

## Phase 7: User Story 5 - Track File Edit Line Changes (Priority: P2)

**Goal**: Track lines added/removed by diffing consecutive file-history versions

**Independent Test**: Have Claude edit files, verify monitor calculates lines added/removed by diffing vN vs vN-1.

### Phase Start
- [ ] T144 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T145 [GIT] Pull and rebase on origin/main if needed
- [ ] T146 [US5] Create retro/P7.md for this phase

### Tests for User Story 5
- [ ] T147 [GIT] Commit: initialize phase 7 retro
- [ ] T148 [P] [US5] Unit test for file version parsing (@vN pattern) in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T149 [P] [US5] Unit test for line diff calculation in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T150 [P] [US5] Unit test for v1 skip behavior in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T151 [GIT] Commit: add file_history_tracker tests

### Implementation for User Story 5
- [ ] T152 [US5] Add similar crate dependency for line diffs in monitor/Cargo.toml (use devs:rust-dev agent)
- [ ] T153 [GIT] Commit: add similar crate dependency
- [ ] T154 [US5] Implement FileVersion struct and parsing in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T155 [GIT] Commit: add FileVersion struct
- [ ] T156 [US5] Implement file_history_tracker module with directory watching in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T157 [GIT] Commit: implement file_history_tracker directory watching
- [ ] T158 [US5] Implement async diff operation for vN vs vN-1 in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T159 [GIT] Commit: implement async diff operation
- [ ] T160 [US5] Implement v1 skip logic (no diff for initial versions) in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T161 [GIT] Commit: implement v1 skip logic
- [ ] T162 [US5] Wire file_history_tracker to watcher in monitor/src/watcher.rs (use devs:rust-dev agent)
- [ ] T163 [GIT] Commit: wire file_history_tracker to watcher
- [ ] T164 [US5] Add 100ms debounce for file-history changes in monitor/src/trackers/file_history_tracker.rs (use devs:rust-dev agent)
- [ ] T165 [GIT] Commit: add debounce to file_history_tracker
- [ ] T166 [US5] Add client store handler for file_change events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T167 [GIT] Commit: add client file_change handler
- [ ] T168 [US5] Run /sdd:map incremental for Phase 7 changes
- [ ] T169 [GIT] Commit: update codebase documents for phase 7
- [ ] T170 [US5] Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T171 [GIT] Commit: finalize phase 7 retro

### Phase Completion
- [ ] T172 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T173 [GIT] Create/update PR to main with phase summary
- [ ] T174 [GIT] Verify all CI checks pass
- [ ] T175 [GIT] Report PR ready status

**Checkpoint**: File edit line change tracking fully functional

---

## Phase 8: User Story 6 - View Session Metrics (Priority: P2)

**Goal**: Track global session metrics from stats-cache.json

**Depends On**: Phase 3 (stats_tracker foundation must be complete before starting)

**Note**: SessionMetricsEvent extends existing stats_tracker implementation (T046). Do NOT create a new tracker module; extend stats_tracker.rs.

**Independent Test**: Run Claude Code sessions, verify monitor captures message counts, tool call counts, session duration.

### Phase Start
- [ ] T176 [GIT] Verify working tree is clean before starting Phase 8
- [ ] T177 [GIT] Pull and rebase on origin/main if needed
- [ ] T178 [US6] Create retro/P8.md for this phase

### Implementation for User Story 6
- [ ] T179 [GIT] Commit: initialize phase 8 retro
- [ ] T180 [US6] Extend StatsCache struct with session metrics fields in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [ ] T181 [GIT] Commit: extend StatsCache with session metrics
- [ ] T182 [US6] Implement SessionMetricsEvent emission in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [ ] T183 [GIT] Commit: implement SessionMetricsEvent emission
- [ ] T184 [US6] Add client store handler for session_metrics events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T185 [GIT] Commit: add client session_metrics handler
- [ ] T186 [US6] Run /sdd:map incremental for Phase 8 changes
- [ ] T187 [GIT] Commit: update codebase documents for phase 8
- [ ] T188 [US6] Review retro/P8.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T189 [GIT] Commit: finalize phase 8 retro

### Phase Completion
- [ ] T190 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T191 [GIT] Create/update PR to main with phase summary
- [ ] T192 [GIT] Verify all CI checks pass
- [ ] T193 [GIT] Report PR ready status

**Checkpoint**: Session metrics tracking fully functional

---

## Phase 9: User Story 7 - View Activity Patterns by Hour (Priority: P3)

**Goal**: Track hourly activity distribution from stats-cache.json

**Independent Test**: Use Claude Code at different times, verify monitor captures hourCounts data.

### Phase Start
- [ ] T194 [GIT] Verify working tree is clean before starting Phase 9
- [ ] T195 [GIT] Pull and rebase on origin/main if needed
- [ ] T196 [US7] Create retro/P9.md for this phase

### Implementation for User Story 7
- [ ] T197 [GIT] Commit: initialize phase 9 retro
- [ ] T198 [US7] Implement ActivityPatternEvent emission in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [ ] T199 [GIT] Commit: implement ActivityPatternEvent emission
- [ ] T200 [US7] Add client store handler for activity_pattern events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T201 [GIT] Commit: add client activity_pattern handler
- [ ] T202 [US7] Run /sdd:map incremental for Phase 9 changes
- [ ] T203 [GIT] Commit: update codebase documents for phase 9
- [ ] T204 [US7] Review retro/P9.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T205 [GIT] Commit: finalize phase 9 retro

### Phase Completion
- [ ] T206 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T207 [GIT] Create/update PR to main with phase summary
- [ ] T208 [GIT] Verify all CI checks pass
- [ ] T209 [GIT] Report PR ready status

**Checkpoint**: Activity pattern tracking fully functional

---

## Phase 10: User Story 8 - View Model Distribution (Priority: P3)

**Goal**: Track usage distribution across Claude models

**Independent Test**: Use different models during sessions, verify monitor captures per-model usage.

### Phase Start
- [ ] T210 [GIT] Verify working tree is clean before starting Phase 10
- [ ] T211 [GIT] Pull and rebase on origin/main if needed
- [ ] T212 [US8] Create retro/P10.md for this phase

### Implementation for User Story 8
- [ ] T213 [GIT] Commit: initialize phase 10 retro
- [ ] T214 [US8] Implement ModelDistributionEvent emission in monitor/src/trackers/stats_tracker.rs (use devs:rust-dev agent)
- [ ] T215 [GIT] Commit: implement ModelDistributionEvent emission
- [ ] T216 [US8] Add client store handler for model_distribution events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T217 [GIT] Commit: add client model_distribution handler
- [ ] T218 [US8] Run /sdd:map incremental for Phase 10 changes
- [ ] T219 [GIT] Commit: update codebase documents for phase 10
- [ ] T220 [US8] Review retro/P10.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T221 [GIT] Commit: finalize phase 10 retro

### Phase Completion
- [ ] T222 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T223 [GIT] Create/update PR to main with phase summary
- [ ] T224 [GIT] Verify all CI checks pass
- [ ] T225 [GIT] Report PR ready status

**Checkpoint**: Model distribution tracking fully functional

---

## Phase 11: User Story 9 - Track Active Projects (Priority: P3)

**Goal**: Track which projects have active Claude Code sessions

**Independent Test**: Run Claude Code in different project directories, verify monitor identifies active projects.

### Phase Start
- [ ] T226 [GIT] Verify working tree is clean before starting Phase 11
- [ ] T227 [GIT] Pull and rebase on origin/main if needed
- [ ] T228 [US9] Create retro/P11.md for this phase

### Tests for User Story 9
- [ ] T229 [GIT] Commit: initialize phase 11 retro
- [ ] T230 [P] [US9] Unit test for project path slug parsing in monitor/src/trackers/project_tracker.rs (use devs:rust-dev agent)
- [ ] T231 [P] [US9] Unit test for active session detection (no summary event) in monitor/src/trackers/project_tracker.rs (use devs:rust-dev agent)
- [ ] T232 [GIT] Commit: add project_tracker tests

### Implementation for User Story 9
- [ ] T233 [US9] Implement project_tracker module in monitor/src/trackers/project_tracker.rs (use devs:rust-dev agent)
- [ ] T234 [GIT] Commit: implement project_tracker module
- [ ] T235 [US9] Implement project path slug parsing in monitor/src/trackers/project_tracker.rs (use devs:rust-dev agent)
- [ ] T236 [GIT] Commit: implement project path slug parsing
- [ ] T237 [US9] Implement active session detection via summary event check in monitor/src/trackers/project_tracker.rs (use devs:rust-dev agent)
- [ ] T238 [GIT] Commit: implement active session detection
- [ ] T239 [US9] Wire project_tracker to watcher in monitor/src/watcher.rs (use devs:rust-dev agent)
- [ ] T240 [GIT] Commit: wire project_tracker to watcher
- [ ] T241 [US9] Add client store handler for project_activity events in client/src/hooks/useEventStore.ts (use devs:typescript-dev agent)
- [ ] T242 [GIT] Commit: add client project_activity handler
- [ ] T243 [US9] Run /sdd:map incremental for Phase 11 changes
- [ ] T244 [GIT] Commit: update codebase documents for phase 11
- [ ] T245 [US9] Review retro/P11.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T246 [GIT] Commit: finalize phase 11 retro

### Phase Completion
- [ ] T247 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T248 [GIT] Create/update PR to main with phase summary
- [ ] T249 [GIT] Verify all CI checks pass
- [ ] T250 [GIT] Report PR ready status

**Checkpoint**: Project activity tracking fully functional

---

## Phase 12: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, privacy verification, and documentation

### Phase Start
- [ ] T251 [GIT] Verify working tree is clean before starting Phase 12
- [ ] T252 [GIT] Pull and rebase on origin/main if needed
- [ ] T253 Create retro/P12.md for this phase

### Implementation
- [ ] T254 [GIT] Commit: initialize phase 12 retro
- [ ] T255 [P] Add integration test for all trackers in monitor/src/tests/enhanced_tracking_test.rs (use devs:rust-dev agent)
- [ ] T256 [GIT] Commit: add integration test for all trackers
- [ ] T257 [P] Add privacy compliance tests - verify no code/prompts transmitted in monitor/src/tests/privacy_test.rs (use devs:rust-dev agent)
- [ ] T258 [GIT] Commit: add privacy compliance tests
- [ ] T259 [P] Add inotify limit warning (80% threshold) in monitor/src/watcher.rs (use devs:rust-dev agent)
- [ ] T260 [GIT] Commit: add inotify limit warning
- [ ] T261 [P] Update quickstart.md with all new event types in specs/005-monitor-enhanced-tracking/quickstart.md
- [ ] T262 [GIT] Commit: update quickstart.md
- [ ] T263 Run cargo clippy and fix any warnings in monitor crate
- [ ] T264 [GIT] Commit: fix clippy warnings
- [ ] T265 Run cargo fmt --check and fix any formatting issues
- [ ] T266 [GIT] Commit: fix formatting issues
- [ ] T267 Run full test suite: cargo test -p vibetea-monitor --test-threads=1
- [ ] T268 [GIT] Commit: verify test suite passes
- [ ] T269 Run /sdd:map incremental for Phase 12 changes
- [ ] T270 [GIT] Commit: update codebase documents for phase 12
- [ ] T271 Review retro/P12.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T272 [GIT] Commit: finalize phase 12 retro

### Event Pipeline Hardening
- [ ] T272A [P] Implement event coalescing strategy for rapid file changes in monitor/src/sender.rs (use devs:rust-dev agent)
  - Add event deduplication logic for duplicate consecutive events
  - Batch similar events within 100ms window
  - Add metrics for coalesced event counts
- [ ] T272B [GIT] Commit: add event coalescing
- [ ] T272C Implement graceful shutdown signal handling in monitor/src/main.rs (use devs:rust-dev agent)
  - Install SIGTERM/SIGINT handlers
  - Flush in-flight events before exit (300ms timeout)
  - Drain sender buffer queue
- [ ] T272D [GIT] Commit: add graceful shutdown handling

### Phase Completion
- [ ] T273 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T274 [GIT] Create/update PR to main with phase summary
- [ ] T275 [GIT] Verify all CI checks pass
- [ ] T276 [GIT] Report PR ready status

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-11)**: All depend on Foundational phase completion
  - US1 (P1), US2 (P1), US3 (P1): Can proceed in parallel or sequentially
  - US4 (P2), US5 (P2), US6 (P2): Depend on Foundational, can be parallel
  - US7 (P3), US8 (P3), US9 (P3): Depend on Foundational, can be parallel
- **Polish (Phase 12)**: Depends on all desired user stories being complete

### User Story Dependencies

| User Story | Priority | Dependencies | Notes |
|------------|----------|--------------|-------|
| US1 - Token Usage | P1 | Foundational | stats_tracker foundation |
| US2 - Agent Spawns | P1 | Foundational | Uses existing JSONL parser |
| US3 - Skill Invocations | P1 | Foundational | history.jsonl watching |
| US4 - Todo Progress | P2 | Foundational | Needs summary event correlation |
| US5 - File Edit Lines | P2 | Foundational | Async diff operations |
| US6 - Session Metrics | P2 | US1 | Extends stats_tracker |
| US7 - Activity Patterns | P3 | US1 | Extends stats_tracker |
| US8 - Model Distribution | P3 | US1 | Extends stats_tracker |
| US9 - Project Activity | P3 | Foundational | projects/ directory |

### Parallel Opportunities

Within each phase, tasks marked [P] can run in parallel:

- **Phase 1**: T004/T005, T007/T008 (types in different crates)
- **Phase 2**: T022/T023 (different utility modules)
- **Phase 3**: T041/T042 (different test functions)
- **Phase 4**: T070/T071 (different test functions)
- **Phase 5**: T093/T094 (different test files)
- **Phase 6**: T118/T119/T120 (different test functions)
- **Phase 7**: T148/T149/T150 (different test functions)
- **Phase 11**: T230/T231 (different test functions)
- **Phase 12**: T255/T257/T259/T261 (different files)

---

## Parallel Example: Phase 3 (User Story 1)

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for stats_tracker parsing" [T041]
Task: "Unit test for TokenUsageEvent emission" [T042]

# After tests written, implement in sequence:
Task: "Implement StatsCache struct" [T044]
Task: "Implement stats_tracker module" [T046]
Task: "Implement TokenUsageEvent emission" [T048]
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Token Usage)
4. Complete Phase 4: User Story 2 (Agent Spawns)
5. Complete Phase 5: User Story 3 (Skill Invocations)
6. **STOP and VALIDATE**: Test P1 stories independently
7. Deploy/demo if ready - this is your MVP

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1/US2/US3 (P1) → Test → Deploy (MVP!)
3. Add US4/US5/US6 (P2) → Test → Deploy
4. Add US7/US8/US9 (P3) → Test → Deploy
5. Polish → Final Deploy

---

## Summary

| Metric | Count |
|--------|-------|
| Total Tasks | 283 |
| Setup Phase | 15 tasks |
| Foundational Phase | 22 tasks |
| User Story Tasks | ~206 tasks (9 stories) |
| Polish Phase | 30 tasks |
| Parallelizable Tasks | ~43 |

**MVP Scope**: Phases 1-5 (Setup + Foundational + US1/US2/US3)

**Key Trackers to Implement**:
1. `stats_tracker.rs` - stats-cache.json (US1, US6, US7, US8)
2. `agent_tracker.rs` - Task tool parsing (US2)
3. `skill_tracker.rs` - history.jsonl (US3)
4. `todo_tracker.rs` - todos/*.json (US4)
5. `file_history_tracker.rs` - file-history/ (US5)
6. `project_tracker.rs` - projects/ (US9)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- [GIT] label marks git workflow tasks
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD where specified)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Use `--test-threads=1` for all Rust tests (env var isolation)
