/**
 * Hook for managing session timeout logic.
 *
 * Sets up a periodic interval that checks and updates session states
 * based on time thresholds:
 * - Active -> Inactive: After 5 minutes without events
 * - Inactive/Ended -> Removed: After 30 minutes without events
 *
 * This hook should be called once at the app root level (App.tsx).
 */

import { useEffect } from 'react';

import { SESSION_CHECK_INTERVAL_MS, useEventStore } from './useEventStore';

/**
 * Initializes the session timeout checking mechanism.
 *
 * Sets up an interval that periodically calls updateSessionStates()
 * to transition sessions between states based on time thresholds.
 * Cleans up the interval on unmount.
 *
 * @example
 * ```tsx
 * // In App.tsx (call once at root level)
 * export default function App() {
 *   useSessionTimeouts();
 *   return <Dashboard />;
 * }
 * ```
 */
export function useSessionTimeouts(): void {
  const updateSessionStates = useEventStore(
    (state) => state.updateSessionStates
  );

  useEffect(() => {
    // Set up periodic check for session state transitions
    const intervalId = setInterval(() => {
      updateSessionStates();
    }, SESSION_CHECK_INTERVAL_MS);

    // Clean up interval on unmount
    return () => {
      clearInterval(intervalId);
    };
  }, [updateSessionStates]);
}
