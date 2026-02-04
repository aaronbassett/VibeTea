# Feature Specification: Client Frontend Redesign

**Feature Branch**: `002-client-frontend-redesign`
**Created**: 2026-02-03
**Status**: Draft
**Input**: User description: "Massively improve the client frontend with graphs, iconography, twinkling lights, and a dynamic dev tool aesthetic using frontend-vibes design system"
**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## Overview

Transform the VibeTea dashboard from a functional placeholder into a visually distinctive, emotionally expressive developer tool. The redesign applies the **frontend-vibes** aesthetic: faux-ASCII art, warm dark palettes with vibrant accents, spring-based motion, and atmospheric backgrounds. The result should feel like a "super cool dev tool" that makes users excited to monitor their Claude Code sessions.

## Clarifications

### Session 2026-02-03

- Q: Is User Story 7 (Graph Visualizations) in MVP scope given P3 priority, or deferred to Phase 2? → A: Include in MVP - graphs are required for launch
- Q: How is "genuinely new event" determined for entrance animations (FR-005)? → A: Event less than 5 seconds old regardless of source
- Q: How does heatmap glow "extend on rapid events" (FR-003)? → A: Restart 2s timer on each event + increase glow brightness up to 5 event stacks, brightness decays over 2s when events stop
- Q: What loading/empty states should display during initial fetch or zero sessions? → A: Skeleton loaders matching component layout, then empty state with call-to-action text
- Q: How should high-frequency events (50+/sec) be handled for animations? → A: Throttle to max 10 entrance animations per second, render excess events without animation

## Design Direction

Following the frontend-vibes design system:

