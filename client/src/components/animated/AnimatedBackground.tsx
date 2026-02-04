/**
 * AnimatedBackground Component
 *
 * Renders a GPU-accelerated animated background for the VibeTea dashboard
 * featuring a flickering grid and floating particle effects.
 *
 * Features:
 * - Flickering grid background with 20px cells (0.5-2Hz flicker, 5-15% opacity)
 * - Floating particle/twinkle effects (10-20 particles with slow drift)
 * - Automatic pause when tab is not visible (Page Visibility API)
 * - Respects user's reduced motion preference
 * - GPU-accelerated using CSS transform and opacity properties
 *
 * @see FR-002 specification for animation requirements
 */

import {
  type CSSProperties,
  memo,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';

import { COLORS } from '../../constants/design-tokens';
import { useIntersectionObserver } from '../../hooks/useIntersectionObserver';
import { usePageVisibility } from '../../hooks/usePageVisibility';
import { useReducedMotion } from '../../hooks/useReducedMotion';

/**
 * Props for the AnimatedBackground component.
 */
export interface AnimatedBackgroundProps {
  /** Show flickering grid background (default: true) */
  showGrid?: boolean;
  /** Show particle/twinkle effects (default: true) */
  showParticles?: boolean;
  /** Additional CSS classes for the container */
  className?: string;
}

/**
 * Configuration constants for the animated background.
 */
const CONFIG = {
  /** Grid cell size in pixels */
  GRID_CELL_SIZE: 20,
  /** Number of particles on desktop */
  PARTICLE_COUNT_DESKTOP: 15,
  /** Number of particles on mobile (reduced for performance) */
  PARTICLE_COUNT_MOBILE: 8,
  /** Mobile breakpoint in pixels */
  MOBILE_BREAKPOINT: 768,
  /** Minimum particle size in pixels */
  PARTICLE_MIN_SIZE: 2,
  /** Maximum particle size in pixels */
  PARTICLE_MAX_SIZE: 4,
  /** Minimum particle animation duration in seconds */
  PARTICLE_MIN_DURATION: 15,
  /** Maximum particle animation duration in seconds */
  PARTICLE_MAX_DURATION: 30,
  /** Number of grid columns to render (viewport-based calculation) */
  MAX_GRID_CELLS: 2500, // Limit for performance
} as const;

/**
 * Represents a single floating particle.
 */
interface Particle {
  id: number;
  x: number; // Initial X position (percentage)
  y: number; // Initial Y position (percentage)
  size: number; // Particle size in pixels
  opacity: number; // Base opacity
  duration: number; // Animation duration in seconds
  delay: number; // Animation delay in seconds
  driftX: number; // Horizontal drift amount (percentage)
  driftY: number; // Vertical drift amount (percentage)
}

/**
 * Generates a pseudo-random number based on a seed.
 * Used for deterministic particle generation.
 */
function seededRandom(seed: number): number {
  const x = Math.sin(seed * 9999) * 10000;
  return x - Math.floor(x);
}

/**
 * Generates an array of particles with random properties.
 */
function generateParticles(count: number): Particle[] {
  return Array.from({ length: count }, (_, i) => {
    const seed = i + 1;
    return {
      id: i,
      x: seededRandom(seed * 1) * 100,
      y: seededRandom(seed * 2) * 100,
      size:
        CONFIG.PARTICLE_MIN_SIZE +
        seededRandom(seed * 3) *
          (CONFIG.PARTICLE_MAX_SIZE - CONFIG.PARTICLE_MIN_SIZE),
      opacity: 0.3 + seededRandom(seed * 4) * 0.4,
      duration:
        CONFIG.PARTICLE_MIN_DURATION +
        seededRandom(seed * 5) *
          (CONFIG.PARTICLE_MAX_DURATION - CONFIG.PARTICLE_MIN_DURATION),
      delay: seededRandom(seed * 6) * 10,
      driftX: (seededRandom(seed * 7) - 0.5) * 20, // -10% to +10%
      driftY: (seededRandom(seed * 8) - 0.5) * 20, // -10% to +10%
    };
  });
}

/**
 * Styles for the AnimatedBackground component.
 */
const styles = {
  container: {
    position: 'fixed',
    inset: 0,
    zIndex: -1,
    pointerEvents: 'none',
    overflow: 'hidden',
    contain: 'layout paint',
    transform: 'translateZ(0)',
    backfaceVisibility: 'hidden',
  } satisfies CSSProperties,

  gridLayer: {
    position: 'absolute',
    inset: 0,
    display: 'grid',
    gridTemplateColumns: `repeat(auto-fill, ${CONFIG.GRID_CELL_SIZE}px)`,
    gridTemplateRows: `repeat(auto-fill, ${CONFIG.GRID_CELL_SIZE}px)`,
    contain: 'layout paint',
  } satisfies CSSProperties,

  gridCell: {
    width: CONFIG.GRID_CELL_SIZE,
    height: CONFIG.GRID_CELL_SIZE,
    backgroundColor: COLORS.grid.line,
    willChange: 'opacity',
  } satisfies CSSProperties,

  particleLayer: {
    position: 'absolute',
    inset: 0,
    contain: 'layout paint',
  } satisfies CSSProperties,

  particle: {
    position: 'absolute',
    borderRadius: '50%',
    backgroundColor: COLORS.accent.orange,
    willChange: 'transform, opacity',
    transform: 'translateZ(0)',
  } satisfies CSSProperties,
} as const;

/**
 * Generates keyframe animation name for particle drift.
 */
function getParticleKeyframeName(particle: Particle): string {
  return `particle-drift-${particle.id}`;
}

/**
 * GridLayer Component
 *
 * Renders the flickering grid background.
 */
const GridLayer = memo(function GridLayer({
  isPaused,
  cellCount,
}: {
  isPaused: boolean;
  cellCount: number;
}) {
  const cells = useMemo(() => {
    return Array.from({ length: cellCount }, (_, i) => i);
  }, [cellCount]);

  return (
    <div
      style={styles.gridLayer}
      className={`gpu-accelerated ${isPaused ? 'animation-paused' : ''}`}
      aria-hidden="true"
    >
      {cells.map((index) => (
        <div
          key={index}
          className="grid-cell"
          style={{
            ...styles.gridCell,
            ['--cell-index' as string]: index % 50, // Limit stagger range
          }}
        />
      ))}
    </div>
  );
});

/**
 * SingleParticle Component
 *
 * Renders an individual floating particle with drift animation.
 */
const SingleParticle = memo(function SingleParticle({
  particle,
  isPaused,
}: {
  particle: Particle;
  isPaused: boolean;
}) {
  const keyframeName = getParticleKeyframeName(particle);

  const particleStyle: CSSProperties = {
    ...styles.particle,
    left: `${particle.x}%`,
    top: `${particle.y}%`,
    width: particle.size,
    height: particle.size,
    opacity: particle.opacity,
    boxShadow: `0 0 ${particle.size * 2}px ${COLORS.accent.orange}40`,
    animation: `${keyframeName} ${particle.duration}s ease-in-out infinite`,
    animationDelay: `${particle.delay}s`,
    animationPlayState: isPaused ? 'paused' : 'running',
  };

  return <div style={particleStyle} aria-hidden="true" />;
});

/**
 * ParticleLayer Component
 *
 * Renders the floating particle effects.
 */
const ParticleLayer = memo(function ParticleLayer({
  particles,
  isPaused,
}: {
  particles: Particle[];
  isPaused: boolean;
}) {
  return (
    <div
      style={styles.particleLayer}
      className="gpu-accelerated"
      aria-hidden="true"
    >
      {particles.map((particle) => (
        <SingleParticle
          key={particle.id}
          particle={particle}
          isPaused={isPaused}
        />
      ))}
    </div>
  );
});

/**
 * Generates CSS keyframes for particle drift animations.
 */
function generateParticleKeyframes(particles: Particle[]): string {
  return particles
    .map((particle) => {
      const keyframeName = getParticleKeyframeName(particle);
      return `
        @keyframes ${keyframeName} {
          0%, 100% {
            transform: translate(0, 0) translateZ(0);
            opacity: ${particle.opacity};
          }
          25% {
            transform: translate(${particle.driftX * 0.5}%, ${particle.driftY}%) translateZ(0);
            opacity: ${particle.opacity * 0.7};
          }
          50% {
            transform: translate(${particle.driftX}%, ${particle.driftY * 0.5}%) translateZ(0);
            opacity: ${particle.opacity * 1.2};
          }
          75% {
            transform: translate(${particle.driftX * 0.3}%, ${-particle.driftY * 0.3}%) translateZ(0);
            opacity: ${particle.opacity * 0.8};
          }
        }
      `;
    })
    .join('\n');
}

/**
 * AnimatedBackground Component
 *
 * Renders a GPU-accelerated animated background with a flickering grid
 * and floating particle effects. Automatically pauses when the tab is
 * not visible and respects the user's reduced motion preference.
 *
 * @example
 * ```tsx
 * // Default usage with both grid and particles
 * <AnimatedBackground />
 *
 * // Grid only
 * <AnimatedBackground showParticles={false} />
 *
 * // Particles only
 * <AnimatedBackground showGrid={false} />
 *
 * // With custom class
 * <AnimatedBackground className="my-custom-class" />
 * ```
 */
export const AnimatedBackground = memo(function AnimatedBackground({
  showGrid = true,
  showParticles = true,
  className = '',
}: AnimatedBackgroundProps) {
  const prefersReducedMotion = useReducedMotion();
  const isPageVisible = usePageVisibility();
  const [containerRef, isInViewport] =
    useIntersectionObserver<HTMLDivElement>();
  const styleRef = useRef<HTMLStyleElement | null>(null);

  // Track viewport dimensions for grid cell calculation
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const [isMobile, setIsMobile] = useState(false);

  // Update dimensions on mount and resize
  useEffect(() => {
    const updateDimensions = (): void => {
      setDimensions({
        width: window.innerWidth,
        height: window.innerHeight,
      });
      setIsMobile(window.innerWidth < CONFIG.MOBILE_BREAKPOINT);
    };

    updateDimensions();
    window.addEventListener('resize', updateDimensions);

    return () => {
      window.removeEventListener('resize', updateDimensions);
    };
  }, []);

  // Calculate grid cell count based on viewport
  const gridCellCount = useMemo(() => {
    if (!showGrid || prefersReducedMotion) return 0;
    const cols = Math.ceil(dimensions.width / CONFIG.GRID_CELL_SIZE);
    const rows = Math.ceil(dimensions.height / CONFIG.GRID_CELL_SIZE);
    return Math.min(cols * rows, CONFIG.MAX_GRID_CELLS);
  }, [dimensions.width, dimensions.height, showGrid, prefersReducedMotion]);

  // Generate particles based on device type
  const particleCount = isMobile
    ? CONFIG.PARTICLE_COUNT_MOBILE
    : CONFIG.PARTICLE_COUNT_DESKTOP;

  const particles = useMemo(() => {
    if (!showParticles || prefersReducedMotion) return [];
    return generateParticles(particleCount);
  }, [showParticles, prefersReducedMotion, particleCount]);

  // Inject particle keyframes into document
  useEffect(() => {
    if (particles.length === 0) {
      // Remove existing style element if particles are disabled
      if (styleRef.current) {
        styleRef.current.remove();
        styleRef.current = null;
      }
      return;
    }

    const keyframes = generateParticleKeyframes(particles);

    if (!styleRef.current) {
      styleRef.current = document.createElement('style');
      styleRef.current.setAttribute('data-animated-background', 'particles');
      document.head.appendChild(styleRef.current);
    }

    styleRef.current.textContent = keyframes;

    return () => {
      if (styleRef.current) {
        styleRef.current.remove();
        styleRef.current = null;
      }
    };
  }, [particles]);

  // Determine if animations should be paused
  // Pause when: page is not visible, reduced motion preferred, or element is off-screen
  const isPaused = !isPageVisible || prefersReducedMotion || !isInViewport;

  // If reduced motion is preferred and nothing to show, render nothing
  if (prefersReducedMotion && !showGrid && !showParticles) {
    return null;
  }

  // Build container class names
  const containerClassName = [
    'gpu-accelerated',
    'contain-layout',
    'contain-paint',
    isPaused ? 'animation-paused' : '',
    className,
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div
      ref={containerRef}
      style={styles.container}
      className={containerClassName}
      role="presentation"
      aria-hidden="true"
    >
      {showGrid && gridCellCount > 0 && (
        <GridLayer isPaused={isPaused} cellCount={gridCellCount} />
      )}
      {showParticles && particles.length > 0 && (
        <ParticleLayer particles={particles} isPaused={isPaused} />
      )}
    </div>
  );
});

export default AnimatedBackground;
