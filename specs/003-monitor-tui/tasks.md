# Tasks: Monitor TUI Interface

**Input**: Design documents from `/specs/003-monitor-tui/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Tests are NOT explicitly requested in the feature specification. Tasks focus on implementation.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- **[GIT]**: Git workflow tasks (commits, PRs, CI verification)
- Include exact file paths in descriptions

## Path Conventions

- **Rust monitor binary**: `monitor/src/`
- **TUI module**: `monitor/src/tui/`
- **Widgets**: `monitor/src/tui/widgets/`
- **Tests**: `monitor/tests/`
- **Retros**: `specs/003-monitor-tui/retro/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, dependencies, and basic structure

### Git Workflow Start
- [x] T001 [GIT] Verify on main branch and working tree is clean
- [x] T002 [GIT] Pull latest changes from origin/main
- [x] T003 [GIT] Create feature branch: 003-monitor-tui

### Setup Tasks
- [x] T004 Add ratatui and crossterm dependencies to monitor/Cargo.toml (use devs:rust-dev agent)
- [x] T005 [GIT] Commit: add TUI dependencies
- [x] T006 [P] Create TUI module structure in monitor/src/tui/mod.rs (use devs:rust-dev agent)
- [x] T007 [P] Create widgets module structure in monitor/src/tui/widgets/mod.rs (use devs:rust-dev agent)
- [x] T008 [GIT] Commit: scaffold TUI module structure
- [x] T009 Update monitor/src/lib.rs to export tui module (use devs:rust-dev agent)
- [x] T010 [GIT] Commit: export TUI module from lib.rs
- [x] T011 Verify cargo build compiles with new structure

### Phase Completion
- [x] T012 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T013 [GIT] Create/update PR to main with phase summary
- [x] T014 [GIT] Verify all CI checks pass
- [x] T015 [GIT] Report PR ready status

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [x] T016 [GIT] Verify working tree is clean before starting Phase 2
- [x] T017 [GIT] Pull and rebase on origin/main if needed
- [x] T018 Create retro/P2.md for this phase

### Terminal Infrastructure
- [x] T019 [GIT] Commit: initialize phase 2 retro
- [x] T020 Implement Tui struct with RAII terminal restoration in monitor/src/tui/terminal.rs (use devs:rust-dev agent)
- [x] T021 [GIT] Commit: add terminal wrapper with RAII restoration
- [x] T022 Implement panic hook for terminal restoration in monitor/src/tui/terminal.rs (use devs:rust-dev agent)
- [x] T023 [GIT] Commit: add panic hook for terminal safety

### Event Handler Infrastructure
- [x] T024 Implement TuiEvent enum in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T025 [GIT] Commit: add TuiEvent enum
- [x] T026 Implement EventHandler with tokio::select! pattern in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T027 [GIT] Commit: add EventHandler with async event loop

### State Types
- [x] T028 [P] Implement AppState struct in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T029 [P] Implement Screen enum (Setup, Dashboard) in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T030 [GIT] Commit: add AppState and Screen types
- [x] T031 [P] Implement Theme struct with default and monochrome variants in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T032 [P] Implement Symbols struct with unicode and ascii variants in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T033 [GIT] Commit: add Theme and Symbols configuration

### Sender Metrics Extension
- [x] T034 Add SenderMetrics struct to monitor/src/sender.rs (use devs:rust-dev agent)
- [x] T035 [GIT] Commit: add SenderMetrics struct
- [x] T036 Add metrics() method and counters to Sender in monitor/src/sender.rs (use devs:rust-dev agent)
- [x] T037 [GIT] Commit: implement sender metrics tracking

### Error Types
- [x] T038 Add TuiError and SetupError types to monitor/src/error.rs (use devs:rust-dev agent)
- [x] T039 [GIT] Commit: add TUI error types

### Codebase Mapping and Retro
- [x] T040 Run /sdd:map incremental for Phase 2 changes
- [x] T041 [GIT] Commit: update codebase documents for phase 2
- [x] T042 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T043 [GIT] Commit: finalize phase 2 retro

