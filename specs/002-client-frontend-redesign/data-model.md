# Data Model: Client Frontend Redesign

**Created**: 2026-02-03
**Purpose**: Define component state interfaces, design tokens, and animation configurations

---

## Design Tokens

### Color Palette (FR-010)

```typescript
// src/constants/design-tokens.ts

export const COLORS = {
  background: {
    primary: '#131313',    // Main background
    secondary: '#1a1a1a',  // Card/panel background
    tertiary: '#242424',   // Elevated surfaces
  },
  accent: {
    orange: '#d97757',     // Primary accent (electric orange/coral)
    orangeLight: '#e89a7a', // Hover state
    orangeDark: '#c45f3f',  // Active state
  },
  text: {
    primary: '#f5f5f5',    // Main text (WCAG AA on #131313)
    secondary: '#a0a0a0',  // Secondary text
    muted: '#6b6b6b',      // Disabled/placeholder
  },
  status: {
    connected: '#4ade80',   // Green for connected
    connecting: '#facc15',  // Yellow for connecting
    disconnected: '#ef4444', // Red for disconnected
    error: '#ef4444',
  },
  grid: {
    line: '#2a2a2a',       // Grid line color
    glow: '#d97757',       // Glow effect color
  },
} as const;

export type ColorToken = typeof COLORS;
```

### Spring Configurations (FR-004)

```typescript
// src/constants/design-tokens.ts

export const SPRING_CONFIGS = {
  // Expressive animations (user feedback, status changes)
  expressive: {
    type: 'spring' as const,
    stiffness: 260,
    damping: 20,
  },
  // Standard animations (UI transitions)
  standard: {
    type: 'spring' as const,
    stiffness: 300,
    damping: 30,
  },
  // Gentle animations (subtle effects)
  gentle: {
    type: 'spring' as const,
    stiffness: 120,
    damping: 14,
  },
} as const;

export type SpringConfig = (typeof SPRING_CONFIGS)[keyof typeof SPRING_CONFIGS];
```

### Animation Timing

```typescript
// src/constants/design-tokens.ts

export const ANIMATION_TIMING = {
  // Flicker animation (FR-002)
  flicker: {
    minFrequencyHz: 0.5,
    maxFrequencyHz: 2,
    minOpacity: 0.05,
    maxOpacity: 0.15,
  },
  // Glow decay (FR-003)
  glowDecay: {
    durationMs: 2000,
    maxStack: 5,
  },
  // Event animation age threshold (FR-005)
  eventAnimationMaxAgeMs: 5000,
  // Animation throttle (FR-018)
  maxEntranceAnimationsPerSecond: 10,
} as const;
```

---

## Component State Interfaces

### Heatmap Glow State (FR-003)

```typescript
// Extended state for enhanced heatmap cells

interface HeatmapGlowState {
  /** Brightness level (0-1), stacks up to 5 events */
  brightness: number;
  /** Timestamp when timer was last reset */
  timerResetAt: number;
  /** Number of stacked events (0-5, brightness = stack * 0.2) */
  eventStack: number;
}

interface EnhancedHeatmapCell {
  /** Hour bucket key (YYYY-MM-DD-HH format) */
  key: string;
  /** Event count for this hour */
  count: number;
  /** Glow animation state */
  glow: HeatmapGlowState;
}
```

### Event Animation State (FR-005, FR-018)

```typescript
// Per-event animation tracking (component-local, not in global store per FR-014)

interface EventAnimationState {
  /** Whether this event should animate on entrance */
  shouldAnimate: boolean;
  /** Event ID for tracking */
  eventId: string;
  /** Timestamp when animation was queued */
  animationQueuedAt: number | null;
}
```

### Session Card Animation State (FR-004)

```typescript
// Animation states for session cards

type SessionAnimationPhase = 'entering' | 'idle' | 'exiting' | 'statusChange';

interface SessionCardAnimationState {
  /** Current animation phase */
  phase: SessionAnimationPhase;
  /** Previous status (for status change animation) */
  previousStatus: SessionStatus | null;
  /** Whether hover state is active */
  isHovered: boolean;
}
```

### Connection Status Animation State (FR-006)

```typescript
// Enhanced connection status states

type ConnectionAnimationType =
  | 'pulse'      // Connected - gentle pulse
  | 'ring'       // Connecting/reconnecting - expanding rings
  | 'static'     // Disconnected - no animation
  | 'error';     // Error state

interface ConnectionStatusAnimationState {
  /** Current animation type */
  animation: ConnectionAnimationType;
  /** Ring animation iteration count (for connecting) */
  ringIteration: number;
}
```

