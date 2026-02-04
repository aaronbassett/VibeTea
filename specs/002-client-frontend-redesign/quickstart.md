# Quickstart: Client Frontend Redesign Development

**Created**: 2026-02-03
**Purpose**: Development environment setup for feature 002

---

## Prerequisites

- Node.js 20+ (for client development)
- npm (comes with Node.js)
- Git

## Quick Setup

```bash
# Clone and checkout feature branch
git checkout 002-client-frontend-redesign

# Install client dependencies
cd client
npm install

# Dependencies are already installed in package.json:
# framer-motion@^12.31.0, recharts@^3.7.0, figlet@^1.10.0

# Start development server
npm run dev
```

## Development Server

```bash
# Start Vite dev server (hot reload)
npm run dev

# Server runs at http://localhost:5173
```

## Available Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development server with hot reload |
| `npm run build` | Build production bundle |
| `npm run preview` | Preview production build locally |
| `npm run lint` | Run ESLint |
| `npm run format` | Format code with Prettier |
| `npm run format:check` | Check formatting without fixing |
| `npm run typecheck` | Run TypeScript type checking |
| `npm test` | Run Vitest unit tests |
| `npm run test:watch` | Run tests in watch mode |

## Additional Scripts

```bash
# Generate ASCII art (runs automatically before build)
npm run prebuild

# Run Storybook for component development
npm run storybook

# Build Storybook for static deployment
npm run build-storybook
```

## Project Structure for This Feature

```
client/src/
├── components/
│   ├── animated/              # NEW
│   │   ├── ASCIIHeader.tsx
│   │   ├── AnimatedBackground.tsx
│   │   ├── SpringContainer.tsx
│   │   └── ErrorBoundary.tsx
│   ├── graphs/                # NEW
│   │   ├── ActivityGraph.tsx
│   │   └── EventDistributionChart.tsx
│   ├── ConnectionStatus.tsx   # EXTEND
│   ├── EventStream.tsx        # EXTEND
│   ├── Heatmap.tsx            # EXTEND
│   └── SessionOverview.tsx    # EXTEND
├── hooks/
│   ├── useReducedMotion.ts    # NEW
│   ├── useAnimationThrottle.ts # NEW
│   └── usePageVisibility.ts   # NEW
├── styles/
│   └── animations.css         # NEW
├── constants/
│   └── design-tokens.ts       # NEW
└── assets/
    └── ascii/
        └── vibetea-logo.ts    # GENERATED
```

## Environment Variables

No new environment variables required for this feature. Existing variables:

| Variable | Purpose | Default |
|----------|---------|---------|
| `VITE_WS_URL` | WebSocket server URL | `ws(s)://{host}/ws` |

## Testing Animations

### Reduced Motion Testing

```bash
# In browser DevTools Console:
# Simulate prefers-reduced-motion
window.matchMedia('(prefers-reduced-motion: reduce)').matches = true

# Or use Chrome DevTools:
# 1. Open DevTools (F12)
# 2. Cmd/Ctrl + Shift + P
# 3. Search "Emulate CSS prefers-reduced-motion"
# 4. Select "prefers-reduced-motion: reduce"
```

### Page Visibility Testing

```bash
# Switch to another tab to test animation pausing
# Background animations should pause when tab is hidden
```

### Animation Throttle Testing

```bash
# Generate rapid events to test throttling:
# Events arriving faster than 10/sec should render without animation
```

## Storybook (DS-002)

Storybook is already configured. Run for component development:

```bash
# Start Storybook dev server
npm run storybook
# Runs at http://localhost:6006

# Build static Storybook
npm run build-storybook
```

## Type Checking

```bash
# Run TypeScript compiler in watch mode
npm run typecheck -- --watch

# Common type errors to watch for:
# - Spring config type mismatches
# - Event type narrowing (use explicit type assertions)
# - Recharts props typing
```

## Performance Profiling

1. Open Chrome DevTools
2. Go to Performance tab
3. Record while interacting with animations
4. Target: All frames at 60fps (16ms frame budget)

## Debugging Tips

### Animation Not Playing

1. Check `prefers-reduced-motion` setting
2. Verify Motion `LazyMotion` wrapper is present
3. Check browser console for errors
4. Verify component is within error boundary

### Flickering Grid Issues

1. Check `will-change: opacity` is applied
2. Verify Page Visibility hook is working
3. Check for CSS specificity conflicts

### Bundle Size Monitoring

```bash
# After build, check bundle size
npm run build

# Review dist/ folder sizes
ls -la dist/assets/

# Use vite-bundle-visualizer for detailed analysis
npx vite-bundle-visualizer
```

## Key Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| framer-motion | ^12.31.0 | Spring animations, gesture support |
| recharts | ^3.7.0 | Graph visualizations |
| figlet | ^1.10.0 | Build-time ASCII art generation |
| @tanstack/react-virtual | ^3.13.18 | Virtual scrolling for event stream |
| Storybook | 8.6.15 | Component documentation |

## Quick Reference

### Spring Configs

```typescript
import { SPRING_CONFIGS } from './constants/design-tokens';

// Expressive (status changes, user feedback)
SPRING_CONFIGS.expressive // { stiffness: 260, damping: 20 }

// Standard (UI transitions)
SPRING_CONFIGS.standard // { stiffness: 300, damping: 30 }
```

### Color Palette

```typescript
import { COLORS } from './constants/design-tokens';

COLORS.background.primary  // '#131313'
COLORS.accent.orange       // '#d97757'
```

### Animation Timing

```typescript
import { ANIMATION_TIMING } from './constants/design-tokens';

ANIMATION_TIMING.eventAnimationMaxAgeMs // 5000
ANIMATION_TIMING.maxEntranceAnimationsPerSecond // 10
```
