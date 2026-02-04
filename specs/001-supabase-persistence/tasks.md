# Tasks: Supabase Persistence Layer

**Input**: Design documents from `/specs/001-supabase-persistence/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/
**Branch**: `001-supabase-persistence`

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- **[GIT]**: Git workflow task (commit, push, PR, CI verification)
- Include exact file paths in descriptions

## Path Conventions

- **Monitor (Rust)**: `monitor/src/`
- **Client (TypeScript/React)**: `client/src/`
- **Supabase Edge Functions (Deno)**: `supabase/functions/`
- **Database Migrations**: `supabase/migrations/`

---

## Phase 1: Setup (Supabase Infrastructure)

**Purpose**: Initialize Supabase project structure and local development environment

### Phase Start
- [x] T001 [GIT] Verify on feature branch 001-supabase-persistence and working tree is clean
- [x] T002 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T003 Initialize Supabase project with `supabase init` in repository root
- [x] T004 [GIT] Commit: initialize supabase project
- [x] T005 [P] Create supabase/.env.local with template configuration per quickstart.md (include all env vars: SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY, VIBETEA_SUBSCRIBER_TOKEN, VIBETEA_PUBLIC_KEYS)
- [x] T006 [P] Create monitor/.env.local with VIBETEA_SUPABASE_URL and VIBETEA_SUPABASE_BATCH_INTERVAL_SECS; update client/.env.local with VITE_SUPABASE_URL placeholder
- [x] T007 [GIT] Commit: add environment configuration templates

### Phase Completion
- [x] T008 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T009 [GIT] Create/update PR to main with phase summary
- [x] T010 [GIT] Verify all CI checks pass
- [x] T011 [GIT] Report PR ready status

---

## Phase 2: Foundational (Database Schema & Shared Types)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [x] T012 [GIT] Verify working tree is clean before starting Phase 2
- [x] T013 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T014 Create retro/P2.md for this phase
- [x] T015 [GIT] Commit: initialize phase 2 retro
- [x] T016 Create database migration supabase/migrations/20260203000000_create_events_table.sql with events table, indexes, and RLS per data-model.md
- [x] T017 [GIT] Commit: add events table migration
- [x] T018 [P] Create bulk_insert_events PostgreSQL function in supabase/migrations/20260203000001_create_functions.sql per data-model.md
- [x] T019 [P] Create get_hourly_aggregates PostgreSQL function in supabase/migrations/20260203000001_create_functions.sql per data-model.md
- [x] T020 [GIT] Commit: add database functions for bulk insert and aggregation
- [x] T021 Add HourlyAggregate type to client/src/types/events.ts per data-model.md (use devs:typescript-dev agent)
- [x] T022 [GIT] Commit: add HourlyAggregate type
- [x] T023 Create shared Ed25519 signature verification utility in supabase/functions/_shared/auth.ts per research.md (use devs:typescript-dev agent)
- [x] T024 [GIT] Commit: add shared auth utility for edge functions
- [x] T025 Run codebase mapping for Phase 2 changes (/sdd:map incremental)
- [x] T026 [GIT] Commit: update codebase documents for phase 2
- [x] T027 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T028 [GIT] Commit: finalize phase 2 retro

### Phase Completion
- [x] T029 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T030 [GIT] Create/update PR to main with phase summary
- [x] T031 [GIT] Verify all CI checks pass
- [x] T032 [GIT] Report PR ready status

**Checkpoint**: Foundation ready - database schema, functions, and shared types are in place

---

## Phase 3: User Story 4 - Secure Edge Functions Handle All Database Access (Priority: P4)

**Goal**: Implement edge functions with authentication that mediate all database access, including database RPC wiring

**Why this story first**: This is the security foundation - both US2 (ingest) and US3 (query) depend on having fully functional authenticated edge functions. Even though spec priority is P4, it's architecturally blocking.

**Independent Test**: Verify direct database access fails (RLS denies), authenticated edge function requests succeed

**Dependencies**: T023-T024 (shared auth utility) MUST be complete before edge function scaffolds

### Phase Start
- [x] T033 [GIT] Verify working tree is clean before starting Phase 3
- [x] T034 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T035 [US4] Create retro/P3.md for this phase
- [x] T036 [GIT] Commit: initialize phase 3 retro
- [x] T037 [P] [US4] Create ingest edge function scaffold in supabase/functions/ingest/index.ts with Deno imports; imports _shared/auth.ts (blockedBy: T024) (use devs:typescript-dev agent)
- [x] T038 [P] [US4] Create query edge function scaffold in supabase/functions/query/index.ts with Deno imports; imports _shared/auth.ts (blockedBy: T024) (use devs:typescript-dev agent)
- [x] T039 [GIT] Commit: scaffold edge function structures
- [x] T040 Add deno test configuration to CI workflow in .github/workflows/ci.yml (moved from Phase 1 - edge functions now exist)
- [x] T041 [GIT] Commit: add edge function tests to CI
- [x] T042 [P] [US4] Implement Ed25519 signature verification in ingest edge function per ingest.yaml contract (use devs:typescript-dev agent)
- [x] T043 [P] [US4] Implement bearer token validation in query edge function per query.yaml contract (use devs:typescript-dev agent)
- [x] T044 [GIT] Commit: implement authentication for both edge functions
- [x] T045 [US4] Add request body validation to ingest edge function (max 1000 events, event schema validation, eventType→event_type field mapping) per ingest.yaml (use devs:typescript-dev agent)
- [x] T046 [GIT] Commit: add ingest request validation
- [x] T047 [US4] Add query parameter validation to query edge function (days must be 7 or 30) per query.yaml (use devs:typescript-dev agent)
- [x] T048 [GIT] Commit: add query parameter validation
- [x] T049 [US4] Add error response handling to both edge functions per contracts (use devs:typescript-dev agent)
- [x] T050 [GIT] Commit: add error response handling
- [x] T051 [US4] Wire ingest edge function to database - implement bulk_insert_events RPC call (blockedBy: T020) (use devs:typescript-dev agent)
- [x] T052 [GIT] Commit: wire ingest to database
- [x] T053 [US4] Wire query edge function to database - implement get_hourly_aggregates RPC call (blockedBy: T019) (use devs:typescript-dev agent)
- [x] T054 [GIT] Commit: wire query to database
- [x] T055 [P] [US4] Create unit tests for ingest auth in supabase/functions/ingest/index.test.ts (use devs:typescript-dev agent)
- [x] T056 [P] [US4] Create unit tests for query auth in supabase/functions/query/index.test.ts (use devs:typescript-dev agent)
- [x] T057 [GIT] Commit: add edge function auth tests
- [x] T058 [US4] Create integration test verifying RLS denies direct database access (validates SC-003) in supabase/functions/_tests/rls.test.ts (use devs:typescript-dev agent)
- [x] T059 [GIT] Commit: add RLS negative test
- [x] T060 [US4] Run codebase mapping for Phase 3 changes (/sdd:map incremental)
- [x] T061 [GIT] Commit: update codebase documents for phase 3
- [x] T062 [US4] Review retro/P3.md and extract critical learnings to CLAUDE.md (security patterns take priority)
- [x] T063 [GIT] Commit: finalize phase 3 retro

### Phase Completion
- [x] T064 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T065 [GIT] Create/update PR to main with phase summary
- [x] T066 [GIT] Verify all CI checks pass
- [x] T067 [GIT] Report PR ready status

**Checkpoint**: Edge functions fully functional with auth and database wiring; RLS blocks direct access; service role bypasses RLS

---

## Phase 4: User Story 2 - Monitor Batches and Persists Events (Priority: P2)

**Goal**: Monitor batches events locally and sends them to the ingest edge function periodically

**Independent Test**: Run monitor with `VIBETEA_SUPABASE_URL` configured, generate events, verify they appear in Supabase database

**Dependencies**: Phase 3 MUST be complete (ingest edge function with database wiring ready)

### Phase Start
- [x] T068 [GIT] Verify working tree is clean before starting Phase 4
- [x] T069 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T070 [US2] Create retro/P4.md for this phase
- [x] T071 [GIT] Commit: initialize phase 4 retro
- [x] T072 [US2] Add PersistenceConfig struct to monitor/src/config.rs with VIBETEA_SUPABASE_URL, VIBETEA_SUPABASE_BATCH_INTERVAL_SECS, and VIBETEA_SUPABASE_RETRY_LIMIT (use devs:rust-dev agent)
- [x] T073 [GIT] Commit: add persistence configuration
- [x] T074 [US2] Create persistence module scaffold in monitor/src/persistence.rs with EventBatcher struct (use devs:rust-dev agent)
- [x] T075 [GIT] Commit: scaffold persistence module
- [x] T076 [US2] Add pub mod persistence to monitor/src/lib.rs
- [x] T077 [GIT] Commit: export persistence module
- [x] T078 [US2] Implement event buffering in EventBatcher with max 1000 events limit per FR-002/FR-010 (use devs:rust-dev agent)
- [x] T079 [GIT] Commit: implement event buffering
- [x] T080 [US2] Implement batch submission with Ed25519 signing using existing key pair; JSON serializes eventType field (use devs:rust-dev agent)
- [x] T081 [GIT] Commit: implement signed batch submission
- [x] T082 [US2] Implement retry logic with exponential backoff (1s, 2s, 4s; max 3 retries per FR-015) per spec edge cases (use devs:rust-dev agent)
- [x] T083 [GIT] Commit: implement retry logic
- [x] T084 [US2] Implement batch interval timer using tokio - sends batch when interval elapses OR 1000 events queued (use devs:rust-dev agent)
- [x] T085 [GIT] Commit: implement batch interval timer
- [x] T086 [US2] Initialize persistence in monitor/src/main.rs if VIBETEA_SUPABASE_URL is configured (use devs:rust-dev agent)
- [x] T087 [GIT] Commit: initialize persistence in monitor main
- [x] T088 [P] [US2] Create unit tests for EventBatcher in monitor/src/persistence.rs with mocked HTTP (use devs:rust-dev agent)
- [x] T089 [P] [US2] Create integration test for batch submission in supabase/functions/ingest/index.test.ts (use devs:typescript-dev agent)
- [x] T090 [GIT] Commit: add persistence tests
- [x] T091 [US2] Run codebase mapping for Phase 4 changes (/sdd:map incremental)
- [x] T092 [GIT] Commit: update codebase documents for phase 4
- [x] T093 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T094 [GIT] Commit: finalize phase 4 retro

### Phase Completion
- [x] T095 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T096 [GIT] Create/update PR to main with phase summary
- [x] T097 [GIT] Verify all CI checks pass
- [x] T098 [GIT] Report PR ready status

**Checkpoint**: Monitor batches and persists events; events visible in Supabase database via edge function

---

## Phase 5: User Story 3 - Client Queries Historic Data (Priority: P3)

**Goal**: Client fetches hourly aggregates from query edge function and stores them in Zustand

**Independent Test**: Seed historic data in Supabase, load client, verify heatmap displays the seeded data

**Dependencies**: Phase 3 MUST be complete (query edge function with database wiring ready)

### Phase Start
- [x] T099 [GIT] Verify working tree is clean before starting Phase 5
- [x] T100 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T101 [US3] Create retro/P5.md for this phase
- [x] T102 [GIT] Commit: initialize phase 5 retro
- [x] T103 [US3] Add query integration test verifying aggregation returns HourlyAggregate[] (sorted DESC by date/hour) in supabase/functions/query/index.test.ts (use devs:typescript-dev agent)
- [x] T104 [GIT] Commit: add query integration test
- [x] T105 [US3] Extend EventStore in client/src/store/eventStore.ts with historicData state per data-model.md (use devs:typescript-dev agent)
- [x] T106 [GIT] Commit: add historic data to event store
- [x] T107 [US3] Implement fetchHistoricData action in EventStore with bearer token auth (use devs:typescript-dev agent)
- [x] T108 [GIT] Commit: implement fetch historic data action
- [x] T109 [US3] Create useHistoricData hook in client/src/hooks/useHistoricData.ts with 5-minute stale-while-revalidate caching per research.md (use devs:typescript-dev agent)
- [x] T110 [GIT] Commit: create useHistoricData hook
- [x] T111 [P] [US3] Create MSW handlers for query edge function in client mocks (use devs:typescript-dev agent)
- [x] T112 [P] [US3] Create unit tests for useHistoricData hook (use devs:typescript-dev agent)
- [x] T113 [GIT] Commit: add client historic data tests
- [x] T114 [US3] Run codebase mapping for Phase 5 changes (/sdd:map incremental)
- [x] T115 [GIT] Commit: update codebase documents for phase 5
- [x] T116 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T117 [GIT] Commit: finalize phase 5 retro

### Phase Completion
- [x] T118 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T119 [GIT] Create/update PR to main with phase summary
- [x] T120 [GIT] Verify all CI checks pass
- [x] T121 [GIT] Report PR ready status

**Checkpoint**: Client fetches and caches historic aggregates; data available in Zustand store

---

## Phase 6: User Story 1 - View Historic Activity Heatmap (Priority: P1)

**Goal**: Display historic activity in the heatmap component, merging with real-time data

**Why this story last**: This is the UI presentation layer that depends on US2 (data exists), US3 (data fetchable), and US4 (secure access)

**Independent Test**: Enable persistence, generate events over multiple days, view heatmap showing both historic and real-time data

### Phase Start
- [x] T122 [GIT] Verify working tree is clean before starting Phase 6
- [x] T123 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T124 [US1] Create retro/P6.md for this phase
- [x] T125 [GIT] Commit: initialize phase 6 retro
- [x] T126 [US1] Create persistence feature detection helper in client/src/utils/persistence.ts (check VITE_SUPABASE_URL) (use devs:typescript-dev agent)
- [x] T127 [GIT] Commit: add persistence feature detection
- [x] T128 [US1] Implement data merging logic in client/src/components/Heatmap.tsx - real-time events in current hour take precedence over historic hourly aggregates (hour-level merge, not event-level) (use devs:react-dev agent)
- [x] T129 [GIT] Commit: implement heatmap data merging
- [x] T130 [US1] Add loading state to Heatmap: "Fetching historic data..." with 5s timeout before showing error (use devs:react-dev agent)
- [x] T131 [GIT] Commit: add heatmap loading state
- [x] T132 [US1] Add error state to Heatmap: "Unable to load historic data. Showing real-time events only." with Retry button; fallback to real-time only (use devs:react-dev agent)
- [x] T133 [GIT] Commit: add heatmap error handling
- [x] T134 [US1] Implement conditional rendering - hide heatmap card entirely when persistence disabled (use devs:react-dev agent)
- [x] T135 [GIT] Commit: implement conditional heatmap visibility
- [x] T136 [US1] Add 7/30 day toggle to heatmap view - queries last N calendar days (or less if insufficient data) (use devs:react-dev agent)
- [x] T137 [GIT] Commit: add day range toggle
- [x] T138 [P] [US1] Create component tests for Heatmap with historic data in client/src/components/Heatmap.test.tsx (use devs:react-dev agent)
- [x] T139 [P] [US1] Create integration test for data merging logic (use devs:typescript-dev agent)
- [ ] T140 [GIT] Commit: add heatmap tests
- [ ] T141 [US1] Run codebase mapping for Phase 6 changes (/sdd:map incremental)
- [ ] T142 [GIT] Commit: update codebase documents for phase 6
- [ ] T143 [US1] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T144 [GIT] Commit: finalize phase 6 retro

### Phase Completion
- [ ] T145 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T146 [GIT] Create/update PR to main with phase summary
- [ ] T147 [GIT] Verify all CI checks pass
- [ ] T148 [GIT] Report PR ready status

**Checkpoint**: Heatmap displays historic data merged with real-time; hidden when persistence disabled

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation validation, contract verification, and final test suite across all user stories

**Dependencies**: ALL previous phases MUST be complete before Phase 7

### Phase Start
- [ ] T149 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T150 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [ ] T151 Create retro/P7.md for this phase
- [ ] T152 [GIT] Commit: initialize phase 7 retro
- [ ] T153 [P] Validate CLAUDE.md contains all persistence environment variables and development notes (add any missing)
- [ ] T154 [P] Validate quickstart.md accuracy against actual implementation (fix any discrepancies)
- [ ] T155 [GIT] Commit: documentation validation fixes
- [ ] T156 Validate contracts/ingest.yaml and contracts/query.yaml match actual edge function implementations (FR-014 compliance)
- [ ] T157 [GIT] Commit: contract validation fixes if needed
- [ ] T158 Run full test suite (cargo test, npm test, deno test) and fix any failures (requires Phase 6 complete)
- [ ] T159 [GIT] Commit: fix any test issues
- [ ] T160 Validate quickstart.md by following steps in clean local environment
- [ ] T161 [GIT] Commit: quickstart validation fixes if needed
- [ ] T162 Run final codebase mapping (/sdd:map incremental)
- [ ] T163 [GIT] Commit: update codebase documents for phase 7
- [ ] T164 Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T165 [GIT] Commit: finalize phase 7 retro

### Phase Completion
- [ ] T166 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T167 [GIT] Create/update PR to main with phase summary
- [ ] T168 [GIT] Verify all CI checks pass
- [ ] T169 [GIT] Report PR ready status

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **US4 Security (Phase 3)**: Depends on Foundational (T023-T024 shared auth) - BLOCKS US2 and US3; includes full edge function implementation with database wiring
- **US2 Monitor (Phase 4)**: Depends on US4 (needs fully functional ingest endpoint)
- **US3 Client Query (Phase 5)**: Depends on US4 (needs fully functional query endpoint)
- **US1 Heatmap UI (Phase 6)**: Depends on US2 + US3 (needs data flowing and queryable)
- **Polish (Phase 7)**: Depends on ALL user stories being complete

### Dependency Graph

```
Phase 1: Setup (T001-T011)
    │
    v
