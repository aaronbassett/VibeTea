# Tasks: Supabase Authentication

**Input**: Design documents from `/specs/002-supabase-auth/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are NOT included in this task list. Add test tasks if TDD approach is explicitly requested.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- **[GIT]**: Git workflow task
- Include exact file paths in descriptions

## Path Conventions

- **Server**: `server/src/` (Rust)
- **Client**: `client/src/` (TypeScript/React)
- **Supabase**: `supabase/functions/`, `supabase/migrations/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, Supabase configuration, and basic structure

### Phase Start
- [ ] T001 [GIT] Verify on feature branch 002-supabase-auth and working tree is clean
- [ ] T002 [GIT] Pull and rebase on origin/main if needed

### Setup Tasks
- [ ] T003 [P] Add @supabase/supabase-js dependency to client in client/package.json
- [ ] T004 [P] Create Supabase client service in client/src/services/supabase.ts (use devs:typescript-dev agent)
- [ ] T005 [P] Add Supabase environment variables to server/src/config.rs (use devs:rust-dev agent)
- [ ] T006 [P] Create .env.example files with required Supabase variables for server and client
- [ ] T007 [GIT] Commit: setup Supabase client and configuration

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [ ] T008 [GIT] Verify working tree is clean before starting Phase 2
- [ ] T009 Create retro/P2.md for this phase

### Supabase Infrastructure
- [ ] T010 Create Supabase migration for monitor_public_keys table in supabase/migrations/001_public_keys.sql
- [ ] T011 [GIT] Commit: add public_keys migration
- [ ] T012 Create Supabase edge function for public keys in supabase/functions/public-keys/index.ts (use devs:typescript-dev agent)
- [ ] T013 [GIT] Commit: add public-keys edge function

### Server Session Infrastructure
- [ ] T014 [P] Create session store module in server/src/session.rs (use devs:rust-dev agent)
- [ ] T015 [GIT] Commit: add session store module
- [ ] T016 [P] Create Supabase client module in server/src/supabase.rs (use devs:rust-dev agent)
- [ ] T017 [GIT] Commit: add Supabase client module
- [ ] T018 [P] Add auth error types to server/src/error.rs (use devs:rust-dev agent)
- [ ] T019 [GIT] Commit: add auth error types

### Privacy Compliance (Constitution I)
- [ ] T019a Create privacy test in server/tests/auth_privacy_test.rs verifying no JWTs or session tokens appear in logs (use devs:rust-dev agent)
- [ ] T019b [GIT] Commit: add auth privacy test

### Phase Completion
- [ ] T020 Run /sdd:map incremental for Phase 2 changes
- [ ] T021 [GIT] Commit: update codebase documents for phase 2
- [ ] T022 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T023 [GIT] Commit: finalize phase 2 retro
- [ ] T024 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T025 [GIT] Create/update PR to main with phase summary
- [ ] T026 [GIT] Verify all CI checks pass
- [ ] T027 [GIT] Report PR ready status

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Client GitHub Authentication (Priority: P1) 🎯 MVP

**Goal**: Allow users to authenticate via GitHub OAuth using Supabase

**Independent Test**: Log in with GitHub, receive a session token, and connect to the WebSocket. Delivers secure individual user authentication.

### Phase Start
- [ ] T028 [GIT] Verify working tree is clean before starting Phase 3
- [ ] T029 [GIT] Pull and rebase on origin/main if needed
- [ ] T030 [US1] Create retro/P3.md for this phase

### Client Auth Implementation
- [ ] T031 [P] [US1] Create useAuth hook in client/src/hooks/useAuth.ts (use devs:typescript-dev agent)
- [ ] T032 [P] [US1] Create Login page component in client/src/pages/Login.tsx (use devs:react-dev agent)
- [ ] T033 [P] [US1] Create Dashboard page component in client/src/pages/Dashboard.tsx (use devs:react-dev agent)
- [ ] T034 [GIT] Commit: add auth hook and page components
- [ ] T035 [US1] Add auth routing to client/src/App.tsx (use devs:react-dev agent) **[Depends on T031, T032]**
- [ ] T036 [GIT] Commit: add auth routing
- [ ] T037 [US1] Update client/src/components/TokenForm.tsx to use Supabase session (use devs:react-dev agent)
- [ ] T038 [GIT] Commit: update TokenForm for Supabase

