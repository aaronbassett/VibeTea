/**
 * Hook for detecting user's reduced motion preference.
 *
 * Listens to the `prefers-reduced-motion` media query and returns a boolean
 * indicating whether the user prefers reduced motion. Updates reactively
 * when the system preference changes.
 *
 * Use this hook to conditionally disable or simplify animations for users
 * who have enabled reduced motion in their operating system settings.
 */

import { useEffect, useState } from 'react';

/**
 * Media query string for detecting reduced motion preference.
 */
const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)';

/**
 * Detects whether the user prefers reduced motion.
 *
 * This hook subscribes to the `prefers-reduced-motion` media query and
 * returns `true` when the user has enabled reduced motion in their system
 * settings. The value updates automatically when the preference changes.
 *
 * Handles SSR gracefully by returning `false` when `window` is not available.
 *
 * @returns `true` if the user prefers reduced motion, `false` otherwise
 *
 * @example
 * ```tsx
 * function AnimatedComponent() {
 *   const prefersReducedMotion = useReducedMotion();
 *
 *   return (
 *     <div
 *       style={{
 *         transition: prefersReducedMotion ? 'none' : 'transform 0.3s ease',
 *       }}
 *     >
 *       Content
 *     </div>
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * function FadeInComponent() {
 *   const prefersReducedMotion = useReducedMotion();
 *
 *   // Skip animation entirely for reduced motion users
 *   if (prefersReducedMotion) {
 *     return <div>Content</div>;
 *   }
 *
 *   return <FadeIn><div>Content</div></FadeIn>;
 * }
 * ```
 */
export function useReducedMotion(): boolean {
  // Initialize with false for SSR compatibility
  // The actual value will be set in useEffect when running in the browser
  const [prefersReducedMotion, setPrefersReducedMotion] = useState<boolean>(
    () => {
      // Check if window is available (browser environment)
      if (typeof window === 'undefined') {
        return false;
      }
      // Get initial value from media query
      return window.matchMedia(REDUCED_MOTION_QUERY).matches;
    }
  );

  useEffect(() => {
    // Guard for SSR - window may not be available
    if (typeof window === 'undefined') {
      return;
    }

    const mediaQuery = window.matchMedia(REDUCED_MOTION_QUERY);

    /**
     * Handler for media query change events.
     * Updates state when user toggles their system preference.
     */
    const handleChange = (event: MediaQueryListEvent): void => {
      setPrefersReducedMotion(event.matches);
    };

    // Subscribe to changes
    mediaQuery.addEventListener('change', handleChange);

    // Clean up listener on unmount
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, []);

  return prefersReducedMotion;
}
