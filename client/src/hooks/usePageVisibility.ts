/**
 * Hook for tracking browser tab visibility state.
 *
 * Uses the Page Visibility API to detect when the user switches
 * browser tabs or minimizes the window. Useful for pausing
 * animations, reducing network requests, or saving state when
 * the user is not actively viewing the page.
 */

import { useEffect, useState } from 'react';

/**
 * Safely determines if the page is currently visible.
 *
 * Handles SSR environments where `document` is not available
 * by returning `true` as the default visibility state.
 *
 * @returns `true` if the page is visible or in SSR context, `false` otherwise
 */
function getIsPageVisible(): boolean {
  if (typeof document === 'undefined') {
    return true;
  }
  return !document.hidden;
}

/**
 * Tracks whether the browser tab is visible or hidden.
 *
 * Returns a boolean that updates when the user switches tabs,
 * minimizes the browser, or otherwise changes page visibility.
 * Handles SSR gracefully by assuming the page is visible when
 * `document` is not available.
 *
 * @returns `true` if the page is currently visible, `false` if hidden
 *
 * @example
 * ```tsx
 * function VideoPlayer() {
 *   const isVisible = usePageVisibility();
 *
 *   useEffect(() => {
 *     if (!isVisible) {
 *       pauseVideo();
 *     }
 *   }, [isVisible]);
 *
 *   return <video src="..." />;
 * }
 * ```
 *
 * @example
 * ```tsx
 * // Reduce API polling when tab is hidden
 * function Dashboard() {
 *   const isVisible = usePageVisibility();
 *
 *   useEffect(() => {
 *     if (!isVisible) return;
 *
 *     const intervalId = setInterval(fetchData, 5000);
 *     return () => clearInterval(intervalId);
 *   }, [isVisible]);
 *
 *   return <DashboardContent />;
 * }
 * ```
 */
export function usePageVisibility(): boolean {
  const [isVisible, setIsVisible] = useState(getIsPageVisible);

  useEffect(() => {
    // Skip event listener setup in SSR environment
    if (typeof document === 'undefined') {
      return;
    }

    const handleVisibilityChange = (): void => {
      setIsVisible(!document.hidden);
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    // Clean up event listener on unmount
    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  return isVisible;
}
