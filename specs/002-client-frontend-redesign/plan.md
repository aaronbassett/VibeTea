# Implementation Plan: Client Frontend Redesign

**Branch**: `002-client-frontend-redesign` | **Date**: 2026-02-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-client-frontend-redesign/spec.md`

## Summary

Transform the VibeTea client dashboard from functional placeholder to a visually distinctive developer tool using the **frontend-vibes** aesthetic. The redesign introduces ASCII art headers, flickering grid backgrounds, particle effects, enhanced heatmap/session/event animations, graph visualizations, and connection status polish - all while maintaining 60fps performance, accessibility compliance (prefers-reduced-motion, WCAG AA), and <50KB gzipped bundle increase.

## Technical Context

**Language/Version**: TypeScript 5.9, React 19.2
**Primary Dependencies**: React, Zustand 5.0, TailwindCSS 4.1, Vite 7.3
**Animation Library**: Framer Motion (or lighter alternative like motion-one if bundle size requires)
**Charting Library**: Recharts or uPlot (lightweight, compatible with Tailwind palette)
**Storage**: N/A (in-memory Zustand store only)
**Testing**: Vitest 4.0 with React Testing Library (planned)
**Target Platform**: Modern browsers (ES2020+, CSS animations, Intersection Observer)
**Project Type**: Web SPA (React client)
**Performance Goals**: 60fps during normal operation, <3s TTI, Lighthouse >80
**Constraints**: <50KB gzipped animation library increase, max 10 concurrent animations, GPU-accelerated CSS only (transform, opacity)
**Scale/Scope**: Single-page dashboard with 7 animated components, extending existing structure

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Privacy by Design | PASS | No new data collection; animations are purely visual presentation |
| II. Unix Philosophy | PASS | Client displays only; new components are focused (one effect each) |
| III. Keep It Simple (KISS/YAGNI) | WATCH | Animation complexity must be justified by UX improvement |
| IV. Event-Driven Communication | PASS | Client receives events via WebSocket; no new communication patterns |
| V. Test What Matters | PASS | Visual regression tests for animated states; Storybook for components |
| VI. Fail Fast & Loud | PASS | Error boundaries with static fallback for animation failures (FR-013) |
| VII. Modularity & Clear Boundaries | PASS | Each animated component isolated; shared animation configs |

**Constitution Violations Requiring Justification**: None

## Project Structure

### Documentation (this feature)

```text
specs/002-client-frontend-redesign/
├── plan.md              # This file (/sdd:plan command output)
├── research.md          # Phase 0 output - animation library selection, performance research
├── data-model.md        # Phase 1 output - component state interfaces
├── quickstart.md        # Phase 1 output - dev setup guide
├── contracts/           # Phase 1 output - component API documentation
└── tasks.md             # Phase 2 output (/sdd:tasks command - NOT created by /sdd:plan)
```

### Source Code (repository root)

```text
client/
├── src/
│   ├── components/
│   │   ├── animated/           # NEW - animated primitive components
│   │   │   ├── ASCIIHeader.tsx         # FR-001 - figlet art with spring entrance
│   │   │   ├── AnimatedBackground.tsx  # FR-002 - flickering grid + particles
│   │   │   ├── SpringContainer.tsx     # Shared spring animation wrapper
│   │   │   └── ErrorBoundary.tsx       # FR-013 - static fallback wrapper
│   │   ├── ConnectionStatus.tsx   # EXTEND - FR-006 animated indicators
│   │   ├── EventStream.tsx        # EXTEND - FR-005 SVG icons, entrance animations
│   │   ├── Heatmap.tsx            # EXTEND - FR-003 glow effects
│   │   ├── SessionOverview.tsx    # EXTEND - FR-004 spring transitions
│   │   └── graphs/                # NEW - FR-015 graph visualizations
│   │       ├── ActivityGraph.tsx         # Line/area activity trends
│   │       └── EventDistributionChart.tsx # Event type proportions
│   ├── hooks/
│   │   ├── useReducedMotion.ts    # NEW - prefers-reduced-motion hook
│   │   └── useAnimationThrottle.ts # NEW - FR-018 throttle entrance animations
│   ├── styles/
│   │   └── animations.css         # NEW - CSS keyframes for flicker, pulse
│   ├── constants/
│   │   └── design-tokens.ts       # NEW - color palette, spring configs
│   ├── assets/
│   │   └── ascii/                 # NEW - pre-generated figlet strings
│   │       └── vibetea-logo.ts    # Build-time generated ASCII art
│   └── types/
│       └── events.ts              # EXTEND - if needed for graph data
└── .storybook/                    # DS-002 - component documentation
    └── main.ts