### Phase Completion
- [ ] T039 [US1] Run /sdd:map incremental for Phase 3 changes
- [ ] T040 [GIT] Commit: update codebase documents for phase 3
- [ ] T041 [US1] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T042 [GIT] Commit: finalize phase 3 retro
- [ ] T043 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T044 [GIT] Create/update PR to main with phase summary
- [ ] T045 [GIT] Verify all CI checks pass
- [ ] T046 [GIT] Report PR ready status

**Checkpoint**: User Story 1 complete - users can authenticate via GitHub OAuth

---

## Phase 4: User Story 2 - Server Session Token Exchange (Priority: P1)

**Goal**: Server validates Supabase JWTs and issues session tokens for WebSocket access

**Independent Test**: Call the session endpoint with a valid Supabase JWT and verify a session token is returned that works for WebSocket connection.

### Phase Start
- [ ] T047 [GIT] Verify working tree is clean before starting Phase 4
- [ ] T048 [GIT] Pull and rebase on origin/main if needed
- [ ] T049 [US2] Create retro/P4.md for this phase

### Server Auth Implementation
- [ ] T050 [P] [US2] Add JWT validation via Supabase API in server/src/auth.rs (use devs:rust-dev agent)
- [ ] T051 [GIT] Commit: add Supabase JWT validation
- [ ] T052 [P] [US2] Add session token generation to server/src/session.rs (use devs:rust-dev agent)
- [ ] T053 [GIT] Commit: add session token generation
- [ ] T054 [US2] Add POST /auth/session endpoint to server/src/routes.rs (use devs:rust-dev agent)
- [ ] T055 [GIT] Commit: add session endpoint
- [ ] T056 [US2] Update GET /ws to validate session tokens in server/src/routes.rs (use devs:rust-dev agent)
- [ ] T057 [GIT] Commit: update WebSocket auth
- [ ] T058 [US2] Add session cleanup task to server/src/main.rs (use devs:rust-dev agent)
- [ ] T059 [GIT] Commit: add session cleanup task

### Client Session Token Integration
- [ ] T060 [US2] Update client/src/hooks/useAuth.ts to exchange JWT for session token (use devs:typescript-dev agent) **[Depends on Phase 3 T031]**
- [ ] T061 [GIT] Commit: add session token exchange to client
- [ ] T062 [US2] Update client/src/hooks/useWebSocket.ts to use session token (use devs:typescript-dev agent)
- [ ] T063 [GIT] Commit: update WebSocket to use session token

### Phase Completion
- [ ] T064 [US2] Run /sdd:map incremental for Phase 4 changes
- [ ] T065 [GIT] Commit: update codebase documents for phase 4
- [ ] T066 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T067 [GIT] Commit: finalize phase 4 retro
- [ ] T068 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T069 [GIT] Create/update PR to main with phase summary
- [ ] T070 [GIT] Verify all CI checks pass
- [ ] T071 [GIT] Report PR ready status

**Checkpoint**: User Story 2 complete - JWT exchange and session tokens working

---

## Phase 5: User Story 3 - Client Reconnection with Token Refresh (Priority: P2)

**Goal**: Automatic reconnection and token refresh when WebSocket disconnects

**Independent Test**: Simulate WebSocket disconnect and verify automatic reconnection with existing token, then simulate server restart and verify automatic token refresh.

### Phase Start
- [ ] T072 [GIT] Verify working tree is clean before starting Phase 5
- [ ] T073 [GIT] Pull and rebase on origin/main if needed
- [ ] T074 [US3] Create retro/P5.md for this phase

### Client Reconnection Logic
- [ ] T075 [P] [US3] Add token refresh logic on 401 to client/src/hooks/useWebSocket.ts (use devs:typescript-dev agent)
- [ ] T076 [GIT] Commit: add token refresh on 401
- [ ] T077 [P] [US3] Add session expiry detection to client/src/hooks/useAuth.ts (use devs:typescript-dev agent)
- [ ] T078 [GIT] Commit: add session expiry detection
- [ ] T079 [US3] Add redirect to login on expired Supabase session in client/src/App.tsx (use devs:react-dev agent)
- [ ] T080 [GIT] Commit: add login redirect on session expiry

### Phase Completion
- [ ] T081 [US3] Run /sdd:map incremental for Phase 5 changes
- [ ] T082 [GIT] Commit: update codebase documents for phase 5
- [ ] T083 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T084 [GIT] Commit: finalize phase 5 retro
- [ ] T085 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T086 [GIT] Create/update PR to main with phase summary
- [ ] T087 [GIT] Verify all CI checks pass
- [ ] T088 [GIT] Report PR ready status

**Checkpoint**: User Story 3 complete - automatic reconnection and token refresh working

---

## Phase 6: User Story 4 - Monitor Public Key Management via Supabase (Priority: P2)