Phase 2: Foundational (T012-T032)
    │   DB schema, types, shared auth utils
    │
    v
Phase 3: US4 - Security (T033-T067)
    │   Edge functions + auth + DB wiring + RLS test
    │
    ├───────────────┬───────────────┐
    v               v               │
Phase 4: US2    Phase 5: US3       │
(T068-T098)     (T099-T121)        │
Monitor         Client Query       │
    │               │               │
    └───────────────┴───────────────┘
                    │
                    v
            Phase 6: US1 (T122-T148)
            Heatmap UI
                    │
                    v
            Phase 7: Polish (T149-T169)
```

### Critical Task Dependencies (blockedBy)

| Task | Depends On | Reason |
|------|------------|--------|
| T037, T038 (edge function scaffolds) | T024 | Imports _shared/auth.ts |
| T051 (ingest DB wiring) | T020 | Uses bulk_insert_events function |
| T053 (query DB wiring) | T019 | Uses get_hourly_aggregates function |
| T158 (full test suite) | Phase 6 complete | Needs all components |

### User Story Independence

Note: While the phases are ordered for a single implementer, the user stories maintain conceptual independence:

- **US4 (Security)**: Can be tested with curl commands alone
- **US2 (Monitor)**: Can be tested by checking database records
- **US3 (Client Query)**: Can be tested with seeded data
- **US1 (Heatmap)**: Can be tested with mocked responses

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
- T018 and T019 (database functions) can run in parallel
- T021, T023 (types and shared utils) can run in parallel after T020 commit

**Within Phase 3 (US4)**:
- T037 and T038 (edge function scaffolds) can run in parallel (both blockedBy T024)
- T042 and T043 (auth implementations) can run in parallel
- T055 and T056 (auth tests) can run in parallel

**Within Phase 4 (US2)**:
- T088 and T089 (persistence tests) can run in parallel

**Within Phase 5 (US3)**:
- T111 and T112 (client tests) can run in parallel

**Within Phase 6 (US1)**:
- T138 and T139 (heatmap tests) can run in parallel

**Within Phase 7 (Polish)**:
- T153 and T154 (documentation validation) can run in parallel

---

## Parallel Example: Phase 2 Foundational

```bash
# After T017 (events table migration), launch functions in parallel:
Task: "Create bulk_insert_events PostgreSQL function in supabase/migrations/..."
Task: "Create get_hourly_aggregates PostgreSQL function in supabase/migrations/..."