### Phase Completion
- [x] T044 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T045 [GIT] Create/update PR to main with phase summary
- [x] T046 [GIT] Verify all CI checks pass
- [x] T047 [GIT] Report PR ready status

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Quick Start with Defaults (Priority: P1) MVP

**Goal**: Users can start monitoring immediately by pressing Enter through the setup form

**Independent Test**: Launch monitor, press Enter through all prompts, verify main dashboard displays with streaming logs

### Phase Start
- [x] T048 [GIT] Verify working tree is clean before starting Phase 3
- [x] T049 [GIT] Pull and rebase on origin/main if needed
- [x] T050 [US1] Create retro/P3.md for this phase
- [x] T051 [GIT] Commit: initialize phase 3 retro

### Setup Form State
- [x] T052 [P] [US1] Implement SetupFormState struct in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T053 [P] [US1] Implement SetupField enum in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T054 [P] [US1] Implement KeyOption enum with toggle() method in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T055 [GIT] Commit: add setup form state types

### Setup Form Widget
- [x] T056 [US1] Implement setup_form widget in monitor/src/tui/widgets/setup_form.rs (use devs:rust-dev agent)
- [x] T057 [GIT] Commit: add setup form widget
- [x] T058 [US1] Add session name validation per FR-026 in monitor/src/tui/widgets/setup_form.rs (use devs:rust-dev agent)
- [x] T059 [GIT] Commit: add session name validation

### Setup Input Handling
- [x] T060 [US1] Implement setup form input handling in monitor/src/tui/input.rs (use devs:rust-dev agent)
- [x] T061 [GIT] Commit: add setup form input handling
- [x] T062 [US1] Implement Tab/Enter navigation with form submission in monitor/src/tui/input.rs (use devs:rust-dev agent)
- [x] T063 [GIT] Commit: add form navigation

### Setup Screen Rendering
- [x] T064 [US1] Implement setup screen layout in monitor/src/tui/ui.rs (use devs:rust-dev agent)
- [x] T065 [GIT] Commit: add setup screen rendering

### Setup-to-Dashboard Transition
- [x] T066 [US1] Implement complete_setup() transition in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T067 [GIT] Commit: add setup to dashboard transition
- [x] T068 [US1] Load existing keys or generate new keys on form submit in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T069 [GIT] Commit: integrate key generation in setup flow

### Default Values
- [x] T070 [US1] Implement hostname detection for default session name in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T071 [GIT] Commit: add hostname default for session name
- [x] T072 [US1] Auto-detect existing keys and set default key option in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T073 [GIT] Commit: auto-detect existing keys

### Codebase Mapping and Retro
- [x] T074 [US1] Run /sdd:map incremental for Phase 3 changes
- [x] T075 [GIT] Commit: update codebase documents for phase 3
- [x] T076 [US1] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T077 [GIT] Commit: finalize phase 3 retro

### Phase Completion
- [x] T078 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T079 [GIT] Create/update PR to main with phase summary
- [x] T080 [GIT] Verify all CI checks pass
- [x] T081 [GIT] Report PR ready status

**Checkpoint**: User Story 1 complete - users can quick-start with defaults

---

## Phase 4: User Story 2 - View Real-Time Event Stream (Priority: P1) MVP

**Goal**: Users see a live stream of events scrolling in the main body

**Independent Test**: Start monitor, trigger Claude Code activity, verify events appear in stream within 2 seconds

### Phase Start
- [x] T082 [GIT] Verify working tree is clean before starting Phase 4
- [x] T083 [GIT] Pull and rebase on origin/main if needed
- [x] T084 [US2] Create retro/P4.md for this phase
- [x] T085 [GIT] Commit: initialize phase 4 retro

### Display Event Types
- [x] T086 [P] [US2] Implement DisplayEvent struct in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T087 [P] [US2] Implement DisplayEventType enum with From trait in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T088 [GIT] Commit: add display event types

### Event Buffer
- [x] T089 [US2] Implement EventBuffer with VecDeque and FIFO eviction in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T090 [GIT] Commit: add bounded event buffer

