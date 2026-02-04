# Component API Contracts

**Created**: 2026-02-03
**Purpose**: Document public APIs for new animated components

---

## ASCIIHeader

**Location**: `client/src/components/animated/ASCIIHeader.tsx`
**Requirement**: FR-001

### Props

```typescript
interface ASCIIHeaderProps {
  /**
   * Pre-generated ASCII art string to display.
   * Should be imported from src/assets/ascii/
   */
  text: string;

  /**
   * Whether to animate the header on initial mount.
   * @default true
   */
  animateOnLoad?: boolean;

  /**
   * Additional CSS classes for the container.
   */
  className?: string;
}
```

### Usage

```tsx
import { ASCIIHeader } from './components/animated/ASCIIHeader';
import { VIBETEA_ASCII } from './assets/ascii/vibetea-logo';

<ASCIIHeader text={VIBETEA_ASCII} animateOnLoad />
```

### Behavior

- Renders ASCII art in a `<pre>` element with monospace font
- Spring entrance animation from opacity 0 to 1
- Respects `prefers-reduced-motion` (no animation if enabled)
- Font color uses accent orange (#d97757)

---

## AnimatedBackground

**Location**: `client/src/components/animated/AnimatedBackground.tsx`
**Requirement**: FR-002, FR-012

### Props

```typescript
interface AnimatedBackgroundProps {
  /**
   * Whether to show the flickering grid effect.
   * @default true
   */
  showGrid?: boolean;

  /**
   * Whether to show particle/twinkle effects.
   * @default true
   */
  showParticles?: boolean;

  /**
   * Number of particles (10-20 per spec).
   * @default 15
   */
  particleCount?: number;

  /**
   * Additional CSS classes.
   */
  className?: string;

  /**
   * Children to render on top of background.
   */
  children?: React.ReactNode;
}
```

### Usage

```tsx
import { AnimatedBackground } from './components/animated/AnimatedBackground';

<AnimatedBackground showGrid showParticles>
  <MainContent />
</AnimatedBackground>
```

### Behavior

- Renders behind all content (z-index: -1)
- Grid uses CSS animations at 0.5-2Hz with 5-15% opacity variation
- Particles drift slowly with random positions
- Pauses when tab is not visible (Page Visibility API)
- GPU-accelerated via `will-change: opacity`

---

## ErrorBoundary

**Location**: `client/src/components/animated/ErrorBoundary.tsx`
**Requirement**: FR-013

### Props

```typescript
interface AnimationErrorBoundaryProps {
  /**
   * Static fallback UI to render when animation fails.
   */
  fallback: React.ReactNode;

  /**
   * Children components to wrap.
   */
  children: React.ReactNode;

  /**
   * Optional callback when error occurs.
   */
  onError?: (error: Error, componentName: string) => void;
}
```

### Usage

```tsx
import { AnimationErrorBoundary } from './components/animated/ErrorBoundary';

<AnimationErrorBoundary
  fallback={<StaticHeader text="VibeTea" />}
  onError={(error) => console.error('Animation failed:', error)}
>
  <ASCIIHeader text={VIBETEA_ASCII} />
</AnimationErrorBoundary>
```

### Behavior

- Catches errors in child component tree
- Renders fallback UI on error
- Logs error details for debugging
- Does not re-throw (graceful degradation)

---

## ActivityGraph

**Location**: `client/src/components/graphs/ActivityGraph.tsx`
**Requirement**: FR-015 (activity trends)

### Props

```typescript
interface ActivityGraphProps {
  /**
   * Events to visualize. Read from Zustand store.
   */
  events: readonly VibeteaEvent[];

  /**
   * Time range for visualization.
   * @default '1h'
   */
  timeRange?: '1h' | '6h' | '24h';

  /**
   * Callback when user changes time range.
   */
  onTimeRangeChange?: (range: '1h' | '6h' | '24h') => void;

  /**
   * Additional CSS classes.
   */
  className?: string;
}
```

### Usage

```tsx
import { ActivityGraph } from './components/graphs/ActivityGraph';
import { useEventStore } from './hooks/useEventStore';

const events = useEventStore((state) => state.events);

<ActivityGraph
  events={events}
  timeRange="1h"
  onTimeRangeChange={setTimeRange}
/>
```

### Behavior

- Line/area chart showing event count over time
- Uses warm palette colors
- Animates on initial render and data changes
- Accessible with ARIA labels
- Time range toggle with 1h/6h/24h options

---

## EventDistributionChart

**Location**: `client/src/components/graphs/EventDistributionChart.tsx`
**Requirement**: FR-015 (event type distribution)

### Props

```typescript
interface EventDistributionChartProps {
  /**
   * Events to analyze. Read from Zustand store.
   */
  events: readonly VibeteaEvent[];

  /**
   * Additional CSS classes.
   */
  className?: string;
}
```

### Usage

```tsx
import { EventDistributionChart } from './components/graphs/EventDistributionChart';
import { useEventStore } from './hooks/useEventStore';

const events = useEventStore((state) => state.events);

<EventDistributionChart events={events} />
```

### Behavior

- Pie/donut chart showing event type proportions
- Each type has distinct color from warm palette
- Animates on initial render
- Accessible with ARIA labels for each segment
- Shows count and percentage on hover/focus

---

## Hook APIs

### useReducedMotion

**Location**: `client/src/hooks/useReducedMotion.ts`
**Requirement**: FR-008

```typescript
/**
 * Detects user's motion preference.
 * @returns true if user prefers reduced motion
 */
function useReducedMotion(): boolean;
```

### useAnimationThrottle

**Location**: `client/src/hooks/useAnimationThrottle.ts`
**Requirement**: FR-018

```typescript
/**
 * Throttles entrance animations to max 10/sec.
 * @returns Function that returns true if animation should proceed
 */
function useAnimationThrottle(): () => boolean;
```

### usePageVisibility

**Location**: `client/src/hooks/usePageVisibility.ts`
**Requirement**: FR-002 (pauses when tab hidden)

```typescript
/**
 * Tracks page visibility state.
 * @returns true if tab is visible
 */
function usePageVisibility(): boolean;
```

---

## Extended Existing Components

### ConnectionStatus Enhancement

**Location**: `client/src/components/ConnectionStatus.tsx`
**Requirement**: FR-006

**New Props Added**:

```typescript
interface ConnectionStatusProps {
  // ... existing props ...

  /**
   * Whether to show animated indicators.
   * @default true
   */
  animated?: boolean;
}
```

**New Behavior**:
- Connected: Glowing green pulse animation
- Connecting/Reconnecting: Animated ring effect
- Disconnected: Static warning indicator

### Heatmap Enhancement

**Location**: `client/src/components/Heatmap.tsx`
**Requirement**: FR-003

**New Behavior**:
- Cells glow with orange accent on new events
- Glow brightness stacks up to 5 events
- 2s decay timer resets on each new event
- Uses CSS `box-shadow` for glow effect

### SessionOverview Enhancement

**Location**: `client/src/components/SessionOverview.tsx`
**Requirement**: FR-004

**New Behavior**:
- Status transitions animate with spring physics (260/20)
- Active sessions have animated glowing border
- Hover state shows subtle scale and glow enhancement

### EventStream Enhancement

**Location**: `client/src/components/EventStream.tsx`
**Requirement**: FR-005, FR-018

**New Behavior**:
- Custom SVG icons per event type (replace Unicode emoji)
- Entrance animation for events < 5 seconds old
- Animation throttled to 10/sec max
- Older events render immediately without animation
