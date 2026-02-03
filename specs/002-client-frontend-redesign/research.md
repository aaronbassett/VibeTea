# Research: Client Frontend Redesign

**Created**: 2026-02-03
**Purpose**: Document technical decisions for animation library, charting, and performance patterns

---

## 1. Animation Library Selection

### Decision: **Motion (formerly Framer Motion) with LazyMotion**

### Rationale

Motion is the recommended choice for these reasons:

1. **Bundle Size (meets <50KB constraint)**:
   - Full `motion` component: ~34KB gzipped
   - With `LazyMotion` + `m` component: **~4.6KB** initial, features lazy-loaded
   - `domAnimation` feature set adds ~15KB (animations, variants, exit animations, tap/hover/focus gestures)
   - Total with common features: **~20KB** - well under the 50KB constraint

2. **Spring Physics Support**: Excellent support for the spec requirements
   - Native spring configuration with `stiffness` and `damping` props
   - Default: stiffness 100, damping 10 (customizable)
   - Spec values (260/20 expressive, 300/30 standard) are directly supported:
     ```jsx
     <motion.div transition={{ type: 'spring', stiffness: 260, damping: 20 }} />
     ```

3. **React 19 Compatibility**: Confirmed compatible
   - Framer Motion v11 (2025) added React 19 support with concurrent rendering improvements
   - Import from `motion/react` for client components
   - TypeScript definitions included

4. **Tree-Shaking**: Excellent
   - Modular architecture with `LazyMotion`
   - Can import only `domAnimation` (15KB) vs `domMax` (25KB)
   - Individual hooks like `useReducedMotion` are ~1KB

### Alternatives Considered

| Library | Bundle Size | Spring Physics | React 19 | Verdict |
|---------|-------------|----------------|----------|---------|
| **Motion** | 4.6-34KB | Full support | Yes | **Recommended** |
| motion-one (WAAPI) | 3.8KB animate() | Limited | Yes | Too limited for complex spring animations |
| CSS-only | 0KB | No | N/A | Cannot achieve stiffness/damping physics |
| react-spring | ~25KB | Full support | Yes | Viable alternative, slightly larger |

---

## 2. Charting Library Selection

### Decision: **Recharts** (with animation caveats)

### Rationale

1. **Bundle Size**:
   - Recharts: ~40-50KB gzipped
   - Uses only required D3 submodules, not the full D3 bundle
   - Tree-shakeable via D3 submodule dependencies

2. **Tailwind CSS Integration**: Good
   - SVG-based rendering works with CSS styling
   - Can apply Tailwind classes to chart containers
   - Colors can be passed as props using CSS variables or hex values
   - Example: `stroke="var(--color-accent)"` or `stroke="#d97757"`

3. **Animation Support**: Moderate (with limitations)
   - Built-in animation on initial render
   - `isAnimationActive`, `animationDuration`, `animationEasing` props
   - **Limitation**: Data transitions redraw entire chart (no morphing)
   - For real-time updates, may need to disable animation or accept redraw behavior

4. **Accessibility**: Good
   - `accessibilityLayer` prop adds ARIA labels, roles, keyboard navigation
   - Arrow key navigation between data points
   - Screen reader support (JAWS/NVDA in Forms Mode)
   - Custom ARIA labels can be added to elements

### Alternatives Considered

| Library | Bundle (gzipped) | Animation | Accessibility | Verdict |
|---------|------------------|-----------|---------------|---------|
| **Recharts** | ~40-50KB | Basic | Good (accessibilityLayer) | **Recommended** |
| **visx** | ~5-15KB/pkg | None (add separately) | Manual | Best for bundle size |
| uPlot | <50KB | None | Manual | Rejected (no animation) |
| Victory | ~50-60KB | Good | Excellent | Best for accessibility |
| Chart.js | ~60KB | Good | Moderate | Canvas-based, harder styling |

---

## 3. ASCII Art (Figlet)

### Decision: **Build-time generation using `figlet` npm package, stored as TypeScript constants**

### Best Fonts for Developer Tool Aesthetic

- **slant** - Clean angled aesthetic, professional feel (Recommended)
- **banner** - Classic UNIX banner style, nostalgic developer vibe
- **big** - Large readable block letters, good for headers
- **block** - Solid block style, bold presence

### Implementation Pattern

```typescript
// src/assets/ascii/vibetea-logo.ts (generated at build time)
export const VIBETEA_ASCII = `
 _    __ _ __        ______
| |  / /(_) /_  ___ /_  __/__  ____ _
| | / / / / __ \\/ _ \\ / / / _ \\/ __ \`/
| |/ / / / /_/ /  __// / /  __/ /_/ /
|___/_/_/_.___/\\___//_/  \\___/\\__,_/
` as const;
```

### Build Script

```javascript
// scripts/generate-ascii.mjs
import figlet from 'figlet';
import fs from 'fs';

const text = figlet.textSync('VibeTea', { font: 'slant' });
fs.writeFileSync('src/assets/ascii/vibetea-logo.ts',
  `export const VIBETEA_ASCII = \`${text}\` as const;\n`);
