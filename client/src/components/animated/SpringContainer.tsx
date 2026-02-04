/**
 * SpringContainer - Reusable spring-animated wrapper component
 *
 * Provides spring-based entrance animations for child content with
 * configurable spring physics. Respects user's reduced motion preferences.
 *
 * @module components/animated/SpringContainer
 */

import type { ReactElement, ReactNode } from 'react';
import { m } from 'framer-motion';

import { SPRING_CONFIGS } from '../../constants/design-tokens';
import { useReducedMotion } from '../../hooks/useReducedMotion';

/**
 * Available HTML elements for the SpringContainer wrapper.
 */
type ContainerElement =
  | 'div'
  | 'section'
  | 'article'
  | 'aside'
  | 'main'
  | 'header'
  | 'footer';

/**
 * Available spring configuration presets.
 */
type SpringType = 'expressive' | 'standard' | 'gentle';

/**
 * Props for the SpringContainer component.
 */
export interface SpringContainerProps {
  /** Content to render inside the animated container */
  children: ReactNode;
  /** Spring configuration to use (default: 'standard') */
  springType?: SpringType;
  /** Animation delay in seconds (default: 0) */
  delay?: number;
  /** Whether to animate on mount (default: true) */
  animateOnMount?: boolean;
  /** Additional CSS classes */
  className?: string;
  /** HTML element to render as (default: 'div') */
  as?: ContainerElement;
}

/**
 * Y offset in pixels for the entrance animation.
 */
const ENTRANCE_Y_OFFSET = 20;

/**
 * Creates animation variants for the spring container.
 *
 * @param springType - The spring configuration preset to use
 * @param delay - Animation delay in seconds
 * @returns Animation variants object for framer-motion
 */
function createAnimationVariants(springType: SpringType, delay: number) {
  const springConfig = SPRING_CONFIGS[springType];

  return {
    hidden: {
      opacity: 0,
      y: ENTRANCE_Y_OFFSET,
    },
    visible: {
      opacity: 1,
      y: 0,
      transition: {
        ...springConfig,
        delay,
      },
    },
  };
}

/**
 * Static variants for when animation is disabled (reduced motion preference).
 */
const staticVariants = {
  hidden: {
    opacity: 1,
    y: 0,
  },
  visible: {
    opacity: 1,
    y: 0,
  },
};

/**
 * Mapping of HTML element types to their corresponding framer-motion components.
 */
const motionComponents = {
  div: m.div,
  section: m.section,
  article: m.article,
  aside: m.aside,
  main: m.main,
  header: m.header,
  footer: m.footer,
} as const;

/**
 * SpringContainer - Reusable wrapper component for spring-based entrance animations.
 *
 * Wraps children in a framer-motion component that animates with a spring physics
 * entrance animation (fade in from below). Automatically respects the user's
 * `prefers-reduced-motion` system setting, rendering children statically when
 * reduced motion is preferred.
 *
 * The component supports three spring presets:
 * - `expressive`: Bouncy, energetic feel (stiffness: 260, damping: 20)
 * - `standard`: Balanced, smooth transitions (stiffness: 300, damping: 30)
 * - `gentle`: Soft, subtle animations (stiffness: 120, damping: 14)
 *
 * @example
 * ```tsx
 * // Default usage with standard spring
 * <SpringContainer>
 *   <div>Animated content</div>
 * </SpringContainer>
 *
 * // With expressive spring and delay
 * <SpringContainer springType="expressive" delay={0.2}>
 *   <Card>Dashboard panel</Card>
 * </SpringContainer>
 *
 * // As a section element with custom class
 * <SpringContainer as="section" className="dashboard-section">
 *   <DashboardContent />
 * </SpringContainer>
 *
 * // Disable mount animation (renders statically)
 * <SpringContainer animateOnMount={false}>
 *   <StaticContent />
 * </SpringContainer>
 * ```
 */
export function SpringContainer({
  children,
  springType = 'standard',
  delay = 0,
  animateOnMount = true,
  className,
  as = 'div',
}: SpringContainerProps): ReactElement {
  const prefersReducedMotion = useReducedMotion();

  // Determine whether to use animated or static variants
  const shouldAnimate = animateOnMount && !prefersReducedMotion;
  const variants = shouldAnimate
    ? createAnimationVariants(springType, delay)
    : staticVariants;

  // Get the appropriate motion component for the specified element type
  const MotionComponent = motionComponents[as];

  return (
    <MotionComponent
      className={className}
      initial="hidden"
      animate="visible"
      variants={variants}
    >
      {children}
    </MotionComponent>
  );
}

export default SpringContainer;