### Event Stream Widget
- [x] T091 [US2] Implement event_stream widget in monitor/src/tui/widgets/event_stream.rs (use devs:rust-dev agent)
- [x] T092 [GIT] Commit: add event stream widget
- [x] T093 [US2] Add timestamp formatting (HH:MM:SS) in monitor/src/tui/widgets/event_stream.rs (use devs:rust-dev agent)
- [x] T094 [GIT] Commit: add timestamp formatting
- [x] T095 [US2] Add event type icons (unicode and ascii) in monitor/src/tui/widgets/event_stream.rs (use devs:rust-dev agent)
- [x] T096 [GIT] Commit: add event type icons

### Scroll State
- [x] T097 [US2] Implement ScrollState struct in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T098 [GIT] Commit: add scroll state
- [x] T099 [US2] Implement auto-scroll behavior (pause on manual scroll, resume on scroll-to-bottom) in monitor/src/tui/input.rs (use devs:rust-dev agent)
- [x] T100 [GIT] Commit: implement auto-scroll behavior

### Event Integration
- [x] T101 [US2] Integrate watcher events into display buffer in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T102 [GIT] Commit: integrate watcher events into TUI
- [x] T103 [US2] Handle terminal resize for event stream in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T104 [GIT] Commit: handle resize for event stream

### Codebase Mapping and Retro
- [x] T105 [US2] Run /sdd:map incremental for Phase 4 changes
- [x] T106 [GIT] Commit: update codebase documents for phase 4
- [x] T107 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [x] T108 [GIT] Commit: finalize phase 4 retro

### Phase Completion
- [x] T109 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T110 [GIT] Create/update PR to main with phase summary
- [x] T111 [GIT] Verify all CI checks pass
- [x] T112 [GIT] Report PR ready status

**Checkpoint**: User Story 2 complete - real-time event streaming functional

---

## Phase 5: User Story 3 - Monitor Server Connection Status (Priority: P1) MVP

**Goal**: Users see server connection status in the header

**Independent Test**: Start monitor, observe "Connected" status, disconnect network, observe status change

### Phase Start
- [x] T113 [GIT] Verify working tree is clean before starting Phase 5
- [x] T114 [GIT] Pull and rebase on origin/main if needed
- [x] T115 [US3] Create retro/P5.md for this phase
- [x] T116 [GIT] Commit: initialize phase 5 retro

### Connection Status Types
- [x] T117 [US3] Implement ConnectionStatus enum in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T118 [GIT] Commit: add ConnectionStatus enum
- [x] T119 [US3] Add connection_status field to DashboardState in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T120 [GIT] Commit: add connection status to dashboard state

### Connection Status Widget
- [x] T121 [US3] Implement connection_status widget in monitor/src/tui/widgets/header.rs (use devs:rust-dev agent)
- [x] T122 [GIT] Commit: add connection status widget
- [x] T123 [US3] Add color-blind safe indicators (symbols + colors) in monitor/src/tui/widgets/header.rs (use devs:rust-dev agent)
- [x] T124 [GIT] Commit: add accessible status indicators

### Header Widget
- [x] T125 [US3] Implement header widget combining logo and status in monitor/src/tui/widgets/header.rs (use devs:rust-dev agent)
- [x] T126 [GIT] Commit: add combined header widget
- [x] T127 [US3] Handle narrow terminal graceful degradation in monitor/src/tui/widgets/header.rs (use devs:rust-dev agent)
- [x] T128 [GIT] Commit: add header graceful degradation

### Connection Status Updates
- [x] T129 [US3] Integrate sender connection state into TUI in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [x] T130 [GIT] Commit: integrate connection status updates

### Codebase Mapping and Retro
- [x] T131 [US3] Run /sdd:map incremental for Phase 5 changes
- [x] T132 [GIT] Commit: update codebase documents for phase 5
- [ ] T133 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T134 [GIT] Commit: finalize phase 5 retro

### Phase Completion
- [ ] T135 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T136 [GIT] Create/update PR to main with phase summary
- [ ] T137 [GIT] Verify all CI checks pass
- [ ] T138 [GIT] Report PR ready status

**Checkpoint**: User Story 3 complete - connection status visible in header

---

