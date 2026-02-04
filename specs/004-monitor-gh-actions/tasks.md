# Tasks: Monitor GitHub Actions Deployment

**Input**: Design documents from `/specs/004-monitor-gh-actions/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Integration tests are included per FR-027/FR-028 to verify round-trip key export → env load → sign → verify.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Monitor**: `monitor/src/` (Rust binary)
- **Tests**: `monitor/tests/` (Rust integration tests)
- **GitHub Action**: `.github/actions/vibetea-monitor/` (composite action)
- **Workflows**: `.github/workflows/` (example CI workflow)
- **Documentation**: `README.md` (GitHub Actions section)

---

## Phase 1: Setup

**Purpose**: Branch and project verification

### Phase Start
- [x] T001 [GIT] Verify on main branch and working tree is clean
- [x] T002 [GIT] Pull latest changes from origin/main
- [x] T003 [GIT] Create feature branch: 004-monitor-gh-actions

### Setup Tasks
- [x] T004 Verify existing monitor crate structure in monitor/src/
- [x] T005 [GIT] Commit: verify project structure

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [x] T006 [GIT] Verify working tree is clean before starting Phase 2
- [x] T007 Create retro/P2.md for this phase

### Foundational Tasks
- [x] T008 [P] Add KeySource enum to monitor/src/crypto.rs (use devs:rust-dev agent)
- [x] T009 [P] Add CryptoError variants if needed in monitor/src/crypto.rs (use devs:rust-dev agent)
- [x] T010 [GIT] Commit: add KeySource enum and error variants
- [x] T011 Add public_key_fingerprint() method to Crypto in monitor/src/crypto.rs (use devs:rust-dev agent)
- [x] T012 [GIT] Commit: add public key fingerprint method

**Checkpoint**: Foundation ready - user story implementation can now begin

### Phase Completion
- [x] T013 Run /sdd:map incremental for Phase 2 changes
- [x] T014 [GIT] Commit: update codebase documents for phase 2
- [x] T015 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T016 [GIT] Commit: finalize phase 2 retro
- [x] T017 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T018 [GIT] Create/update PR to main with phase summary
- [x] T019 [GIT] Verify all CI checks pass
- [x] T020 [GIT] Report PR ready status

---

## Phase 3: User Story 1 - Configure Monitor with Environment Variable Private Key (Priority: P1) 🎯 MVP

**Goal**: Enable loading Ed25519 private key from `VIBETEA_PRIVATE_KEY` environment variable

**Independent Test**: Set `VIBETEA_PRIVATE_KEY` env var, run monitor, verify it authenticates and sends events

### Phase Start
- [x] T021 [GIT] Verify working tree is clean before starting Phase 3
- [x] T022 [GIT] Pull and rebase on origin/main if needed
- [x] T023 [US1] Create retro/P3.md for this phase
- [x] T024 [GIT] Commit: initialize phase 3 retro

### Tests for User Story 1
- [x] T025 [P] [US1] Create env_key_test.rs integration test in monitor/tests/env_key_test.rs (use devs:rust-dev agent)
- [x] T026 [GIT] Commit: add env var key loading tests (expecting failures)

### Implementation for User Story 1
- [x] T027 [US1] Implement load_from_env() method in monitor/src/crypto.rs (use devs:rust-dev agent)
  - Trim whitespace from VIBETEA_PRIVATE_KEY (FR-005)
  - Decode Base64 standard (RFC 4648) (FR-021)
  - Validate 32-byte length (FR-022)
  - Return clear error messages (FR-004)
- [x] T028 [GIT] Commit: implement load_from_env for VIBETEA_PRIVATE_KEY
- [x] T029 [US1] Implement load_with_fallback() method in monitor/src/crypto.rs (use devs:rust-dev agent)
  - Check env var first, fall back to file (FR-002)
  - Return (Crypto, KeySource) tuple
  - Log which source was used (INFO level) (FR-007)
- [x] T030 [GIT] Commit: implement load_with_fallback with precedence logic
- [x] T031 [US1] Update run command in monitor/src/main.rs to use load_with_fallback (use devs:rust-dev agent)
  - Log key source and fingerprint at startup (FR-007)
  - Log INFO if file key ignored (FR-002)
  - Never log private key value (FR-019)
  - Exit code 1 for config errors (FR-026)
- [x] T032 [GIT] Commit: update run command to support env var keys
- [x] T033 [US1] Zero intermediate buffers after SigningKey construction in monitor/src/crypto.rs (FR-020) (use devs:rust-dev agent)
- [x] T034 [GIT] Commit: add memory zeroing for key material
- [x] T035 [US1] Run tests: cargo test -p vibetea-monitor -- --test-threads=1
- [x] T036 [GIT] Commit: all US1 tests passing

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

### Phase Completion
- [x] T037 [US1] Run /sdd:map incremental for Phase 3 changes
- [x] T038 [GIT] Commit: update codebase documents for phase 3
- [x] T039 [US1] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T040 [GIT] Commit: finalize phase 3 retro
- [x] T041 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T042 [GIT] Create/update PR to main with phase summary
- [x] T043 [GIT] Verify all CI checks pass
- [x] T044 [GIT] Report PR ready status

---

## Phase 4: User Story 2 - Export Existing Key for GitHub Actions (Priority: P1) 🎯 MVP

**Goal**: Allow exporting private key in Base64 format for use as GitHub Actions secret

**Independent Test**: Run `export-key` on existing keypair, use output with `VIBETEA_PRIVATE_KEY` to authenticate

### Phase Start
- [x] T045 [GIT] Verify working tree is clean before starting Phase 4
- [x] T046 [GIT] Pull and rebase on origin/main if needed
- [x] T047 [US2] Create retro/P4.md for this phase
- [x] T048 [GIT] Commit: initialize phase 4 retro

### Tests for User Story 2
- [x] T049 [P] [US2] Create key_export_test.rs integration test in monitor/tests/key_export_test.rs (use devs:rust-dev agent)
  - Test round-trip: export → env load → sign → verify (FR-027, FR-028)
  - Test output format: base64 + single newline (FR-003)
  - Test errors to stderr (FR-023)
- [x] T050 [GIT] Commit: add export-key tests (expecting failures)

### Implementation for User Story 2
- [x] T051 [US2] Implement export_key_base64() method in monitor/src/crypto.rs (use devs:rust-dev agent)
  - Output ONLY base64 key + newline to stdout (FR-003)
  - Use Base64 standard (RFC 4648) (FR-021)
- [x] T052 [GIT] Commit: implement export_key_base64 method
- [x] T053 [US2] Add export-key command parsing in monitor/src/main.rs (use devs:rust-dev agent)
  - Support --path/-p option for custom key location
  - All diagnostic messages to stderr (FR-023)
  - Exit code 1 for missing key (FR-026)
  - Exit code 2 for I/O errors (FR-026)
- [x] T054 [GIT] Commit: add export-key command CLI parsing
- [x] T055 [US2] Implement export-key command handler in monitor/src/main.rs (use devs:rust-dev agent)
  - Load key from path or default ~/.vibetea
  - Output base64 to stdout
  - Error: "No key found at {path}/key.priv\nRun 'vibetea-monitor init' first."
- [x] T056 [GIT] Commit: implement export-key command handler
- [x] T057 [US2] Run tests: cargo test -p vibetea-monitor -- --test-threads=1
- [x] T058 [GIT] Commit: all US2 tests passing

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

### Phase Completion
- [x] T059 [US2] Run /sdd:map incremental for Phase 4 changes
- [x] T060 [GIT] Commit: update codebase documents for phase 4
- [x] T061 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T062 [GIT] Commit: finalize phase 4 retro
- [x] T063 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T064 [GIT] Create/update PR to main with phase summary
- [x] T065 [GIT] Verify all CI checks pass
- [x] T066 [GIT] Report PR ready status

---

## Phase 5: User Story 3 - Run Monitor in GitHub Actions Workflow (Priority: P2)

**Goal**: Demonstrate monitor deployment pattern in GitHub Actions workflows

**Independent Test**: Run workflow that starts monitor, verify events appear on VibeTea server

### Phase Start
- [x] T067 [GIT] Verify working tree is clean before starting Phase 5
- [x] T068 [GIT] Pull and rebase on origin/main if needed
- [x] T069 [US3] Create retro/P5.md for this phase
- [x] T070 [GIT] Commit: initialize phase 5 retro

### Implementation for User Story 3
- [x] T071 [P] [US3] Add "GitHub Actions Setup" section to README.md (FR-015, FR-016, FR-017, FR-018)
  - Step-by-step setup instructions
  - How to export existing keys
  - Example workflow snippet
  - Required secrets and environment variables
- [x] T072 [GIT] Commit: add GitHub Actions documentation to README
- [x] T073 [US3] Create example CI workflow in .github/workflows/ci-with-monitor.yml
  - Download monitor binary from releases
  - Configure env vars from secrets (VIBETEA_PRIVATE_KEY, VIBETEA_SERVER_URL)
  - Start monitor in background
  - Use VIBETEA_SOURCE_ID with github context (FR-012, FR-013, FR-014)
- [x] T074 [GIT] Commit: add example CI workflow with monitor

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently

### Phase Completion
- [x] T075 [US3] Run /sdd:map incremental for Phase 5 changes
- [x] T076 [GIT] Commit: update codebase documents for phase 5
- [x] T077 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T078 [GIT] Commit: finalize phase 5 retro
- [x] T079 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T080 [GIT] Create/update PR to main with phase summary
- [x] T081 [GIT] Verify all CI checks pass
- [x] T082 [GIT] Report PR ready status

---

## Phase 6: User Story 4 - Reusable GitHub Action (Priority: P3)

**Goal**: Pre-built GitHub Action for easy monitor setup in workflows

**Independent Test**: Use action in workflow with server-url and private-key inputs, verify events are captured

### Phase Start
- [x] T083 [GIT] Verify working tree is clean before starting Phase 6
- [x] T084 [GIT] Pull and rebase on origin/main if needed
- [x] T085 [US4] Create retro/P6.md for this phase
- [x] T086 [GIT] Commit: initialize phase 6 retro

### Implementation for User Story 4
- [x] T087 [US4] Create composite action in .github/actions/vibetea-monitor/action.yml
  - Required inputs: server-url, private-key (FR-008)
  - Optional inputs: source-id, version (FR-009)
  - Download binary step
  - Start monitor in background step
  - Output monitor-pid
  - Graceful shutdown via SIGTERM (FR-010)
- [ ] T088 [GIT] Commit: create vibetea-monitor composite action
- [ ] T089 [US4] Update README.md with action usage examples
  - Basic usage
  - Custom source ID
  - Pinned version
- [ ] T090 [GIT] Commit: add action usage documentation to README

**Checkpoint**: All user stories should now be independently functional

### Phase Completion
- [ ] T091 [US4] Run /sdd:map incremental for Phase 6 changes
- [ ] T092 [GIT] Commit: update codebase documents for phase 6
- [ ] T093 [US4] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T094 [GIT] Commit: finalize phase 6 retro
- [ ] T095 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T096 [GIT] Create/update PR to main with phase summary
- [ ] T097 [GIT] Verify all CI checks pass
- [ ] T098 [GIT] Report PR ready status

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

### Phase Start
- [ ] T099 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T100 [GIT] Pull and rebase on origin/main if needed
- [ ] T101 Create retro/P7.md for this phase
- [ ] T102 [GIT] Commit: initialize phase 7 retro

### Polish Tasks
- [ ] T103 [P] Run cargo clippy -p vibetea-monitor and fix warnings
- [ ] T104 [P] Run cargo fmt -p vibetea-monitor
- [ ] T105 [GIT] Commit: lint and format cleanup
- [ ] T106 Run full test suite: cargo test -p vibetea-monitor -- --test-threads=1
- [ ] T107 [GIT] Commit: verify all tests pass
- [ ] T108 Validate quickstart.md scenarios manually
- [ ] T109 [GIT] Commit: quickstart validation complete

### Phase Completion
- [ ] T110 Run /sdd:map incremental for Phase 7 changes
- [ ] T111 [GIT] Commit: final codebase document update
- [ ] T112 Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T113 [GIT] Commit: finalize phase 7 retro
- [ ] T114 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T115 [GIT] Create/update PR to main with final summary
- [ ] T116 [GIT] Verify all CI checks pass
- [ ] T117 [GIT] Report PR ready status

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) → Phase 2 (Foundational) → Phase 3-6 (User Stories) → Phase 7 (Polish)
                         ↓
              ┌──────────┴──────────┬──────────────┐
              ↓                     ↓              ↓
         Phase 3 (US1)         Phase 4 (US2)  [wait for US1/US2]
         Env Var Key           Export Key           ↓
              ↓                     ↓          Phase 5 (US3)
              └─────────────────────┴───────→  Workflow Demo
                                                   ↓
                                              Phase 6 (US4)
                                              GitHub Action
```