- **Color Palette**: Warm blacks (#131313-#1a1a1a) with electric orange/coral accents (#d97757)
- **Typography**: Figlet ASCII art for headers (pre-generated at build time), clean monospace for data
- **Motion**: Spring-based animations (stiffness: 260, damping: 20 for expressive; 300/30 for standard)
- **Backgrounds**: Flickering grid using CSS animations (0.5-2Hz flicker, 5-15% opacity variation)
- **Visual Language**: Technical precision meets organic imperfection

## User Scenarios & Testing *(mandatory)*

### User Story 1 - First Impressions Matter (Priority: P1)

A developer opens the VibeTea dashboard for the first time. They see an animated hero header with ASCII art "VibeTea" logo, a flickering grid background, and warm glowing accents. The interface immediately communicates "this is a developer tool with personality."

**Why this priority**: First impressions determine whether users engage with a tool. A distinctive, polished aesthetic establishes credibility and delight.

**Independent Test**: Can be tested by viewing the login/auth screen and verifying the hero section animates on load with ASCII art, grid background, and warm color palette.

**Acceptance Scenarios**:

1. **Given** the user visits the dashboard without a token, **When** the page loads, **Then** they see an animated ASCII art "VibeTea" header with spring entrance animation
2. **Given** the user is viewing the login screen, **When** they observe the background, **Then** they see a subtle flickering grid effect creating visual atmosphere
3. **Given** the user is on any screen, **When** they view the color scheme, **Then** they see warm blacks (#131313) with electric orange (#d97757) accents

---

### User Story 2 - Dynamic Activity Visualization (Priority: P1)

A developer monitoring active sessions sees a reimagined activity heatmap with enhanced visual feedback. Cells pulse and glow based on activity level, with smooth spring animations for state changes. The heatmap feels alive and responsive.

**Why this priority**: The heatmap is the primary data visualization. Making it visually engaging increases user attention and comprehension.

**Independent Test**: Can be tested by connecting with active sessions and verifying heatmap cells animate with glowing effects and activity-based pulse variations.

**Acceptance Scenarios**:

1. **Given** sessions are actively generating events, **When** the user views the heatmap, **Then** active cells glow with intensity proportional to event frequency
2. **Given** a cell receives new events, **When** the count increases, **Then** the cell animates with a spring-based scale and glow effect (2s timer restarts per event, brightness stacks up to 5 events, decays over 2s)
3. **Given** the user hovers over a heatmap cell, **When** viewing the tooltip, **Then** it appears with a smooth spring animation and styled with warm palette

---

### User Story 3 - Session Cards with Personality (Priority: P2)

Session overview cards display with enhanced visual hierarchy and motion. Active sessions have animated borders or glowing effects. Status changes trigger spring animations. The cards feel tactile and responsive.

**Why this priority**: Session cards are the secondary navigation. Enhanced visuals improve scanability and status recognition.

**Independent Test**: Can be tested by observing session cards during status transitions (active → idle → ended) and verifying smooth animations.

**Acceptance Scenarios**:

1. **Given** an active session exists, **When** the user views the session card, **Then** it displays with an animated glowing border effect
2. **Given** a session changes status, **When** the transition occurs, **Then** the card animates smoothly with spring physics
3. **Given** the user hovers over a session card, **When** hovering, **Then** the card responds with subtle scale and glow enhancement

---

### User Story 4 - Event Stream with Visual Polish (Priority: P2)

The event stream displays with enhanced styling: refined event type badges with custom SVG iconography, subtle entrance animations for genuinely new events only (not scrolled-into-view historical events), and improved visual hierarchy. Event types are clearly distinguishable through color and icon.

**Why this priority**: The event stream is the main data display. Visual refinement improves readability during high-frequency updates.

**Independent Test**: Can be tested by generating various event types and verifying distinct visual treatment and smooth scroll behavior.

**Acceptance Scenarios**:

1. **Given** an event less than 5 seconds old appears in the stream, **When** it renders, **Then** it enters with a subtle fade/slide animation (events older than 5 seconds do not animate)
2. **Given** different event types (tool, session, error), **When** viewing the stream, **Then** each type has distinct SVG iconography and color-coded badges matching the warm palette
3. **Given** the user scrolls through events, **When** events are visible, **Then** they display clear visual hierarchy with timestamp, type badge, and description

---

### User Story 5 - Atmospheric Background System (Priority: P2)

The dashboard background includes atmospheric elements: flickering grid, subtle particle effects or "twinkling lights", and depth through layered transparencies. The atmosphere is subtle enough to not distract from data.

**Why this priority**: Atmospheric backgrounds create the "cool dev tool" feeling without interfering with functionality.

**Independent Test**: Can be tested by observing the background layer with no overlapping elements and verifying animation performance.

**Acceptance Scenarios**:

1. **Given** the dashboard loads, **When** viewing the background, **Then** a flickering grid pattern (20px cells, 0.5-2Hz flicker, 5-15% opacity variation) is visible
2. **Given** the user is monitoring sessions, **When** observing the overall UI, **Then** subtle particle/twinkle effects (10-20 particles, slow drift) add visual interest
3. **Given** any background effects are active, **When** the browser tab is not visible, **Then** animations pause to conserve resources (Page Visibility API)

---

### User Story 6 - Connection Status with Flair (Priority: P3)

The connection status indicator transforms from a simple dot to a more expressive visual: animated ring effects when connecting, glowing pulse when connected, distinct treatment for error states.

**Why this priority**: Connection status is critical feedback but lower priority for redesign as current implementation is functional.

**Independent Test**: Can be tested by toggling connection states and verifying visual feedback matches state.

**Acceptance Scenarios**:

1. **Given** the websocket is connected, **When** viewing the status, **Then** a glowing green indicator pulses gently
2. **Given** the websocket is connecting/reconnecting, **When** viewing the status, **Then** an animated ring effect indicates activity
3. **Given** the websocket is disconnected, **When** viewing the status, **Then** a distinct warning visual appears with the reconnect option

---

### User Story 7 - Graph Visualizations (Priority: P3, MVP Required)

Introduce graphical data visualizations showing session activity over time, event type distributions, or other metrics. Graphs use the warm palette and animate on data changes. Data source: real-time event store (Zustand), not Supabase historic data.

**Why this priority**: Lower priority than core visual polish but required for MVP launch. Graphs add analytical value that complements the heatmap visualization.

**Independent Test**: Can be tested by viewing graph components with sample data and verifying rendering and animation.

**Acceptance Scenarios**:

1. **Given** sessions have generated events, **When** viewing analytics, **Then** a line or area graph shows activity trends over time
2. **Given** multiple event types exist, **When** viewing event distribution, **Then** a breakdown visualization shows type proportions
3. **Given** graph data updates, **When** new data arrives, **Then** the graph animates smoothly to reflect changes

---

### User Story 8 - Component Documentation in Storybook (Priority: P3)

Developers working on the dashboard can view and test UI components in isolation using Storybook. Each animated component has stories demonstrating different states and animation behaviors.

**Why this priority**: Storybook improves developer experience and enables visual testing, but is not user-facing functionality.

**Independent Test**: Can be tested by running Storybook and verifying all new animated components have documented stories.

**Acceptance Scenarios**:

1. **Given** a developer runs Storybook, **When** viewing the component library, **Then** all new animated components (ASCIIHeader, AnimatedBackground, EnhancedHeatmapCell, etc.) have documented stories
2. **Given** a component has multiple states, **When** viewing its stories, **Then** each state is demonstrated with controls for animation parameters
3. **Given** reduced motion mode, **When** viewing Storybook, **Then** components can be tested in both animated and reduced-motion modes

---

### Edge Cases

- What happens when the browser does not support CSS animations? Fallback to static styling without visual degradation of functionality.
- How does the system handle reduced motion preferences? Respect `prefers-reduced-motion` media query and disable or minimize animations.
- What happens during poor network conditions? Background effects continue smoothly; data components show loading/error states clearly.
- How does the UI scale on mobile devices? Responsive design maintains functionality with simplified atmospheric effects.
- What happens when animations fail to load? Error boundaries catch failures and render static fallback UI.
- How should interrupted animations be handled? Animations should cancel gracefully without visual artifacts when state changes rapidly.
- What happens during initial data fetch? Skeleton loaders matching component layout display until data arrives.
- What happens when no sessions or events exist? Empty state with call-to-action text guides user on next steps.
- What happens during high-frequency event bursts (50+/sec)? Entrance animations throttled to 10/sec; excess events render immediately without animation.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render an ASCII art header using pre-generated figlet strings with spring entrance animation on initial load
- **FR-002**: System MUST display a flickering grid background (CSS animations, 20px cells, 0.5-2Hz flicker, 5-15% opacity variation) that pauses when tab is not visible
- **FR-003**: Heatmap cells MUST animate with glow effects when receiving new events (glow color: orange accent, 2s timer restarts per event, brightness stacks up to 5 events max, brightness decays over 2s when events stop)
- **FR-004**: Session cards MUST animate status transitions with spring physics (stiffness: 260, damping: 20)
- **FR-005**: Event stream MUST display events with custom SVG iconography and entrance animations for events less than 5 seconds old (based on event timestamp comparison)
- **FR-006**: Connection status MUST display animated visual feedback for each state (connected, connecting, reconnecting, disconnected)
- **FR-007**: All interactive elements MUST respond to hover/focus with spring-based scale or glow feedback
- **FR-008**: System MUST respect `prefers-reduced-motion` media query by disabling or simplifying animations
- **FR-009**: System MUST maintain 60fps performance during normal operation (max 10 concurrent animations)
- **FR-010**: Color palette MUST use warm blacks (#131313-#1a1a1a) and electric orange (#d97757) as primary accent with WCAG AA compliant contrast ratios
- **FR-011**: System MUST support both the token entry view and the main dashboard view with consistent styling
- **FR-012**: Background atmospheric effects MUST be layered behind all UI components without obscuring content
- **FR-013**: Animated components MUST be wrapped in error boundaries with static fallback UI
- **FR-014**: Animation state for event stream items MUST be component-local (not in global store) to avoid re-render cascades
- **FR-015**: System MUST display graph visualizations showing activity trends and event type distribution using real-time event store data
- **FR-016**: System MUST display skeleton loaders matching component layout during initial data fetch
- **FR-017**: System MUST display empty states with call-to-action text when no sessions or events exist
- **FR-018**: Event stream entrance animations MUST be throttled to max 10 per second; excess events render immediately without animation

### Non-Functional Requirements

- **NFR-001**: Animation library bundle size increase MUST be under 50KB gzipped
- **NFR-002**: Background effects MUST use GPU-accelerated CSS properties (transform, opacity) for performance
- **NFR-003**: Component library additions (if any) MUST be tree-shakeable to minimize bundle size
- **NFR-004**: ASCII art MUST be pre-generated at build time (not runtime figlet execution)
- **NFR-005**: Off-screen animated elements MUST pause using Intersection Observer

### Development Standards

- **DS-001**: Pre-commit hooks (Lefthook) MUST run formatting and linting before commits
- **DS-002**: Storybook MUST document all new animated components with interactive controls
- **DS-003**: Visual regression tests MUST be added to CI for animated component states

### CI/CD Requirements

- **CI-001**: GitHub Workflow MUST include Lighthouse performance checks (target: score > 80)
- **CI-002**: GitHub Workflow MUST run Storybook build to verify component documentation
- **CI-003**: GitHub Workflow MUST verify reduced-motion compliance

### Key Entities

- **AnimatedBackground**: Atmospheric layer including flickering grid, particle effects, layered transparencies
- **ASCIIHeader**: Pre-generated figlet text component with spring entrance effects
- **EnhancedHeatmapCell**: Extended heatmap cell with glow and pulse animations (2s timer, brightness stacks to 5 events max)
- **MotionSessionCard**: Session card with spring-based hover and state transition animations
- **StyledEventRow**: Event stream row with SVG iconography and entrance animation (new events only)
- **ConnectionIndicator**: Enhanced connection status with animated ring/glow effects
- **ActivityGraph**: Line/area graph showing activity trends over time with animated data transitions
- **EventDistributionChart**: Breakdown visualization showing event type proportions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users report the dashboard feels "modern" or "polished" in qualitative feedback (target: 80% positive sentiment)
- **SC-002**: UI maintains 60fps during normal operation with animations active (measured via browser DevTools)
- **SC-003**: Time to interactive remains under 3 seconds on standard broadband connections
- **SC-004**: Lighthouse performance score remains above 80 after redesign
- **SC-005**: Users can identify session status and event types at a glance (target: 95% accuracy in usability testing)
- **SC-006**: Dashboard is fully functional with animations disabled (accessibility compliance)
- **SC-007**: All new animated components have Storybook documentation
- **SC-008**: CI pipeline passes with Lighthouse, Storybook build, and reduced-motion checks

## Assumptions

- Framer Motion (or a lighter alternative achieving the same visual result) will be added for spring-based animations
- Figlet ASCII art will be pre-generated at build time and shipped as static strings
- Grid background will use CSS animations (not Canvas/WebGL) for simplicity and performance
- Existing component structure (App.tsx, Heatmap.tsx, SessionOverview.tsx, EventStream.tsx, ConnectionStatus.tsx) will be extended rather than replaced
- Warm palette colors are final and meet WCAG AA contrast requirements
- Lefthook will be used for pre-commit hooks (supports both Rust and TypeScript workflows)
- Browser support: Modern browsers supporting ES2020, CSS animations, Intersection Observer

## Priority Order for Performance Trade-offs

If performance constraints require compromises, preserve effects in this order:
1. Color palette and typography (highest priority - defines aesthetic)
2. Heatmap glow effects (primary data visualization)
3. Session card animations (secondary navigation)
4. Event stream entrance animations (can be simplified)
5. Background flickering grid (can be reduced or removed)
6. Particle/twinkle effects (lowest priority - pure decoration)