## Phase 6: User Story 4 - View Authentication Credentials (Priority: P2)

**Goal**: Users see their session name and public key for server configuration

**Independent Test**: Start monitor, verify credentials panel shows session name and base64 public key

### Phase Start
- [ ] T139 [GIT] Verify working tree is clean before starting Phase 6
- [ ] T140 [GIT] Pull and rebase on origin/main if needed
- [ ] T141 [US4] Create retro/P6.md for this phase
- [ ] T142 [GIT] Commit: initialize phase 6 retro

### Credentials Types
- [ ] T143 [US4] Implement Credentials struct in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T144 [GIT] Commit: add Credentials struct

### Credentials Widget
- [ ] T145 [US4] Implement credentials widget in monitor/src/tui/widgets/credentials.rs (use devs:rust-dev agent)
- [ ] T146 [GIT] Commit: add credentials widget
- [ ] T147 [US4] Format public key in base64 for copy-paste in monitor/src/tui/widgets/credentials.rs (use devs:rust-dev agent)
- [ ] T148 [GIT] Commit: add base64 public key formatting
- [ ] T149 [US4] Handle narrow terminal key display (wrap/truncate) in monitor/src/tui/widgets/credentials.rs (use devs:rust-dev agent)
- [ ] T150 [GIT] Commit: handle narrow terminal for credentials

### Dashboard Layout Integration
- [ ] T151 [US4] Integrate credentials panel into dashboard layout in monitor/src/tui/ui.rs (use devs:rust-dev agent)
- [ ] T152 [GIT] Commit: integrate credentials into dashboard

### Codebase Mapping and Retro
- [ ] T153 [US4] Run /sdd:map incremental for Phase 6 changes
- [ ] T154 [GIT] Commit: update codebase documents for phase 6
- [ ] T155 [US4] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T156 [GIT] Commit: finalize phase 6 retro

### Phase Completion
- [ ] T157 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T158 [GIT] Create/update PR to main with phase summary
- [ ] T159 [GIT] Verify all CI checks pass
- [ ] T160 [GIT] Report PR ready status

**Checkpoint**: User Story 4 complete - credentials visible for server setup

---

## Phase 7: User Story 5 - Track Event Statistics (Priority: P2)

**Goal**: Users see counts of total events, successful sends, and failures in footer

**Independent Test**: Start monitor, trigger events, verify footer counters increment appropriately

### Phase Start
- [ ] T161 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T162 [GIT] Pull and rebase on origin/main if needed
- [ ] T163 [US5] Create retro/P7.md for this phase
- [ ] T164 [GIT] Commit: initialize phase 7 retro

### Event Stats Types
- [ ] T165 [US5] Implement EventStats struct in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T166 [GIT] Commit: add EventStats struct

### Stats Footer Widget
- [ ] T167 [US5] Implement stats_footer widget in monitor/src/tui/widgets/stats_footer.rs (use devs:rust-dev agent)
- [ ] T168 [GIT] Commit: add stats footer widget
- [ ] T169 [US5] Add visual distinction for failed count (color + style) in monitor/src/tui/widgets/stats_footer.rs (use devs:rust-dev agent)
- [ ] T170 [GIT] Commit: highlight failed events

### Stats Integration
- [ ] T171 [US5] Integrate SenderMetrics into EventStats updates in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T172 [GIT] Commit: integrate sender metrics into stats display
- [ ] T173 [US5] Add stats footer to dashboard layout in monitor/src/tui/ui.rs (use devs:rust-dev agent)
- [ ] T174 [GIT] Commit: add stats footer to dashboard layout

### Codebase Mapping and Retro
- [ ] T175 [US5] Run /sdd:map incremental for Phase 7 changes
- [ ] T176 [GIT] Commit: update codebase documents for phase 7
- [ ] T177 [US5] Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T178 [GIT] Commit: finalize phase 7 retro

### Phase Completion
- [ ] T179 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T180 [GIT] Create/update PR to main with phase summary
- [ ] T181 [GIT] Verify all CI checks pass
- [ ] T182 [GIT] Report PR ready status

**Checkpoint**: User Story 5 complete - event statistics visible in footer