### User Story Dependencies

| Story | Depends On | Can Start After |
|-------|------------|-----------------|
| US1 (P1) | Foundational | Phase 2 complete |
| US2 (P1) | Foundational | Phase 2 complete |
| US3 (P2) | US1, US2 | Phase 4 complete (needs export + env var) |
| US4 (P3) | US3 | Phase 5 complete (needs workflow pattern) |

### Within Each User Story

1. Tests written first (if included) - must FAIL before implementation
2. Core crypto methods before CLI integration
3. CLI parsing before command handlers
4. Implementation before test verification
5. Story complete before moving to next priority

### Parallel Opportunities

**Phase 2 (Foundational)**:
```bash
# Can run in parallel:
T008: Add KeySource enum
T009: Add CryptoError variants
```

**Phase 3 (US1)** - Models/methods can be parallel:
```bash
# Sequential due to dependencies:
load_from_env → load_with_fallback → main.rs update
```

**Phase 4 (US2)**:
```bash
# Sequential:
export_key_base64 → CLI parsing → handler
```

**Phase 7 (Polish)**:
```bash
# Can run in parallel:
T103: clippy
T104: fmt
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (env var key loading)
4. Complete Phase 4: User Story 2 (key export)
5. **STOP and VALIDATE**: Test round-trip: init → export → env var load → sign
6. Deploy/demo if ready - users can now use monitor in CI

### Incremental Delivery

1. **After US1 + US2**: Core functionality ready, manual CI setup possible
2. **After US3**: Documentation and example workflow available
3. **After US4**: Zero-config GitHub Action for easy adoption
4. Each story adds convenience without breaking previous stories

---

## Task Summary

| Phase | Story | Task Count | Parallel Tasks |
|-------|-------|------------|----------------|
| 1 | Setup | 5 | 0 |
| 2 | Foundational | 15 | 2 |
| 3 | US1 | 24 | 1 |
| 4 | US2 | 22 | 1 |
| 5 | US3 | 16 | 1 |
| 6 | US4 | 16 | 0 |
| 7 | Polish | 19 | 2 |
| **Total** | | **117** | **7** |

### MVP Scope (Recommended)

Complete through Phase 4 (US1 + US2) = **66 tasks**

This delivers:
- ✅ Load private key from environment variable
- ✅ Export existing key for GitHub Actions secrets
- ✅ Round-trip tested: export → env load → sign → verify

---

## Notes

- All Rust tasks should use `devs:rust-dev` agent
- Run tests with `--test-threads=1` due to env var modifications (per CLAUDE.md)
- Never log private key values - log presence/absence only (FR-019)
- Memory zeroing required after key construction (FR-020)
- Exit codes: 0 success, 1 config error, 2 runtime error (FR-026)
- Non-blocking monitoring: workflow succeeds even if events fail to send (FR-029)