**Goal**: Manage monitor public keys in Supabase instead of environment variables

**Independent Test**: Add a public key to the Supabase table and verify the server accepts events signed by that key within 30 seconds.

### Phase Start
- [ ] T089 [GIT] Verify working tree is clean before starting Phase 6
- [ ] T090 [GIT] Pull and rebase on origin/main if needed
- [ ] T091 [US4] Create retro/P6.md for this phase

### Server Public Key Refresh
- [ ] T092 [US4] Add public key fetching to server/src/supabase.rs (use devs:rust-dev agent)
- [ ] T093 [GIT] Commit: add public key fetching
- [ ] T094 [US4] Add public key cache with periodic refresh to server/src/auth.rs (use devs:rust-dev agent)
- [ ] T095 [GIT] Commit: add public key cache
- [ ] T096 [P] [US4] Add public key refresh task to server/src/main.rs (use devs:rust-dev agent) **[Depends on T094]**
- [ ] T097 [GIT] Commit: add public key refresh task
- [ ] T098 [P] [US4] Update signature verification to use cached keys in server/src/auth.rs (use devs:rust-dev agent) **[Depends on T094]**
- [ ] T099 [GIT] Commit: use cached keys for signature verification

### Phase Completion
- [ ] T100 [US4] Run /sdd:map incremental for Phase 6 changes
- [ ] T101 [GIT] Commit: update codebase documents for phase 6
- [ ] T102 [US4] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T103 [GIT] Commit: finalize phase 6 retro
- [ ] T104 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T105 [GIT] Create/update PR to main with phase summary
- [ ] T106 [GIT] Verify all CI checks pass
- [ ] T107 [GIT] Report PR ready status

**Checkpoint**: User Story 4 complete - public keys managed via Supabase

---

## Phase 7: User Story 5 - Server Startup Resilience (Priority: P3)

**Goal**: Handle transient network errors on server startup

**Independent Test**: Start the server with Supabase temporarily unavailable and verify retry behavior.

### Phase Start
- [ ] T108 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T109 [GIT] Pull and rebase on origin/main if needed
- [ ] T110 [US5] Create retro/P7.md for this phase

### Server Startup Retry
- [ ] T111 [US5] Add startup retry logic with exponential backoff to server/src/supabase.rs (use devs:rust-dev agent)
- [ ] T112 [GIT] Commit: add startup retry logic
- [ ] T113 [US5] Add startup health check to server/src/main.rs (use devs:rust-dev agent)
- [ ] T114 [GIT] Commit: add startup health check
- [ ] T115 [US5] Update config validation in server/src/config.rs for required Supabase variables (use devs:rust-dev agent)
- [ ] T116 [GIT] Commit: add Supabase config validation

### Phase Completion
- [ ] T117 [US5] Run /sdd:map incremental for Phase 7 changes
- [ ] T118 [GIT] Commit: update codebase documents for phase 7
- [ ] T119 [US5] Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T120 [GIT] Commit: finalize phase 7 retro
- [ ] T121 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T122 [GIT] Create/update PR to main with phase summary
- [ ] T123 [GIT] Verify all CI checks pass
- [ ] T124 [GIT] Report PR ready status

**Checkpoint**: User Story 5 complete - server handles startup failures gracefully

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, cleanup, and final validation

### Phase Start
- [ ] T125 [GIT] Verify working tree is clean before starting Phase 8
- [ ] T126 [GIT] Pull and rebase on origin/main if needed
- [ ] T127 Create retro/P8.md for this phase

### Documentation
- [ ] T128 [P] Update README.md with Supabase setup instructions
- [ ] T129 [P] Update CLAUDE.md with authentication patterns learned
- [ ] T130 [GIT] Commit: update documentation

### Final Validation
- [ ] T131 Run quickstart.md validation steps
- [ ] T132 [GIT] Commit: any fixes from quickstart validation
- [ ] T133 Verify all environment variables documented in .env.example
- [ ] T134 [GIT] Commit: finalize environment documentation

### CI/CD Validation (DS-004, DS-006)
- [ ] T134a Update .github/workflows to include tests for auth endpoints (POST /auth/session, GET /ws with token)
- [ ] T134b Verify CI tests both authenticated and unauthenticated request paths
- [ ] T134c [GIT] Commit: add CI workflow for auth endpoints