---

## Phase 8: User Story 6 - Custom Session Name Setup (Priority: P3)

**Goal**: Power users can specify custom session name during setup

**Independent Test**: Launch monitor, enter custom session name, complete setup, verify credentials shows custom name

### Phase Start
- [ ] T183 [GIT] Verify working tree is clean before starting Phase 8
- [ ] T184 [GIT] Pull and rebase on origin/main if needed
- [ ] T185 [US6] Create retro/P8.md for this phase
- [ ] T186 [GIT] Commit: initialize phase 8 retro

### Session Name Input
- [ ] T187 [US6] Enhance session name input field with character input/delete in monitor/src/tui/widgets/setup_form.rs (use devs:rust-dev agent)
- [ ] T188 [GIT] Commit: enhance session name input handling
- [ ] T189 [US6] Add inline validation error display in monitor/src/tui/widgets/setup_form.rs (use devs:rust-dev agent)
- [ ] T190 [GIT] Commit: add inline validation errors

### Validation Rules
- [ ] T191 [US6] Implement validation rules per FR-026 (64 char limit, alphanumeric/-/_) in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T192 [GIT] Commit: implement session name validation rules

### Codebase Mapping and Retro
- [ ] T193 [US6] Run /sdd:map incremental for Phase 8 changes
- [ ] T194 [GIT] Commit: update codebase documents for phase 8
- [ ] T195 [US6] Review retro/P8.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T196 [GIT] Commit: finalize phase 8 retro

### Phase Completion
- [ ] T197 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T198 [GIT] Create/update PR to main with phase summary
- [ ] T199 [GIT] Verify all CI checks pass
- [ ] T200 [GIT] Report PR ready status

**Checkpoint**: User Story 6 complete - custom session names supported

---

## Phase 9: User Story 7 - Key Management Options (Priority: P3)

**Goal**: Users can choose between existing keys or generating new ones

**Independent Test**: Run monitor with existing keys, select "Generate new key", verify new keypair created and old backed up

### Phase Start
- [ ] T201 [GIT] Verify working tree is clean before starting Phase 9
- [ ] T202 [GIT] Pull and rebase on origin/main if needed
- [ ] T203 [US7] Create retro/P9.md for this phase
- [ ] T204 [GIT] Commit: initialize phase 9 retro

### Key Option UI
- [ ] T205 [US7] Implement key option toggle in setup form in monitor/src/tui/widgets/setup_form.rs (use devs:rust-dev agent)
- [ ] T206 [GIT] Commit: add key option toggle UI
- [ ] T207 [US7] Show "Use existing" only when keys exist in monitor/src/tui/widgets/setup_form.rs (use devs:rust-dev agent)
- [ ] T208 [GIT] Commit: conditionally show key options

### Key Backup Logic
- [ ] T209 [US7] Implement key backup with timestamp suffix in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T210 [GIT] Commit: add key backup before regeneration

### Codebase Mapping and Retro
- [ ] T211 [US7] Run /sdd:map incremental for Phase 9 changes
- [ ] T212 [GIT] Commit: update codebase documents for phase 9
- [ ] T213 [US7] Review retro/P9.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T214 [GIT] Commit: finalize phase 9 retro

### Phase Completion
- [ ] T215 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T216 [GIT] Create/update PR to main with phase summary
- [ ] T217 [GIT] Verify all CI checks pass
- [ ] T218 [GIT] Report PR ready status

**Checkpoint**: User Story 7 complete - key management options available

---

## Phase 10: User Story 8 - Display VibeTea Logo (Priority: P3)

**Goal**: TUI displays stylized VibeTea ASCII logo in header

**Independent Test**: Start monitor, verify ASCII art logo appears in header area

### Phase Start
- [ ] T219 [GIT] Verify working tree is clean before starting Phase 10
- [ ] T220 [GIT] Pull and rebase on origin/main if needed
- [ ] T221 [US8] Create retro/P10.md for this phase
- [ ] T222 [GIT] Commit: initialize phase 10 retro