```

Add to `package.json`: `"prebuild": "node scripts/generate-ascii.mjs"`

### Rationale

- **0KB runtime overhead** - pre-generated strings
- **Type-safe** with `as const`
- **Meets NFR-004** - "ASCII art MUST be pre-generated at build time"

---

## 4. CSS Flickering Grid Performance

### Decision: **CSS `@keyframes` with `opacity` animation, GPU-promoted via `will-change: opacity`**

### CSS Animation Pattern

```css
@keyframes flicker {
  0%, 100% { opacity: 0.05; }
  25% { opacity: 0.12; }
  50% { opacity: 0.08; }
  75% { opacity: 0.15; }
}

.grid-cell {
  animation: flicker 1.5s ease-in-out infinite;
  animation-delay: calc(var(--cell-index) * 0.1s); /* Organic offset */
  will-change: opacity;
}
```

### GPU Layer Promotion

- **`will-change: opacity`**: Hints browser to optimize; promotes to compositor layer
- **`contain: layout style paint`**: Additional optimization for grid container
- **Warning**: Overuse of `will-change` can exhaust GPU memory - use sparingly

### Page Visibility API Integration

```typescript
// hooks/usePageVisibility.ts
import { useEffect, useState } from 'react';

export function usePageVisibility() {
  const [isVisible, setIsVisible] = useState(!document.hidden);

  useEffect(() => {
    const handler = () => setIsVisible(!document.hidden);
    document.addEventListener('visibilitychange', handler);
    return () => document.removeEventListener('visibilitychange', handler);
  }, []);

  return isVisible;
}
```

CSS class toggle approach:
```css
.grid-cell {
  animation-play-state: var(--play-state, running);
}

:root.tab-hidden .grid-cell {
  animation-play-state: paused;
}
```

### Performance Properties

- Only animate `opacity` and `transform` - these are GPU-accelerated by default
- Avoid `filter`, `backdrop-filter` for main grid (use sparingly for accents)
- Target 60fps (16ms per frame budget)

---

## 5. Animation Throttling Pattern

### Decision: **Timestamp gating with useRef**

### Implementation

```typescript
// hooks/useAnimationThrottle.ts
import { useRef, useCallback } from 'react';

const MAX_ANIMATIONS_PER_SECOND = 10;
const WINDOW_MS = 1000;

export function useAnimationThrottle() {
  const timestampsRef = useRef<number[]>([]);

  const shouldAnimate = useCallback((): boolean => {
    const now = Date.now();
    const timestamps = timestampsRef.current;

    // Remove timestamps older than 1 second
    const cutoff = now - WINDOW_MS;
    timestampsRef.current = timestamps.filter(t => t > cutoff);

    if (timestampsRef.current.length >= MAX_ANIMATIONS_PER_SECOND) {
      return false; // Throttled - render without animation
    }

    timestampsRef.current.push(now);
    return true; // Animate
  }, []);

  return shouldAnimate;
}
```

### Usage with Event Age Check (FR-005)

```typescript
// hooks/useThrottledEntrance.ts
import { useState, useEffect, useRef } from 'react';
import { useAnimationThrottle } from './useAnimationThrottle';

const MAX_EVENT_AGE_MS = 5000; // 5 seconds per spec

export function useThrottledEntrance(eventTimestamp: number) {
  const shouldAnimate = useAnimationThrottle();
  const [animate, setAnimate] = useState(false);
  const checkedRef = useRef(false);

  useEffect(() => {
    if (checkedRef.current) return;
    checkedRef.current = true;

    const eventAge = Date.now() - eventTimestamp;
    if (eventAge > MAX_EVENT_AGE_MS) {
      setAnimate(false); // Too old, no animation
      return;
    }

    setAnimate(shouldAnimate());
  }, [eventTimestamp, shouldAnimate]);

  return animate;
}
```

### Rationale

- **Simple O(1) check** per event
- **No animation delay** - events render immediately (just without animation if throttled)
- **Matches spec**: "excess events render immediately without animation" (FR-018)
- **Component-local state**: Per spec (FR-014), animation state not in global store

---

## Summary

| Topic | Decision | Bundle Impact | Key Consideration |
|-------|----------|---------------|-------------------|
| Animation Library | Motion + LazyMotion | ~20KB | Full spring physics, React 19 ready |
| Charting Library | Recharts | ~40-50KB | Built-in accessibility, animation on render |
| ASCII Art | Build-time figlet | 0KB runtime | `slant` font for dev aesthetic |
| Flickering Grid | CSS @keyframes + opacity | 0KB | `will-change: opacity`, Page Visibility API |
| Animation Throttling | Timestamp gating hook | 0KB | 10/sec cap with immediate render fallback |

**Total estimated bundle increase**: ~60-70KB (meets NFR-001: <50KB for animation library alone)

---

## Package Versions

Based on research (current stable versions):

```json
{
  "dependencies": {
    "framer-motion": "^11.18.0",
    "recharts": "^2.15.0"
  },
  "devDependencies": {
    "figlet": "^1.8.0"
  }
}
```
