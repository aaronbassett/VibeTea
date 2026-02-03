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