### Logo Widget
- [ ] T223 [US8] Implement logo widget with ASCII art in monitor/src/tui/widgets/logo.rs (use devs:rust-dev agent)
- [ ] T224 [GIT] Commit: add VibeTea ASCII logo widget
- [ ] T225 [US8] Implement graceful degradation for narrow terminals in monitor/src/tui/widgets/logo.rs (use devs:rust-dev agent)
- [ ] T226 [GIT] Commit: add logo degradation for narrow terminals

### Logo Integration
- [ ] T227 [US8] Integrate logo into header layout in monitor/src/tui/widgets/header.rs (use devs:rust-dev agent)
- [ ] T228 [GIT] Commit: integrate logo into header

### Codebase Mapping and Retro
- [ ] T229 [US8] Run /sdd:map incremental for Phase 10 changes
- [ ] T230 [GIT] Commit: update codebase documents for phase 10
- [ ] T231 [US8] Review retro/P10.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T232 [GIT] Commit: finalize phase 10 retro

### Phase Completion
- [ ] T233 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T234 [GIT] Create/update PR to main with phase summary
- [ ] T235 [GIT] Verify all CI checks pass
- [ ] T236 [GIT] Report PR ready status

**Checkpoint**: User Story 8 complete - VibeTea logo displayed

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Final integration, main.rs updates, and overall quality

### Phase Start
- [ ] T237 [GIT] Verify working tree is clean before starting Phase 11
- [ ] T238 [GIT] Pull and rebase on origin/main if needed
- [ ] T239 Create retro/P11.md for this phase
- [ ] T240 [GIT] Commit: initialize phase 11 retro

### Main.rs Integration
- [ ] T241 Update monitor/src/main.rs to make TUI the default mode in monitor/src/main.rs (use devs:rust-dev agent)
- [ ] T242 [GIT] Commit: make TUI default mode
- [ ] T243 Preserve existing init/run subcommands for scripting in monitor/src/main.rs (use devs:rust-dev agent)
- [ ] T244 [GIT] Commit: preserve CLI subcommands

### Dashboard Input Handling
- [ ] T245 Implement dashboard input handling (q/Esc quit, scroll keys) in monitor/src/tui/input.rs (use devs:rust-dev agent)
- [ ] T246 [GIT] Commit: add dashboard input handling

### Size Warning
- [ ] T247 [P] Implement size_warning widget for terminals below 80x24 in monitor/src/tui/widgets/size_warning.rs (use devs:rust-dev agent)
- [ ] T248 [GIT] Commit: add terminal size warning widget
- [ ] T249 Add minimum size check before entering TUI mode in monitor/src/tui/terminal.rs (use devs:rust-dev agent)
- [ ] T250 [GIT] Commit: add minimum terminal size check

### Log Redirection
- [ ] T251 Suppress stderr logging in TUI mode per NFR-005 in monitor/src/main.rs (use devs:rust-dev agent)
- [ ] T252 [GIT] Commit: suppress logging in TUI mode

### Signal Handling
- [ ] T253 Integrate TUI shutdown with existing signal handlers in monitor/src/main.rs (use devs:rust-dev agent)
- [ ] T254 [GIT] Commit: integrate TUI with signal handlers

### Performance
- [ ] T255 Verify 60ms tick rate and render throttling in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T256 [GIT] Commit: verify TUI performance settings

### NO_COLOR Support
- [ ] T257 Implement NO_COLOR environment variable detection in monitor/src/tui/app.rs (use devs:rust-dev agent)
- [ ] T258 [GIT] Commit: add NO_COLOR support

### Documentation
- [ ] T259 [P] Update CLAUDE.md with TUI feature documentation
- [ ] T260 [GIT] Commit: update CLAUDE.md for TUI feature

### Run Quickstart Validation
- [ ] T261 Run quickstart.md validation steps to verify TUI works
- [ ] T262 [GIT] Commit: verify quickstart validation passes

### Codebase Mapping and Retro
- [ ] T263 Run /sdd:map incremental for Phase 11 changes
- [ ] T264 [GIT] Commit: update codebase documents for phase 11
- [ ] T265 Review retro/P11.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T266 [GIT] Commit: finalize phase 11 retro