### Background Animation State (FR-002)

```typescript
// Atmospheric background configuration

interface BackgroundConfig {
  /** Show flickering grid */
  showGrid: boolean;
  /** Show particle/twinkle effects */
  showParticles: boolean;
  /** Number of particles (10-20 per spec) */
  particleCount: number;
  /** Grid cell size in pixels */
  gridCellSize: number;
}

interface BackgroundState extends BackgroundConfig {
  /** Whether tab is visible (Page Visibility API) */
  isTabVisible: boolean;
  /** Animation play state */
  playState: 'running' | 'paused';
}
```

### Graph Data Interfaces (FR-015)

```typescript
// Activity graph data point

interface ActivityDataPoint {
  /** Timestamp (ISO 8601) */
  timestamp: string;
  /** Event count for this time bucket */
  count: number;
  /** Time bucket label for display */
  label: string;
}

interface ActivityGraphProps {
  /** Events to visualize */
  events: readonly VibeteaEvent[];
  /** Time range for visualization */
  timeRange: '1h' | '6h' | '24h';
  /** Callback when time range changes */
  onTimeRangeChange?: (range: '1h' | '6h' | '24h') => void;
}

// Event distribution data

interface EventTypeDistribution {
  /** Event type */
  type: EventType;
  /** Count of events of this type */
  count: number;
  /** Percentage of total */
  percentage: number;
  /** Color for chart segment */
  color: string;
}

interface EventDistributionChartProps {
  /** Events to analyze */
  events: readonly VibeteaEvent[];
}
```

---

## Reduced Motion Support (FR-008)

```typescript
// src/hooks/useReducedMotion.ts

interface ReducedMotionConfig {
  /** Whether user prefers reduced motion */
  prefersReducedMotion: boolean;
  /** Spring config to use (no animation if reduced motion) */
  springConfig: SpringConfig | { duration: 0 };
  /** Whether to show background effects */
  showBackgroundEffects: boolean;
}
```

---

## Error Boundary State (FR-013)

```typescript
// src/components/animated/ErrorBoundary.tsx

interface AnimationErrorBoundaryState {
  /** Whether an error has occurred */
  hasError: boolean;
  /** Error message for logging */
  errorMessage: string | null;
  /** Component that failed */
  failedComponent: string | null;
}

interface AnimationErrorBoundaryProps {
  /** Fallback UI to render on error */
  fallback: React.ReactNode;
  /** Child components to wrap */
  children: React.ReactNode;
  /** Callback when error occurs */
  onError?: (error: Error, componentName: string) => void;
}
```

---

## ASCII Header Configuration (FR-001)

```typescript
// src/components/animated/ASCIIHeader.tsx

interface ASCIIHeaderProps {
  /** ASCII art string to display */
  text: string;
  /** Whether to animate on initial load */
  animateOnLoad?: boolean;
  /** Spring configuration for entrance */
  springConfig?: SpringConfig;
  /** Additional CSS class */
  className?: string;
}
```

---

## Entity Relationships

```
VibeteaEvent (existing)
    │
    ├── drives → EnhancedHeatmapCell.glow
    │              (events update brightness/stack)
    │
    ├── drives → ActivityDataPoint
    │              (aggregated by time bucket)
    │
    ├── drives → EventTypeDistribution
    │              (counted by type)
    │
    └── drives → EventAnimationState
                   (determines entrance animation)

Session (existing)
    │
    └── drives → SessionCardAnimationState
                   (status changes trigger animations)

ConnectionStatus (existing)
    │
    └── drives → ConnectionStatusAnimationState
                   (status maps to animation type)

usePageVisibility
    │
    └── controls → BackgroundState.playState
                     (pauses animations when tab hidden)
```

---

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| HeatmapGlowState | brightness | 0-1 range, increments of 0.2 |
| HeatmapGlowState | eventStack | 0-5 integer |
| EventAnimationState | shouldAnimate | false if event age > 5000ms |
| BackgroundConfig | particleCount | 10-20 range per spec |
| BackgroundConfig | gridCellSize | 20px per spec |
| ActivityDataPoint | count | >= 0 |
| EventTypeDistribution | percentage | 0-100, sum of all = 100 |
