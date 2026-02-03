# Tasks: Client Frontend Redesign

**Input**: Design documents from `/specs/002-client-frontend-redesign/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are NOT explicitly requested in the feature specification. Test tasks are excluded.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- **[GIT]**: Git workflow task (commits, pushes, PRs)
- Include exact file paths in descriptions

## Path Conventions

- **Client**: `client/src/` for all frontend code
- Paths assume monorepo structure per plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, dependencies, and basic structure

### Phase Start
- [x] T001 [GIT] Verify on feature branch 002-client-frontend-redesign and working tree is clean
- [x] T002 [GIT] Pull and rebase on origin/main if needed

### Implementation
- [x] T003 Install framer-motion and recharts dependencies in client/package.json (use devs:typescript-dev agent)
- [x] T004 [GIT] Commit: add animation and charting dependencies
- [x] T005 [P] Install figlet devDependency and @types/figlet in client/package.json (use devs:typescript-dev agent)
- [x] T006 [GIT] Commit: add figlet build dependency
- [x] T007 Create ASCII art generation script at client/scripts/generate-ascii.mjs (use devs:typescript-dev agent)
- [x] T008 [GIT] Commit: add ASCII art generation script
- [x] T009 Add prebuild script to client/package.json for ASCII generation (use devs:typescript-dev agent)
- [x] T010 [GIT] Commit: configure prebuild script for ASCII generation
- [x] T011 Create design tokens file at client/src/constants/design-tokens.ts with COLORS, SPRING_CONFIGS, ANIMATION_TIMING (use devs:typescript-dev agent)
- [x] T012 [GIT] Commit: add design tokens constants
- [x] T013 Create CSS animations file at client/src/styles/animations.css with flicker and pulse keyframes (use devs:typescript-dev agent)
- [x] T014 [GIT] Commit: add CSS animation keyframes
- [x] T015 [P] Create directory structure: client/src/components/animated/, client/src/components/graphs/, client/src/assets/ascii/ (use devs:typescript-dev agent)
- [x] T016 [GIT] Commit: create new component directory structure
- [x] T017 Run npm run build to generate ASCII art file at client/src/assets/ascii/vibetea-logo.ts
- [x] T018 [GIT] Commit: generate initial ASCII art asset

### Phase Completion
- [x] T019 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [x] T020 [GIT] Create/update PR to main with phase summary
- [x] T021 [GIT] Verify all CI checks pass
- [x] T022 [GIT] Report PR ready status

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core hooks and utilities that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [ ] T023 [GIT] Verify working tree is clean before starting Phase 2
- [ ] T024 [GIT] Pull and rebase on origin/main if needed
- [ ] T025 Create retro/P2.md for this phase

### Implementation
- [ ] T026 [GIT] Commit: initialize phase 2 retro
- [ ] T027 [P] Create useReducedMotion hook at client/src/hooks/useReducedMotion.ts (use devs:react-dev agent)
- [ ] T028 [P] Create usePageVisibility hook at client/src/hooks/usePageVisibility.ts (use devs:react-dev agent)
- [ ] T029 [P] Create useAnimationThrottle hook at client/src/hooks/useAnimationThrottle.ts (use devs:react-dev agent)
- [ ] T030 [GIT] Commit: add animation utility hooks
- [ ] T031 Create AnimationErrorBoundary component at client/src/components/animated/ErrorBoundary.tsx (use devs:react-dev agent)
- [ ] T032 [GIT] Commit: add AnimationErrorBoundary component
- [ ] T033 Configure LazyMotion provider wrapper in client/src/App.tsx (use devs:react-dev agent)
- [ ] T034 [GIT] Commit: configure LazyMotion provider
- [ ] T035 Import animations.css in client/src/main.tsx or App.tsx (use devs:react-dev agent)
- [ ] T036 [GIT] Commit: import CSS animations
- [ ] T037 Run /sdd:map incremental for Phase 2 changes
- [ ] T038 [GIT] Commit: update codebase documents for phase 2
- [ ] T039 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T040 [GIT] Commit: finalize phase 2 retro

### Phase Completion
- [ ] T041 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T042 [GIT] Create/update PR to main with phase summary
- [ ] T043 [GIT] Verify all CI checks pass
- [ ] T044 [GIT] Report PR ready status

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - First Impressions Matter (Priority: P1) MVP

**Goal**: Animated hero header with ASCII art, flickering grid background, and warm color palette for login/auth screen

**Independent Test**: View login/auth screen and verify hero section animates on load with ASCII art, grid background, and warm color palette

### Phase Start
- [ ] T045 [GIT] Verify working tree is clean before starting Phase 3
- [ ] T046 [GIT] Pull and rebase on origin/main if needed
- [ ] T047 [US1] Create retro/P3.md for this phase

### Implementation
- [ ] T048 [GIT] Commit: initialize phase 3 retro
- [ ] T049 [P] [US1] Create ASCIIHeader component at client/src/components/animated/ASCIIHeader.tsx with spring entrance animation (use devs:react-dev agent)
- [ ] T050 [P] [US1] Create AnimatedBackground component at client/src/components/animated/AnimatedBackground.tsx with flickering grid and particles (use devs:react-dev agent)
- [ ] T051 [GIT] Commit: add ASCIIHeader and AnimatedBackground components
- [ ] T052 [US1] Create SpringContainer wrapper component at client/src/components/animated/SpringContainer.tsx (use devs:react-dev agent)
- [ ] T053 [GIT] Commit: add SpringContainer wrapper
- [ ] T054 [US1] Integrate ASCIIHeader and AnimatedBackground into token entry view in client/src/App.tsx (use devs:react-dev agent)
- [ ] T055 [GIT] Commit: integrate hero section into token entry view
- [ ] T056 [US1] Apply warm color palette (#131313, #d97757) to token entry view styling (use devs:react-dev agent)
- [ ] T057 [GIT] Commit: apply warm color palette to token entry
- [ ] T058 [US1] Wrap animated components with AnimationErrorBoundary in App.tsx (use devs:react-dev agent)
- [ ] T059 [GIT] Commit: wrap hero components with error boundary
- [ ] T060 [US1] Run /sdd:map incremental for Phase 3 changes
- [ ] T061 [GIT] Commit: update codebase documents for phase 3
- [ ] T062 [US1] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T063 [GIT] Commit: finalize phase 3 retro

### Phase Completion
- [ ] T064 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T065 [GIT] Create/update PR to main with phase summary
- [ ] T066 [GIT] Verify all CI checks pass
- [ ] T067 [GIT] Report PR ready status

**Checkpoint**: User Story 1 (First Impressions) fully functional - ASCII header animates, background flickers, warm palette applied

---

## Phase 4: User Story 2 - Dynamic Activity Visualization (Priority: P1) MVP

**Goal**: Enhanced heatmap with glow effects and spring animations based on activity level

**Independent Test**: Connect with active sessions and verify heatmap cells animate with glowing effects and activity-based pulse variations

### Phase Start
- [ ] T068 [GIT] Verify working tree is clean before starting Phase 4
- [ ] T069 [GIT] Pull and rebase on origin/main if needed
- [ ] T070 [US2] Create retro/P4.md for this phase

### Implementation
- [ ] T071 [GIT] Commit: initialize phase 4 retro
- [ ] T072 [US2] Add HeatmapGlowState interface to client/src/types/ or inline in Heatmap.tsx (use devs:typescript-dev agent)
- [ ] T073 [GIT] Commit: add HeatmapGlowState interface
- [ ] T074 [US2] Enhance Heatmap.tsx with glow effect logic: 2s timer restart, brightness stacking up to 5 events, decay animation (use devs:react-dev agent)
- [ ] T075 [GIT] Commit: add heatmap glow effect logic
- [ ] T076 [US2] Add CSS glow animation using box-shadow with orange accent color in client/src/components/Heatmap.tsx or animations.css (use devs:react-dev agent)
- [ ] T077 [GIT] Commit: add heatmap glow CSS
- [ ] T078 [US2] Add spring-animated tooltip to heatmap cells on hover (use devs:react-dev agent)
- [ ] T079 [GIT] Commit: add animated tooltip to heatmap
- [ ] T080 [US2] Integrate useReducedMotion hook in Heatmap.tsx to respect motion preferences (use devs:react-dev agent)
- [ ] T081 [GIT] Commit: integrate reduced motion support in heatmap
- [ ] T082 [US2] Run /sdd:map incremental for Phase 4 changes
- [ ] T083 [GIT] Commit: update codebase documents for phase 4
- [ ] T084 [US2] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T085 [GIT] Commit: finalize phase 4 retro

### Phase Completion
- [ ] T086 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T087 [GIT] Create/update PR to main with phase summary
- [ ] T088 [GIT] Verify all CI checks pass
- [ ] T089 [GIT] Report PR ready status

**Checkpoint**: User Story 2 (Dynamic Heatmap) fully functional - cells glow, brightness stacks, tooltips animate

---

## Phase 5: User Story 3 - Session Cards with Personality (Priority: P2)

**Goal**: Session cards with animated borders, spring transitions on status change, and hover effects

**Independent Test**: Observe session cards during status transitions (active → idle → ended) and verify smooth animations

### Phase Start
- [ ] T090 [GIT] Verify working tree is clean before starting Phase 5
- [ ] T091 [GIT] Pull and rebase on origin/main if needed
- [ ] T092 [US3] Create retro/P5.md for this phase

### Implementation
- [ ] T093 [GIT] Commit: initialize phase 5 retro
- [ ] T094 [US3] Add SessionCardAnimationState interface to client/src/types/ or inline in SessionOverview.tsx (use devs:typescript-dev agent)
- [ ] T095 [GIT] Commit: add SessionCardAnimationState interface
- [ ] T096 [US3] Enhance SessionOverview.tsx with animated glowing border for active sessions (use devs:react-dev agent)
- [ ] T097 [GIT] Commit: add animated border to active sessions
- [ ] T098 [US3] Add spring-based status transition animations (stiffness: 260, damping: 20) to SessionOverview.tsx (use devs:react-dev agent)
- [ ] T099 [GIT] Commit: add spring status transitions
- [ ] T100 [US3] Add hover state with subtle scale and glow enhancement to session cards (use devs:react-dev agent)
- [ ] T101 [GIT] Commit: add hover effects to session cards
- [ ] T102 [US3] Integrate useReducedMotion hook in SessionOverview.tsx (use devs:react-dev agent)
- [ ] T103 [GIT] Commit: integrate reduced motion support in session cards
- [ ] T104 [US3] Run /sdd:map incremental for Phase 5 changes
- [ ] T105 [GIT] Commit: update codebase documents for phase 5
- [ ] T106 [US3] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T107 [GIT] Commit: finalize phase 5 retro

### Phase Completion
- [ ] T108 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T109 [GIT] Create/update PR to main with phase summary
- [ ] T110 [GIT] Verify all CI checks pass
- [ ] T111 [GIT] Report PR ready status

**Checkpoint**: User Story 3 (Session Cards) fully functional - borders glow, transitions animate, hover responds

---

## Phase 6: User Story 4 - Event Stream with Visual Polish (Priority: P2)

**Goal**: Event stream with SVG iconography, entrance animations for new events only (< 5 sec old), throttled to 10/sec

**Independent Test**: Generate various event types and verify distinct visual treatment and smooth scroll behavior

### Phase Start
- [ ] T112 [GIT] Verify working tree is clean before starting Phase 6
- [ ] T113 [GIT] Pull and rebase on origin/main if needed
- [ ] T114 [US4] Create retro/P6.md for this phase

### Implementation
- [ ] T115 [GIT] Commit: initialize phase 6 retro
- [ ] T116 [P] [US4] Create SVG icon components for event types (tool, session, error) at client/src/components/icons/ or inline (use devs:react-dev agent)
- [ ] T117 [GIT] Commit: add event type SVG icons
- [ ] T118 [US4] Add EventAnimationState interface and component-local animation state to EventStream.tsx (use devs:typescript-dev agent)
- [ ] T119 [GIT] Commit: add event animation state tracking
- [ ] T120 [US4] Implement entrance animation logic: only animate events < 5 seconds old based on timestamp (use devs:react-dev agent)
- [ ] T121 [GIT] Commit: implement event age check for animations
- [ ] T122 [US4] Integrate useAnimationThrottle hook to cap animations at 10/sec in EventStream.tsx (use devs:react-dev agent)
- [ ] T123 [GIT] Commit: integrate animation throttling
- [ ] T124 [US4] Replace Unicode emoji with SVG icons and apply color-coded badges per event type (use devs:react-dev agent)
- [ ] T125 [GIT] Commit: replace emoji with SVG icons
- [ ] T126 [US4] Add fade/slide entrance animation to new event rows using motion component (use devs:react-dev agent)
- [ ] T127 [GIT] Commit: add entrance animation to event rows
- [ ] T128 [US4] Ensure visual hierarchy with timestamp, type badge, and description styling (use devs:react-dev agent)
- [ ] T129 [GIT] Commit: refine event stream visual hierarchy
- [ ] T130 [US4] Run /sdd:map incremental for Phase 6 changes
- [ ] T131 [GIT] Commit: update codebase documents for phase 6
- [ ] T132 [US4] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T133 [GIT] Commit: finalize phase 6 retro

### Phase Completion
- [ ] T134 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T135 [GIT] Create/update PR to main with phase summary
- [ ] T136 [GIT] Verify all CI checks pass
- [ ] T137 [GIT] Report PR ready status

**Checkpoint**: User Story 4 (Event Stream) fully functional - icons display, new events animate, throttle works

---

## Phase 7: User Story 5 - Atmospheric Background System (Priority: P2)

**Goal**: Dashboard background with flickering grid, particle/twinkle effects, and Page Visibility API pause

**Independent Test**: Observe background layer with no overlapping elements and verify animation performance

### Phase Start
- [ ] T138 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T139 [GIT] Pull and rebase on origin/main if needed
- [ ] T140 [US5] Create retro/P7.md for this phase

### Implementation
- [ ] T141 [GIT] Commit: initialize phase 7 retro
- [ ] T142 [US5] Enhance AnimatedBackground.tsx with 20px grid cells, 0.5-2Hz flicker, 5-15% opacity variation (use devs:react-dev agent)
- [ ] T143 [GIT] Commit: refine grid flicker parameters
- [ ] T144 [US5] Add particle/twinkle effect layer (10-20 particles, slow drift) to AnimatedBackground.tsx (use devs:react-dev agent)
- [ ] T145 [GIT] Commit: add particle twinkle effects
- [ ] T146 [US5] Integrate usePageVisibility hook to pause animations when tab is hidden (use devs:react-dev agent)
- [ ] T147 [GIT] Commit: integrate page visibility pause
- [ ] T148 [US5] Apply AnimatedBackground as base layer behind main dashboard content (z-index management) (use devs:react-dev agent)
- [ ] T149 [GIT] Commit: position background behind dashboard content
- [ ] T150 [US5] Add will-change: opacity and contain: layout style paint for GPU optimization (use devs:react-dev agent)
- [ ] T151 [GIT] Commit: add GPU optimization hints
- [ ] T152 [US5] Run /sdd:map incremental for Phase 7 changes
- [ ] T153 [GIT] Commit: update codebase documents for phase 7
- [ ] T154 [US5] Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T155 [GIT] Commit: finalize phase 7 retro

### Phase Completion
- [ ] T156 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T157 [GIT] Create/update PR to main with phase summary
- [ ] T158 [GIT] Verify all CI checks pass
- [ ] T159 [GIT] Report PR ready status

**Checkpoint**: User Story 5 (Atmospheric Background) fully functional - grid flickers, particles drift, pauses when hidden

---

## Phase 8: User Story 6 - Connection Status with Flair (Priority: P3)

**Goal**: Enhanced connection indicator with animated ring effects (connecting), glowing pulse (connected), distinct error states

**Independent Test**: Toggle connection states and verify visual feedback matches state

### Phase Start
- [ ] T160 [GIT] Verify working tree is clean before starting Phase 8
- [ ] T161 [GIT] Pull and rebase on origin/main if needed
- [ ] T162 [US6] Create retro/P8.md for this phase

### Implementation
- [ ] T163 [GIT] Commit: initialize phase 8 retro
- [ ] T164 [US6] Add ConnectionStatusAnimationState interface to ConnectionStatus.tsx (use devs:typescript-dev agent)
- [ ] T165 [GIT] Commit: add connection animation state interface
- [ ] T166 [US6] Implement glowing green pulse animation for connected state (use devs:react-dev agent)
- [ ] T167 [GIT] Commit: add connected pulse animation
- [ ] T168 [US6] Implement animated ring effect for connecting/reconnecting states (use devs:react-dev agent)
- [ ] T169 [GIT] Commit: add connecting ring animation
- [ ] T170 [US6] Implement distinct warning visual for disconnected state with reconnect option (use devs:react-dev agent)
- [ ] T171 [GIT] Commit: add disconnected warning visual
- [ ] T172 [US6] Integrate useReducedMotion hook in ConnectionStatus.tsx (use devs:react-dev agent)
- [ ] T173 [GIT] Commit: integrate reduced motion support in connection status
- [ ] T174 [US6] Run /sdd:map incremental for Phase 8 changes
- [ ] T175 [GIT] Commit: update codebase documents for phase 8
- [ ] T176 [US6] Review retro/P8.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T177 [GIT] Commit: finalize phase 8 retro

### Phase Completion
- [ ] T178 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T179 [GIT] Create/update PR to main with phase summary
- [ ] T180 [GIT] Verify all CI checks pass
- [ ] T181 [GIT] Report PR ready status

**Checkpoint**: User Story 6 (Connection Status) fully functional - pulse, ring, and warning states working

---

## Phase 9: User Story 7 - Graph Visualizations (Priority: P3, MVP Required)

**Goal**: Activity trends line/area graph and event type distribution chart using real-time Zustand event store

**Independent Test**: View graph components with sample data and verify rendering and animation

### Phase Start
- [ ] T182 [GIT] Verify working tree is clean before starting Phase 9
- [ ] T183 [GIT] Pull and rebase on origin/main if needed
- [ ] T184 [US7] Create retro/P9.md for this phase

### Implementation
- [ ] T185 [GIT] Commit: initialize phase 9 retro
- [ ] T186 [P] [US7] Add ActivityDataPoint and ActivityGraphProps interfaces to client/src/types/ or inline (use devs:typescript-dev agent)
- [ ] T187 [P] [US7] Add EventTypeDistribution and EventDistributionChartProps interfaces (use devs:typescript-dev agent)
- [ ] T188 [GIT] Commit: add graph data interfaces
- [ ] T189 [US7] Create ActivityGraph component at client/src/components/graphs/ActivityGraph.tsx using Recharts (use devs:react-dev agent)
- [ ] T190 [GIT] Commit: add ActivityGraph component
- [ ] T191 [US7] Add time range toggle (1h/6h/24h) to ActivityGraph with callback support (use devs:react-dev agent)
- [ ] T192 [GIT] Commit: add time range toggle to ActivityGraph
- [ ] T193 [US7] Create EventDistributionChart component at client/src/components/graphs/EventDistributionChart.tsx using Recharts (use devs:react-dev agent)
- [ ] T194 [GIT] Commit: add EventDistributionChart component
- [ ] T195 [US7] Apply warm palette colors to both graph components (stroke, fill using COLORS constant) (use devs:react-dev agent)
- [ ] T196 [GIT] Commit: apply warm palette to graphs
- [ ] T197 [US7] Enable Recharts accessibilityLayer for ARIA labels and keyboard navigation (use devs:react-dev agent)
- [ ] T198 [GIT] Commit: add accessibility layer to graphs
- [ ] T199 [US7] Integrate graphs into main dashboard layout in App.tsx (use devs:react-dev agent)
- [ ] T200 [GIT] Commit: integrate graphs into dashboard
- [ ] T201 [US7] Run /sdd:map incremental for Phase 9 changes
- [ ] T202 [GIT] Commit: update codebase documents for phase 9
- [ ] T203 [US7] Review retro/P9.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T204 [GIT] Commit: finalize phase 9 retro

### Phase Completion
- [ ] T205 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T206 [GIT] Create/update PR to main with phase summary
- [ ] T207 [GIT] Verify all CI checks pass
- [ ] T208 [GIT] Report PR ready status

**Checkpoint**: User Story 7 (Graphs) fully functional - activity trends and distribution charts display with animation

---

## Phase 10: User Story 8 - Component Documentation in Storybook (Priority: P3)

**Goal**: Storybook setup with stories for all animated components demonstrating states and animation behaviors

**Independent Test**: Run Storybook and verify all new animated components have documented stories

### Phase Start
- [ ] T209 [GIT] Verify working tree is clean before starting Phase 10
- [ ] T210 [GIT] Pull and rebase on origin/main if needed
- [ ] T211 [US8] Create retro/P10.md for this phase

### Implementation
- [ ] T212 [GIT] Commit: initialize phase 10 retro
- [ ] T213 [US8] Initialize Storybook with Vite builder in client/ using npx storybook@latest init (use devs:typescript-dev agent)
- [ ] T214 [GIT] Commit: initialize Storybook
- [ ] T215 [US8] Configure Storybook for Tailwind 4 and framer-motion in client/.storybook/main.ts (use devs:typescript-dev agent)
- [ ] T216 [GIT] Commit: configure Storybook for Tailwind and motion
- [ ] T217 [P] [US8] Create ASCIIHeader.stories.tsx with animateOnLoad variations (use devs:react-dev agent)
- [ ] T218 [P] [US8] Create AnimatedBackground.stories.tsx with grid/particle toggles (use devs:react-dev agent)
- [ ] T219 [P] [US8] Create ErrorBoundary.stories.tsx with error trigger demonstration (use devs:react-dev agent)
- [ ] T220 [GIT] Commit: add animated component stories
- [ ] T221 [P] [US8] Create ActivityGraph.stories.tsx with sample data and time range controls (use devs:react-dev agent)
- [ ] T222 [P] [US8] Create EventDistributionChart.stories.tsx with various event distributions (use devs:react-dev agent)
- [ ] T223 [GIT] Commit: add graph component stories
- [ ] T224 [US8] Create stories for enhanced existing components: Heatmap, SessionOverview, EventStream, ConnectionStatus (use devs:react-dev agent)
- [ ] T225 [GIT] Commit: add enhanced component stories
- [ ] T226 [US8] Add reduced-motion testing decorator to demonstrate both animated and reduced-motion modes (use devs:react-dev agent)
- [ ] T227 [GIT] Commit: add reduced-motion decorator to Storybook
- [ ] T228 [US8] Add storybook script to client/package.json (use devs:typescript-dev agent)
- [ ] T229 [GIT] Commit: add storybook npm script
- [ ] T230 [US8] Run /sdd:map incremental for Phase 10 changes
- [ ] T231 [GIT] Commit: update codebase documents for phase 10
- [ ] T232 [US8] Review retro/P10.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T233 [GIT] Commit: finalize phase 10 retro

### Phase Completion
- [ ] T234 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T235 [GIT] Create/update PR to main with phase summary
- [ ] T236 [GIT] Verify all CI checks pass
- [ ] T237 [GIT] Report PR ready status

**Checkpoint**: User Story 8 (Storybook) fully functional - all components documented with interactive controls

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Performance optimization, accessibility compliance, CI/CD enhancements

### Phase Start
- [ ] T238 [GIT] Verify working tree is clean before starting Phase 11
- [ ] T239 [GIT] Pull and rebase on origin/main if needed
- [ ] T240 Create retro/P11.md for this phase

### Implementation
- [ ] T241 [GIT] Commit: initialize phase 11 retro
- [ ] T242 [P] Add skeleton loader components for initial data fetch (FR-016) in client/src/components/ (use devs:react-dev agent)
- [ ] T243 [P] Add empty state components with call-to-action text (FR-017) in client/src/components/ (use devs:react-dev agent)
- [ ] T244 [GIT] Commit: add skeleton loaders and empty states
- [ ] T245 Add Intersection Observer pause for off-screen animated elements (NFR-005) (use devs:react-dev agent)
- [ ] T246 [GIT] Commit: add intersection observer pause for off-screen elements
- [ ] T247 [P] Verify max 10 concurrent animations rule (FR-009) across all components (use devs:react-dev agent)
- [ ] T248 [P] Audit all interactive elements for hover/focus spring feedback compliance (FR-007) (use devs:react-dev agent)
- [ ] T249 [GIT] Commit: verify animation limits and hover/focus compliance
- [ ] T250 [P] Add Lighthouse CI configuration to GitHub workflow (CI-001) (use devs:typescript-dev agent)
- [ ] T251 [P] Add Storybook build step to GitHub workflow (CI-002) (use devs:typescript-dev agent)
- [ ] T252 [P] Add reduced-motion compliance check to GitHub workflow (CI-003) (use devs:typescript-dev agent)
- [ ] T253 [GIT] Commit: add CI workflow enhancements
- [ ] T254 Run bundle size analysis and verify animation library < 50KB gzipped (NFR-001)
- [ ] T255 [GIT] Commit: verify bundle size compliance
- [ ] T256 Run quickstart.md validation - verify all setup steps work (use devs:typescript-dev agent)
- [ ] T257 [GIT] Commit: validate quickstart documentation
- [ ] T258 Update CLAUDE.md with final feature documentation summary
- [ ] T259 [GIT] Commit: update CLAUDE.md with feature summary
- [ ] T260 Run /sdd:map incremental for final changes
- [ ] T261 [GIT] Commit: final codebase document update
- [ ] T262 Review retro/P11.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T263 [GIT] Commit: finalize polish phase retro

### Phase Completion
- [ ] T264 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T265 [GIT] Create/update PR to main with final phase summary
- [ ] T266 [GIT] Verify all CI checks pass
- [ ] T267 [GIT] Report PR ready status

**Checkpoint**: All user stories complete, CI passing, performance validated, documentation finalized

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1: Setup
    ↓
Phase 2: Foundational (BLOCKS all user stories)
    ↓
┌───────────────────────────────────────────────────────────────┐
│ User Stories (can proceed in parallel or priority order)      │
│                                                               │
│ Phase 3: US1 - First Impressions (P1) MVP                    │
│ Phase 4: US2 - Dynamic Heatmap (P1) MVP                      │
│ Phase 5: US3 - Session Cards (P2)                            │
│ Phase 6: US4 - Event Stream (P2)                             │
│ Phase 7: US5 - Atmospheric Background (P2)                   │
│ Phase 8: US6 - Connection Status (P3)                        │
│ Phase 9: US7 - Graph Visualizations (P3) MVP                 │
│ Phase 10: US8 - Storybook Documentation (P3)                 │
└───────────────────────────────────────────────────────────────┘
    ↓
Phase 11: Polish (after all desired stories complete)
```