# After T020 (functions committed), launch types/utils in parallel:
Task: "Add HourlyAggregate type to client/src/types/events.ts..."
Task: "Create shared Ed25519 signature verification utility..."
```

---

## Implementation Strategy

### MVP First (Phase 1-6)

1. Complete Phase 1: Setup - Supabase initialized
2. Complete Phase 2: Foundational - Schema and shared code ready
3. Complete Phase 3: US4 Security - Edge functions fully functional with auth + DB
4. Complete Phase 4: US2 Monitor - Events flow into Supabase
5. Complete Phase 5: US3 Query - Client can fetch aggregates
6. Complete Phase 6: US1 Heatmap - **MVP COMPLETE** - User sees historic data
7. **STOP and VALIDATE**: Test end-to-end with real monitor

### Success Criteria Mapping

| Success Criteria | Verified By |
|------------------|-------------|
| SC-001: Historic data visible within 5s | Phase 6 heatmap tests (T138-T139) |
| SC-002: 95%+ event persistence | Phase 4 monitor retry tests (T088-T089) |
| SC-003: All access via edge functions | Phase 3 RLS negative test (T058) |
| SC-004: Config via env vars only | All phases (no hardcoded URLs) |
| SC-005: Real-time unaffected by failures | Phase 4 failure isolation tests |
| SC-006: Heatmap hidden when disabled | Phase 6 conditional render test (T134) |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- [GIT] tasks enforce proper git workflow with commits, pushes, and PR verification
- (blockedBy: Txxx) indicates explicit dependency that must complete first
- Each phase has checkpoint criteria before proceeding
- Commit after each task or logical group (batching permitted for parallel [P] tasks)
- Stop at any checkpoint to validate progress
- Edge functions use Deno (TypeScript) - reference devs:typescript-dev agent
- Monitor uses Rust - reference devs:rust-dev agent
- Client uses React/TypeScript - reference devs:react-dev and devs:typescript-dev agents
- Field name transformation: JSON uses eventType (camelCase), SQL uses event_type (snake_case)
