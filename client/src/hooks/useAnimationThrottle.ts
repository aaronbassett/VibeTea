/**
 * Hook for throttling animations to prevent performance degradation.
 *
 * Implements timestamp gating to limit entrance animations to a maximum rate,
 * preventing overwhelming the browser when many events arrive simultaneously.
 * Uses a sliding 1-second window to track animation timestamps.
 *
 * Per FR-014, animation state is kept component-local (via useRef) rather than
 * in a global store, ensuring each component independently manages its
 * animation throttling.
 */

import { useRef, useCallback } from 'react';

import { ANIMATION_TIMING } from '../constants/design-tokens';

/**
 * Duration of the sliding window for counting animations (in milliseconds).
 */
const WINDOW_MS = 1000;

/**
 * Throttles animations to a maximum rate per second.
 *
 * This hook provides a `shouldAnimate` function that components can call
 * before starting an animation. The function returns `true` if the animation
 * should proceed, or `false` if it should be skipped due to throttling.
 *
 * The throttling uses a sliding 1-second window: animations are allowed as
 * long as fewer than `maxEntranceAnimationsPerSecond` (default: 10) animations
 * have occurred in the past second. When the limit is reached, subsequent
 * animation requests return `false` until older timestamps fall outside the window.
 *
 * This is critical for FR-018 compliance, preventing animation storms when
 * many events arrive in rapid succession (e.g., burst of WebSocket messages).
 *
 * @returns A function `shouldAnimate()` that returns `true` if the animation
 *          should run, `false` if it should be skipped due to throttling
 *
 * @example
 * ```tsx
 * function EventItem({ event }: { event: Event }) {
 *   const shouldAnimate = useAnimationThrottle();
 *
 *   // Only animate if within throttle limits
 *   const animate = shouldAnimate();
 *
 *   return (
 *     <div
 *       className={animate ? 'animate-fade-in' : ''}
 *     >
 *       {event.message}
 *     </div>
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * function EventList({ events }: { events: Event[] }) {
 *   const shouldAnimate = useAnimationThrottle();
 *
 *   return (
 *     <ul>
 *       {events.map((event) => {
 *         // Each new event checks the throttle
 *         const canAnimate = shouldAnimate();
 *         return (
 *           <li
 *             key={event.id}
 *             style={{
 *               animation: canAnimate ? 'fadeIn 0.3s ease' : 'none',
 *             }}
 *           >
 *             {event.content}
 *           </li>
 *         );
 *       })}
 *     </ul>
 *   );
 * }
 * ```
 */
export function useAnimationThrottle(): () => boolean {
  /**
   * Stores timestamps of recent animations within the sliding window.
   * Using useRef ensures timestamps persist across renders without causing
   * re-renders when updated.
   */
  const timestampsRef = useRef<number[]>([]);

  /**
   * Determines whether an animation should run based on the current throttle state.
   *
   * This function:
   * 1. Removes timestamps older than the 1-second window
   * 2. Checks if the animation count is below the limit
   * 3. If allowed, records the current timestamp and returns true
   * 4. If throttled, returns false without recording
   *
   * @returns `true` if the animation should proceed, `false` if throttled
   */
  const shouldAnimate = useCallback((): boolean => {
    const now = Date.now();
    const timestamps = timestampsRef.current;

    // Remove timestamps older than the sliding window
    const cutoff = now - WINDOW_MS;
    timestampsRef.current = timestamps.filter((t) => t > cutoff);

    // Check if we've hit the throttle limit
    if (
      timestampsRef.current.length >=
      ANIMATION_TIMING.maxEntranceAnimationsPerSecond
    ) {
      return false; // Throttled - render without animation
    }

    // Record this animation and allow it
    timestampsRef.current.push(now);
    return true; // Animate
  }, []);

  return shouldAnimate;
}