### User Story Dependencies

- **US1 (First Impressions)**: After Foundational - No dependencies on other stories
- **US2 (Heatmap)**: After Foundational - No dependencies on other stories
- **US3 (Session Cards)**: After Foundational - No dependencies on other stories
- **US4 (Event Stream)**: After Foundational - No dependencies on other stories
- **US5 (Background)**: After Foundational - Can reuse AnimatedBackground from US1 if implementing sequentially
- **US6 (Connection Status)**: After Foundational - No dependencies on other stories
- **US7 (Graphs)**: After Foundational - No dependencies on other stories
- **US8 (Storybook)**: After Foundational - Benefits from having components to document

### Parallel Opportunities

Within each user story, tasks marked [P] can run in parallel:
- Setup phase: T005 parallel with T003
- Foundational: T027, T028, T029 (all hooks) in parallel
- US1: T049, T050 (ASCIIHeader and AnimatedBackground) in parallel
- US4: T116 (SVG icons) parallel with earlier tasks in different files
- US7: T186, T187 (interfaces) in parallel
- US8: T217, T218, T219 (stories) in parallel; T221, T222 in parallel
- Polish: T242, T243 (loaders/empty states); T249, T250, T251 (CI steps) in parallel

---

## Implementation Strategy

### MVP First (User Stories 1, 2, and 7)