```

**Structure Decision**: Extending existing client structure with new `animated/`, `graphs/`, and `styles/` directories for clear separation of new visual components from existing functional components.

## Complexity Tracking

> **No violations identified** - feature extends existing patterns without introducing new architectural complexity.

## Learnings from Previous Retros

### From Phase 8-10 (P8.md, P9.md, P10.md)

**What Worked Well:**
- `@tanstack/react-virtual` for large lists - already installed, works with React 19
- Zustand selective subscriptions prevent unnecessary re-renders
- Tailwind v4 CSS-first configuration makes custom animations straightforward
- useMemo for expensive calculations (event counting, cell generation)
- Pure render behavior - avoid Date.now() in useMemo, use event timestamps as reference

**Patterns to Reuse:**
- STATUS_CONFIG record pattern for component state → display properties mapping
- Unicode escape sequences for emoji icons (e.g., `'\u{1F527}'`)
- CSS Grid with `contents` display for row grouping
- Keyboard accessibility pattern (handleKeyDown for Enter/Space)
- View toggle with `role="group"` and `aria-pressed`

**Technical Considerations:**
- TypeScript generic type narrowing doesn't work in switch statements - use explicit type assertions
- React Compiler may warn about unstable function references from hooks - acceptable for visualization hooks

**Dependencies Already Available:**
- No charting library yet - will need to add Recharts or similar (FR-015)
- Framer Motion not yet installed - primary animation library decision needed

---

## Phase 0: Outline & Research

### Research Questions

1. **Animation Library Selection**: Framer Motion vs motion-one vs CSS-only
   - Bundle size comparison (<50KB gzipped constraint)
   - Spring physics support (stiffness: 260/300, damping: 20/30)
   - React 19 compatibility
   - Tree-shaking capabilities

2. **Charting Library Selection** (FR-015): Recharts vs uPlot vs Victory
   - Bundle size impact
   - Tailwind color integration
   - Animation support for data transitions
   - Accessibility (aria labels)

3. **ASCII Art Generation**: Build-time figlet approach
   - Figlet font selection for "cool dev tool" aesthetic
   - Pre-generation script for Vite build
   - String storage format

4. **Flickering Grid Performance**: CSS vs Canvas
   - CSS animation timing functions for organic flicker
   - GPU layer promotion strategies
   - Page Visibility API integration

5. **Event Stream Animation Throttling** (FR-018):
   - Best approach for 10/sec max entrance animations
   - Animation queue vs timestamp gating

### Research Tasks

| Topic | Question | Output |
|-------|----------|--------|
| Animation library | Framer Motion bundle size with tree-shaking? | Decision in research.md |
| Animation library | motion-one as lighter alternative? | Comparison in research.md |
| Charting | Recharts vs uPlot bundle comparison | Decision in research.md |
| ASCII | Best figlet font for developer aesthetic | Font name in research.md |
| Performance | CSS animation flicker implementation | Code pattern in research.md |
| Throttling | Animation throttle implementation pattern | Pattern in research.md |

---

## Phase 1: Design & Contracts

### Data Model

Key interfaces for new animated components:

```typescript
// design-tokens.ts
export const SPRING_CONFIGS = {
  expressive: { stiffness: 260, damping: 20 },  // FR-004
  standard: { stiffness: 300, damping: 30 },
} as const;

export const COLORS = {
  background: {
    primary: '#131313',
    secondary: '#1a1a1a',
  },
  accent: {
    orange: '#d97757',
  },
  // ... WCAG AA compliant contrast ratios
} as const;

// Glow state for heatmap cells (FR-003)
interface HeatmapGlowState {
  brightness: number;      // 0-1, stacks up to 5 events
  timerResetAt: number;    // timestamp for 2s decay timer
  eventStack: number;      // 0-5, brightness = stack * 0.2
}

// Entrance animation state (FR-005, FR-018)
interface EventAnimationState {
  shouldAnimate: boolean;  // false if event > 5 seconds old
  animationQueued: number; // timestamp when added to animation queue
}
```

### Component Contracts

Each new component will have documented props interface in `/contracts/`:

1. **ASCIIHeader.tsx** - `text: string, animateOnLoad?: boolean`
2. **AnimatedBackground.tsx** - `showGrid?: boolean, showParticles?: boolean`
3. **ActivityGraph.tsx** - `events: VibeteaEvent[], timeRange: '1h' | '6h' | '24h'`
4. **EventDistributionChart.tsx** - `events: VibeteaEvent[]`

### Artifacts to Generate

- `research.md` - Animation/charting library decisions
- `data-model.md` - Component state interfaces, animation configs
- `contracts/` - Component API documentation
- `quickstart.md` - Dev environment setup for animations

---

## Phase 2: Local Development Environment

### Current Tooling Status

From CONVENTIONS.md and TESTING.md:

| Tool | Status | Configuration |
|------|--------|---------------|
| ESLint | Configured | `eslint.config.js` |
| Prettier | Configured | `.prettierrc` |
| TypeScript | Configured | `tsconfig.json` (strict mode) |
| Vitest | Configured | Inline in `vite.config.ts` |
| Tailwind | Configured | v4 CSS-first |
| Lefthook | Configured | `lefthook.yml` |

### New Tooling Needed

1. **Storybook** (DS-002):
   - Install `@storybook/react-vite`
   - Configure for Tailwind 4 + Framer Motion
   - Add stories for all new animated components

2. **Visual Regression** (DS-003):
   - Chromatic or Percy integration for CI
   - Snapshot animated component states

3. **Lighthouse CI** (CI-001):
   - Add `@lhci/cli` to workflow
   - Configure performance budget assertions

4. **Reduced Motion Testing** (CI-003):
   - Configure test utility to simulate `prefers-reduced-motion`

### Package Additions (Post-Research)

```json
{
  "devDependencies": {
    "@storybook/react-vite": "^8.x",
    "@lhci/cli": "^0.x"
  },
  "dependencies": {
    "framer-motion": "^11.x",  // or alternative from research
    "recharts": "^2.x"         // or alternative from research
  }
}
```

---

## Next Steps

1. **Run `/sdd:plan` Phase 0** - Execute research tasks, produce `research.md`
2. **Complete Phase 1** - Generate `data-model.md`, `contracts/`, `quickstart.md`
3. **Setup Storybook** - Initialize and configure for new components
4. **Run `/sdd:tasks`** - Generate implementation tasks from this plan