### Phase Completion
- [ ] T135 Run /sdd:map incremental for Phase 8 changes
- [ ] T136 [GIT] Commit: update codebase documents for phase 8
- [ ] T137 Review retro/P8.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T138 [GIT] Commit: finalize phase 8 retro
- [ ] T139 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T140 [GIT] Create/update PR to main with phase summary
- [ ] T141 [GIT] Verify all CI checks pass
- [ ] T142 [GIT] Report PR ready status

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational (Phase 2)
- **User Story 2 (Phase 4)**: Depends on Foundational (Phase 2), benefits from US1 completion
- **User Story 3 (Phase 5)**: Depends on User Story 1 and User Story 2
- **User Story 4 (Phase 6)**: Depends on Foundational (Phase 2), independent of client auth stories
- **User Story 5 (Phase 7)**: Depends on User Story 4 (public key infrastructure)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Client-side only
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - Server-side, integrates with US1
- **User Story 3 (P2)**: Depends on US1 and US2 - Reconnection requires both client and server auth
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) - Independent of client auth
- **User Story 5 (P3)**: Depends on US4 - Startup resilience requires public key infrastructure

### Parallel Opportunities

**Within Phase 1 (Setup)**:
- T003, T004, T005, T006 can run in parallel (different files)

**Within Phase 2 (Foundational)**:
- T010-T013 (Supabase infrastructure) can run in parallel with T014, T016, T018 (Server session infrastructure) after migration is applied
- T014, T016, T018 marked [P] - can run in parallel (different files, no dependencies)

**Within Phase 3 (US1)**:
- T031, T032, T033 can run in parallel (different files)
- T035 depends on T031, T032 completion

**Within Phase 4 (US2)**:
- T050, T052 marked [P] - can run in parallel (JWT validation and token generation are independent)

**Within Phase 5 (US3)**:
- T075, T077 marked [P] - can run in parallel (different hooks, no shared dependencies)

**Within Phase 6 (US4)**:
- T096, T098 marked [P] - can run in parallel after T094 completes

**Across User Stories**:
- US1 (Phase 3) and US4 (Phase 6) can run in parallel after Foundational completes
- US4 and US5 are independent of US1, US2, US3

---

## Parallel Example: Phase 3 (User Story 1)

```bash
# Launch all client components for User Story 1 together:
Task: "Create useAuth hook in client/src/hooks/useAuth.ts"
Task: "Create Login page component in client/src/pages/Login.tsx"
Task: "Create Dashboard page component in client/src/pages/Dashboard.tsx"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Client GitHub Authentication)
4. Complete Phase 4: User Story 2 (Server Session Token Exchange)
5. **STOP and VALIDATE**: Test full auth flow independently
6. Deploy/demo if ready - clients can now authenticate and connect

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test client OAuth → (Partial - no server integration yet)
3. Add User Story 2 → Test full auth flow → Deploy/Demo (MVP!)
4. Add User Story 3 → Test reconnection → Deploy/Demo
5. Add User Story 4 → Test public key management → Deploy/Demo
6. Add User Story 5 → Test startup resilience → Deploy/Demo
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 + 2 (client + server auth)
   - Developer B: User Story 4 + 5 (public key management)
3. Developer A completes auth stories, Developer B integrates
4. Team completes User Story 3 together (requires both auth and server)

---

## Summary

| Phase | Tasks | Key Deliverables |
|-------|-------|------------------|
| Phase 1: Setup | T001-T007 | Supabase client, config, env vars |
| Phase 2: Foundational | T008-T027 (+ T019a-b) | Migration, edge function, session store, Supabase client, privacy test |
| Phase 3: US1 | T028-T046 | Client GitHub OAuth, Login/Dashboard pages |
| Phase 4: US2 | T047-T071 | Session endpoint, JWT validation, WebSocket auth |
| Phase 5: US3 | T072-T088 | Token refresh, reconnection logic |
| Phase 6: US4 | T089-T107 | Public key cache, periodic refresh |
| Phase 7: US5 | T108-T124 | Startup retry, health checks |
| Phase 8: Polish | T125-T142 (+ T134a-c) | Documentation, validation, CI/CD auth tests |

**Total Tasks**: 147 (142 original + 2 privacy + 3 CI/CD)
**Tasks per User Story**: US1=16, US2=22, US3=14, US4=16, US5=14
**Parallel Opportunities**: Setup (4), Foundational (varies), US1 models (3)
**Suggested MVP Scope**: Phases 1-4 (Setup + Foundational + US1 + US2)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- [GIT] tasks enforce standard GitHub workflow
- Each user story should be independently completable and testable
- Commit after each implementation task or logical group
- Stop at any checkpoint to validate story independently
- Server tests must run with `--test-threads=1` per CLAUDE.md
