/**
 * ASCIIHeader - Animated ASCII art header component (FR-001)
 *
 * Displays pre-generated ASCII art with a spring entrance animation.
 * Respects user's reduced motion preferences for accessibility.
 *
 * @module components/animated/ASCIIHeader
 */

import type { CSSProperties, ReactElement } from 'react';
import { m } from 'framer-motion';

import { VIBETEA_ASCII } from '../../assets/ascii/vibetea-logo';
import { COLORS, SPRING_CONFIGS } from '../../constants/design-tokens';
import { useReducedMotion } from '../../hooks/useReducedMotion';

/**
 * Props for the ASCIIHeader component
 */
interface ASCIIHeaderProps {
  /** Custom ASCII text to display (defaults to VIBETEA_ASCII) */
  text?: string;
  /** Whether to animate on mount (default: true) */
  animateOnLoad?: boolean;
  /** Additional CSS classes */
  className?: string;
}

/**
 * Styles for the ASCII art container
 */
const asciiStyles: CSSProperties = {
  color: COLORS.text.primary,
  fontFamily: 'monospace',
  fontSize: '0.75rem',
  lineHeight: 1.2,
  textAlign: 'center',
  whiteSpace: 'pre',
  margin: 0,
  padding: '1rem 0',
  userSelect: 'none',
};

/**
 * Animation variants for the ASCII header entrance
 */
const animationVariants = {
  hidden: {
    opacity: 0,
    y: -20,
  },
  visible: {
    opacity: 1,
    y: 0,
    transition: SPRING_CONFIGS.expressive,
  },
};

/**
 * Static variants for when animation is disabled
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
 * Animated ASCII art header component for the VibeTea dashboard.
 *
 * Displays ASCII art with a spring-based entrance animation on mount.
 * The animation respects the user's `prefers-reduced-motion` system setting,
 * rendering statically when reduced motion is preferred.
 *
 * @example
 * ```tsx
 * // Default usage with VIBETEA_ASCII logo
 * <ASCIIHeader />
 *
 * // With custom ASCII text
 * <ASCIIHeader text={customAsciiArt} />
 *
 * // Without entrance animation
 * <ASCIIHeader animateOnLoad={false} />
 *
 * // With additional styling
 * <ASCIIHeader className="my-header-class" />
 * ```
 */
export function ASCIIHeader({
  text = VIBETEA_ASCII,
  animateOnLoad = true,
  className,
}: ASCIIHeaderProps): ReactElement {
  const prefersReducedMotion = useReducedMotion();

  // Determine whether to use animated or static variants
  const shouldAnimate = animateOnLoad && !prefersReducedMotion;
  const variants = shouldAnimate ? animationVariants : staticVariants;

  return (
    <m.pre
      className={className}
      style={asciiStyles}
      initial="hidden"
      animate="visible"
      variants={variants}
      role="img"
      aria-label="VibeTea ASCII logo"
    >
      {text}
    </m.pre>
  );
}

export default ASCIIHeader;