Per spec clarification, User Story 7 (Graphs) is MVP-required despite P3 priority.

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (First Impressions) - **MVP**
4. Complete Phase 4: User Story 2 (Heatmap) - **MVP**
5. Complete Phase 9: User Story 7 (Graphs) - **MVP Required**
6. **STOP and VALIDATE**: Test MVP independently
7. Deploy/demo if ready

### Full Feature Delivery

1. MVP phases (Setup → Foundational → US1 → US2 → US7)
2. Add remaining P2 stories: US3, US4, US5
3. Add remaining P3 stories: US6, US8
4. Complete Polish phase
5. Final validation and deployment

### Priority Order for Performance Trade-offs

If performance constraints require compromises (per spec):
1. Color palette and typography (highest - defines aesthetic)
2. Heatmap glow effects (US2 - primary data visualization)
3. Session card animations (US3 - secondary navigation)
4. Event stream entrance animations (US4 - can be simplified)
5. Background flickering grid (US5 - can be reduced)
6. Particle/twinkle effects (US5 - lowest priority, pure decoration)

---

## Notes

- [P] tasks = different files, no dependencies, can run in parallel
- [Story] label maps task to specific user story for traceability
- [GIT] tasks enforce commit-per-task workflow
- Each user story should be independently completable and testable
- Commit after each implementation task (or batch parallelizable tasks)
- Stop at any checkpoint to validate story independently
- All animated components use devs:react-dev agent
- All TypeScript interfaces/types use devs:typescript-dev agent
- Motion library uses LazyMotion for bundle optimization per research.md