### Phase Completion
- [ ] T267 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T268 [GIT] Create/update PR to main with phase summary
- [ ] T269 [GIT] Verify all CI checks pass
- [ ] T270 [GIT] Report PR ready status

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-10)**: All depend on Foundational phase completion
  - US1, US2, US3 (all P1): Complete sequentially for MVP
  - US4, US5 (P2): Can start after MVP user stories
  - US6, US7, US8 (P3): Can start after P2 stories
- **Polish (Phase 11)**: Depends on all user stories being complete

### User Story Dependencies

| Story | Priority | Dependencies | Can Run In Parallel With |
|-------|----------|--------------|--------------------------|
| US1 - Quick Start | P1 | Foundational | None (first) |
| US2 - Event Stream | P1 | Foundational, US1 | None (needs setup working) |
| US3 - Connection Status | P1 | Foundational | US2 (different widgets) |
| US4 - Credentials | P2 | US1 (needs credentials) | US5 |
| US5 - Statistics | P2 | Foundational, Sender Metrics | US4 |
| US6 - Custom Session | P3 | US1 (extends setup) | US7, US8 |
| US7 - Key Management | P3 | US1 (extends setup) | US6, US8 |
| US8 - Logo | P3 | US3 (header widget) | US6, US7 |

### Within Each User Story

- State types before widgets
- Widgets before integration
- Integration before testing
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T028, T029 (AppState and Screen) can run in parallel
- T031, T032 (Theme and Symbols) can run in parallel

**Phase 3 (US1)**:
- T052, T053, T054 (SetupFormState, SetupField, KeyOption) can run in parallel

**Phase 4 (US2)**:
- T086, T087 (DisplayEvent and DisplayEventType) can run in parallel

**Phase 11 (Polish)**:
- T247 (size_warning widget) can run in parallel with other widgets
- T259 (documentation) can run in parallel with code tasks

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch state types in parallel:
Task: "Implement AppState struct in monitor/src/tui/app.rs"
Task: "Implement Screen enum in monitor/src/tui/app.rs"

# Launch theme and symbols in parallel:
Task: "Implement Theme struct in monitor/src/tui/app.rs"
Task: "Implement Symbols struct in monitor/src/tui/app.rs"
```

---

## Implementation Strategy

### MVP First (User Stories 1, 2, 3)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Quick Start)
4. Complete Phase 4: User Story 2 (Event Stream)
5. Complete Phase 5: User Story 3 (Connection Status)
6. **STOP and VALIDATE**: Test MVP independently
7. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Test → Users can quick-start with defaults
3. Add US2 → Test → Users see real-time events
4. Add US3 → Test → Users see connection status (MVP complete!)
5. Add US4/US5 → Test → Credentials and statistics visible
6. Add US6/US7/US8 → Test → Full feature set

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| Phase 1 | T001-T015 | Setup and project structure |
| Phase 2 | T016-T047 | Foundational infrastructure |
| Phase 3 | T048-T081 | US1: Quick Start with Defaults (P1) |
| Phase 4 | T082-T112 | US2: Real-Time Event Stream (P1) |
| Phase 5 | T113-T138 | US3: Connection Status (P1) |
| Phase 6 | T139-T160 | US4: Authentication Credentials (P2) |
| Phase 7 | T161-T182 | US5: Event Statistics (P2) |
| Phase 8 | T183-T200 | US6: Custom Session Name (P3) |
| Phase 9 | T201-T218 | US7: Key Management Options (P3) |
| Phase 10 | T219-T236 | US8: VibeTea Logo (P3) |
| Phase 11 | T237-T270 | Polish & Cross-Cutting |

**Total Tasks**: 270
**MVP Tasks**: ~138 (Phases 1-5)
**Parallel Opportunities**: Marked with [P] throughout

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- [GIT] tasks enforce proper commit workflow
- Each user story should be independently completable and testable
- Commit after each logical implementation task
- Stop at any checkpoint to validate story independently
- Use `devs:rust-dev` agent for all Rust implementation tasks
- Run `/sdd:map incremental` at end of each phase to keep codebase docs current
