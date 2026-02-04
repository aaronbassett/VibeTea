/**
 * Hook for tracking element visibility using Intersection Observer.
 *
 * Uses the Intersection Observer API to detect when an element enters
 * or exits the viewport. Useful for pausing animations when off-screen,
 * lazy loading content, or triggering effects when elements become visible.
 */

import { type RefObject, useEffect, useRef, useState } from 'react';

/**
 * Options for configuring the Intersection Observer.
 *
 * Mirrors the standard IntersectionObserverInit interface for type safety
 * while avoiding ESLint issues with DOM type recognition.
 */
export interface IntersectionObserverOptions {
  /**
   * A number or array of numbers indicating at what percentage of the
   * target's visibility the observer should trigger.
   * Default: 0 (any visibility)
   */
  threshold?: number | number[];

  /**
   * The element used as the viewport for checking visibility.
   * Default: null (browser viewport)
   */
  root?: Element | Document | null;

  /**
   * Margin around the root element, formatted like CSS margin.
   * Default: '0px'
   */
  rootMargin?: string;
}

/**
 * Checks if the Intersection Observer API is available.
 *
 * Handles SSR environments where `window` is not available
 * by returning `false` as the default.
 *
 * @returns `true` if IntersectionObserver is supported, `false` otherwise
 */
function isIntersectionObserverSupported(): boolean {
  if (typeof window === 'undefined') {
    return false;
  }
  return 'IntersectionObserver' in window;
}

/**
 * Tracks whether an element is intersecting with the viewport or a specified root.
 *
 * Returns a ref to attach to the target element and a boolean indicating
 * whether the element is currently visible. The observer is automatically
 * cleaned up on unmount.
 *
 * When the Intersection Observer API is not available (SSR or unsupported browsers),
 * the hook returns `true` as a safe default to ensure animations and features
 * continue to work.
 *
 * @param options - IntersectionObserver options (see IntersectionObserverOptions)
 *
 * @returns Tuple of [ref, isIntersecting] - Ref to attach to the element, and
 *   a boolean indicating whether the element is currently visible
 *
 * @example
 * ```tsx
 * function AnimatedComponent() {
 *   const [ref, isVisible] = useIntersectionObserver<HTMLDivElement>();
 *
 *   return (
 *     <div ref={ref}>
 *       <Animation isPaused={!isVisible} />
 *     </div>
 *   );
 * }
 * ```
 *
 * @example
 * ```tsx
 * // With custom threshold (50% visibility required)
 * function LazyImage({ src }: { src: string }) {
 *   const [ref, isVisible] = useIntersectionObserver<HTMLImageElement>({
 *     threshold: 0.5,
 *     rootMargin: '100px', // Start loading 100px before entering viewport
 *   });
 *
 *   return (
 *     <img
 *       ref={ref}
 *       src={isVisible ? src : undefined}
 *       alt="Lazy loaded image"
 *     />
 *   );
 * }
 * ```
 */
export function useIntersectionObserver<T extends Element>(
  options?: IntersectionObserverOptions
): [RefObject<T | null>, boolean] {
  const elementRef = useRef<T | null>(null);

  // Default to true for SSR and unsupported browsers to ensure features work
  const [isIntersecting, setIsIntersecting] = useState<boolean>(() => {
    // In SSR or without API support, default to visible
    return !isIntersectionObserverSupported();
  });

  // Memoize options to prevent unnecessary re-observations
  // We use JSON.stringify for comparison since options is an object
  const threshold = options?.threshold;
  const root = options?.root;
  const rootMargin = options?.rootMargin;

  useEffect(() => {
    // Guard for SSR and unsupported browsers
    if (!isIntersectionObserverSupported()) {
      // Keep the default value of true
      return;
    }

    const element = elementRef.current;
    if (element === null) {
      return;
    }

    /**
     * Handler for intersection changes.
     * Updates state when element visibility changes.
     */
    const handleIntersection = (entries: IntersectionObserverEntry[]): void => {
      // We only observe one element, so use the first entry
      const entry = entries[0];
      if (entry !== undefined) {
        setIsIntersecting(entry.isIntersecting);
      }
    };

    // Create observer with provided options
    const observer = new IntersectionObserver(handleIntersection, {
      threshold: threshold ?? 0,
      root: root ?? null,
      rootMargin: rootMargin ?? '0px',
    });

    // Start observing the element
    observer.observe(element);

    // Clean up observer on unmount or when dependencies change
    return () => {
      observer.disconnect();
    };
  }, [threshold, root, rootMargin]);

  return [elementRef, isIntersecting];
}
