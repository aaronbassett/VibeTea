/**
 * Virtual scrolling event stream component.
 *
 * Displays VibeTea events with efficient rendering using @tanstack/react-virtual,
 * supporting 1000+ events with auto-scroll behavior and jump-to-latest functionality.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { m } from 'framer-motion';

import {
  selectFilteredEvents,
  useEventStore,
  type ConnectionStatus,
} from '../hooks/useEventStore';
import { useAnimationThrottle } from '../hooks/useAnimationThrottle';
import { useReducedMotion } from '../hooks/useReducedMotion';
import { EVENT_TYPE_ICONS } from './icons/EventIcons';
import { ANIMATION_TIMING, SPRING_CONFIGS } from '../constants/design-tokens';

import type { EventType, VibeteaEvent } from '../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Estimated height of each event row in pixels */
const ESTIMATED_ROW_HEIGHT = 64;

/** Distance from bottom (in pixels) to disable auto-scroll when user scrolls up */
const AUTO_SCROLL_THRESHOLD = 50;

/** Color classes for event type badges */
const EVENT_TYPE_COLORS: Record<EventType, string> = {
  tool: 'bg-blue-600/20 text-blue-400 border-blue-500/30',
  activity: 'bg-green-600/20 text-green-400 border-green-500/30',
  session: 'bg-purple-600/20 text-purple-400 border-purple-500/30',
  summary: 'bg-cyan-600/20 text-cyan-400 border-cyan-500/30',
  error: 'bg-red-600/20 text-red-400 border-red-500/30',
  agent: 'bg-amber-600/20 text-amber-400 border-amber-500/30',
};

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the EventStream component.
 */
interface EventStreamProps {
  /** Additional CSS classes to apply to the container. */
  readonly className?: string;
}

/**
 * Tracks the animation state for an individual event in the stream.
 *
 * Used to determine whether an event should play entrance animations based on:
 * - Age threshold: Events older than `eventAnimationMaxAgeMs` (5s) should not animate
 * - Initial load: Events present on first render should not animate
 * - Throttle limits: Respects `maxEntranceAnimationsPerSecond` (10/s) constraint
 *
 * @see ANIMATION_TIMING.eventAnimationMaxAgeMs - 5000ms threshold
 * @see ANIMATION_TIMING.maxEntranceAnimationsPerSecond - 10/s throttle limit
 */
export interface EventAnimationState {
  /** The unique identifier of the event. */
  readonly eventId: string;
  /** Whether the event is within the animation age threshold (< 5 seconds old). */
  readonly shouldAnimate: boolean;
  /** Whether this event was just added (vs present during initial load). */
  readonly isNew: boolean;
}

/**
 * Tracks recently animated events for throttling entrance animations.
 * Maps event IDs to the timestamp when their animation was triggered.
 */
export type AnimationThrottleMap = Map<string, number>;

/**
 * Determines if an event should animate based on its age.
 *
 * @param eventTimestamp - RFC 3339 timestamp of the event
 * @param currentTime - Current time in milliseconds (Date.now())
 * @param maxAgeMs - Maximum age in milliseconds for animation eligibility (default: 5000)
 * @returns true if the event is within the animation age threshold
 *
 * @example
 * ```ts
 * const eventTs = new Date().toISOString();
 * const canAnimate = shouldEventAnimate(eventTs, Date.now(), 5000);
 * // canAnimate === true (just created)
 *
 * // After 6 seconds...
 * const canAnimateLater = shouldEventAnimate(eventTs, Date.now(), 5000);
 * // canAnimateLater === false (too old)
 * ```
 */
export function shouldEventAnimate(
  eventTimestamp: string,
  currentTime: number,
  maxAgeMs: number = 5000
): boolean {
  const eventTime = new Date(eventTimestamp).getTime();
  const ageMs = currentTime - eventTime;
  return ageMs < maxAgeMs && ageMs >= 0;
}

/**
 * Checks if a new animation can be triggered without exceeding the throttle limit.
 *
 * @param throttleMap - Map of recently animated event IDs to their animation timestamps
 * @param currentTime - Current time in milliseconds
 * @param maxPerSecond - Maximum animations allowed per second (default: 10)
 * @returns true if a new animation can be triggered
 *
 * @example
 * ```ts
 * const throttle = new Map<string, number>();
 * if (canTriggerAnimation(throttle, Date.now(), 10)) {
 *   throttle.set(eventId, Date.now());
 *   // trigger animation...
 * }
 * ```
 */
export function canTriggerAnimation(
  throttleMap: AnimationThrottleMap,
  currentTime: number,
  maxPerSecond: number = 10
): boolean {
  // Count animations triggered within the last second
  const oneSecondAgo = currentTime - 1000;
  let recentCount = 0;

  for (const timestamp of throttleMap.values()) {
    if (timestamp > oneSecondAgo) {
      recentCount++;
    }
  }

  return recentCount < maxPerSecond;
}

/**
 * Cleans up old entries from the throttle map to prevent memory leaks.
 * Removes entries older than 1 second.
 *
 * @param throttleMap - Map of recently animated event IDs to their animation timestamps
 * @param currentTime - Current time in milliseconds
 *
 * @example
 * ```ts
 * // Call periodically or before checking throttle
 * cleanupThrottleMap(throttleMap, Date.now());
 * ```
 */
export function cleanupThrottleMap(
  throttleMap: AnimationThrottleMap,
  currentTime: number
): void {
  const oneSecondAgo = currentTime - 1000;

  for (const [eventId, timestamp] of throttleMap.entries()) {
    if (timestamp <= oneSecondAgo) {
      throttleMap.delete(eventId);
    }
  }
}

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Format RFC 3339 timestamp for display.
 *
 * @param timestamp - RFC 3339 formatted timestamp string
 * @returns Formatted time string (HH:MM:SS.mmm)
 */
function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  } catch {
    return timestamp;
  }
}

/**
 * Get a brief description of the event payload.
 *
 * @param event - The VibeTea event
 * @returns A human-readable description
 */
function getEventDescription(event: VibeteaEvent): string {
  const { type, payload } = event;

  switch (type) {
    case 'session': {
      const sessionPayload = payload as VibeteaEvent<'session'>['payload'];
      return `Session ${sessionPayload.action}: ${sessionPayload.project}`;
    }
    case 'activity': {
      const activityPayload = payload as VibeteaEvent<'activity'>['payload'];
      return activityPayload.project !== undefined
        ? `Activity in ${activityPayload.project}`
        : 'Activity heartbeat';
    }
    case 'tool': {
      const toolPayload = payload as VibeteaEvent<'tool'>['payload'];
      return `${toolPayload.tool} ${toolPayload.status}${toolPayload.context !== undefined ? `: ${toolPayload.context}` : ''}`;
    }
    case 'agent': {
      const agentPayload = payload as VibeteaEvent<'agent'>['payload'];
      return `Agent state: ${agentPayload.state}`;
    }
    case 'summary': {
      const summaryPayload = payload as VibeteaEvent<'summary'>['payload'];
      const summary = summaryPayload.summary;
      return summary.length > 80 ? `${summary.slice(0, 80)}...` : summary;
    }
    case 'error': {
      const errorPayload = payload as VibeteaEvent<'error'>['payload'];
      return `Error: ${errorPayload.category}`;
    }
    default:
      return 'Unknown event';
  }
}

// -----------------------------------------------------------------------------
// Sub-components
// -----------------------------------------------------------------------------

/**
 * Props for the EventRow component.
 */
interface EventRowProps {
  /** The event to render. */
  readonly event: VibeteaEvent;
  /** Whether the row should animate on entrance. */
  readonly shouldAnimate: boolean;
  /** Whether user prefers reduced motion. */
  readonly prefersReducedMotion: boolean;
}

/**
 * Renders a single event row with optional entrance animation.
 */
function EventRow({
  event,
  shouldAnimate,
  prefersReducedMotion,
}: EventRowProps) {
  const Icon = EVENT_TYPE_ICONS[event.type];
  const colorClass = EVENT_TYPE_COLORS[event.type];
  const description = getEventDescription(event);
  const formattedTime = formatTimestamp(event.timestamp);

  // Determine if we should actually animate
  const animate = shouldAnimate && !prefersReducedMotion;

  const content = (
    <>
      {/* Event type icon and badge */}
      <div
        className={`flex items-center justify-center gap-2 px-2.5 py-1 rounded-md border min-w-[90px] ${colorClass}`}
      >
        <Icon className="w-4 h-4 flex-shrink-0" aria-hidden="true" />
        <span className="text-xs font-medium capitalize">{event.type}</span>
      </div>

      {/* Event description */}
      <div className="flex-1 min-w-0">
        <p className="text-sm text-[#f5f5f5] truncate">{description}</p>
        <p className="text-xs text-[#6b6b6b] truncate">
          {event.source} | {event.payload.sessionId.slice(0, 8)}...
        </p>
      </div>

      {/* Timestamp */}
      <div className="flex items-center shrink-0 pl-3 ml-2 border-l border-[#2a2a2a]">
        <time
          dateTime={event.timestamp}
          className="text-xs text-[#a0a0a0] font-mono tabular-nums"
        >
          {formattedTime}
        </time>
      </div>
    </>
  );

  const rowClassName =
    'group flex items-center gap-3 px-4 py-3 border-b border-[#1a1a1a] transition-[background-color,border-color] duration-150 hover:bg-[#1a1a1a]/70 hover:border-[#2a2a2a]';

  // If animating, use framer-motion's m.div
  if (animate) {
    return (
      <m.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={SPRING_CONFIGS.standard}
        className={rowClassName}
        role="listitem"
        aria-label={`${event.type} event at ${formattedTime}: ${description}`}
      >
        {content}
      </m.div>
    );
  }

  // No animation - render plain div
  return (
    <div
      className={rowClassName}
      role="listitem"
      aria-label={`${event.type} event at ${formattedTime}: ${description}`}
    >
      {content}
    </div>
  );
}

/**
 * Button to jump back to the latest events.
 * Uses spring-based hover/focus feedback per FR-007.
 */
function JumpToLatestButton({
  onClick,
  newEventCount,
  prefersReducedMotion,
}: {
  readonly onClick: () => void;
  readonly newEventCount: number;
  readonly prefersReducedMotion: boolean;
}) {
  // Spring-based hover animation (FR-007)
  const hoverProps = prefersReducedMotion
    ? undefined
    : {
        scale: 1.05,
        boxShadow: '0 0 16px 4px rgba(59, 130, 246, 0.4)',
        transition: SPRING_CONFIGS.gentle,
      };

  const tapProps = prefersReducedMotion ? undefined : { scale: 0.95 };

  return (
    <m.button
      type="button"
      onClick={onClick}
      className="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-full shadow-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900"
      aria-label={`Jump to latest events. ${newEventCount} new events available.`}
      whileHover={hoverProps}
      whileTap={tapProps}
    >
      <svg
        className="w-4 h-4"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M19 14l-7 7m0 0l-7-7m7 7V3"
        />
      </svg>
      <span>Jump to Latest</span>
      {newEventCount > 0 && (
        <span className="bg-blue-500 px-2 py-0.5 rounded-full text-xs">
          {newEventCount > 99 ? '99+' : newEventCount}
        </span>
      )}
    </m.button>
  );
}

/**
 * Props for the EmptyState component.
 */
interface EmptyStateProps {
  /** Current WebSocket connection status */
  readonly connectionStatus: ConnectionStatus;
}

/**
 * Get context-aware empty state message based on connection status.
 *
 * @param connectionStatus - Current WebSocket connection status
 * @returns Object with primary message and call-to-action text
 */
function getEmptyStateMessages(connectionStatus: ConnectionStatus): {
  readonly primary: string;
  readonly callToAction: string;
} {
  switch (connectionStatus) {
    case 'connecting':
      return {
        primary: 'Connecting to server...',
        callToAction: 'Events will stream in once connected',
      };
    case 'reconnecting':
      return {
        primary: 'Reconnecting to server...',
        callToAction: 'Events will resume once reconnected',
      };
    case 'disconnected':
      return {
        primary: 'Not connected',
        callToAction: 'Click Connect to start receiving events',
      };
    case 'connected':
    default:
      return {
        primary: 'No events yet',
        callToAction:
          'Events will appear here as Claude Code activity is detected',
      };
  }
}

/**
 * Empty state when no events are available.
 * Displays context-aware messages based on connection status.
 */
function EmptyState({ connectionStatus }: EmptyStateProps) {
  const { primary, callToAction } = getEmptyStateMessages(connectionStatus);

  return (
    <div
      className="flex flex-col items-center justify-center h-full text-gray-500"
      role="status"
      aria-live="polite"
    >
      <svg
        className="w-12 h-12 mb-4"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <p className="text-sm font-medium">{primary}</p>
      <p className="text-xs mt-1 text-center max-w-xs">{callToAction}</p>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Main Component
// -----------------------------------------------------------------------------

/**
 * Virtual scrolling event stream for displaying VibeTea events.
 *
 * Features:
 * - Efficient rendering of 1000+ events using virtual scrolling
 * - Auto-scroll to show new events (pauses when user scrolls up 50px+)
 * - Jump to latest button when auto-scroll is paused
 * - Event type icons and color-coded badges
 * - Accessible with proper ARIA attributes
 *
 * @example
 * ```tsx
 * // Basic usage
 * <EventStream />
 *
 * // With custom styling
 * <EventStream className="h-96 border border-gray-700 rounded-lg" />
 * ```
 */
export function EventStream({ className = '' }: EventStreamProps) {
  // Selective subscription: only re-render when filtered events or connection status change
  const events = useEventStore(selectFilteredEvents);
  const connectionStatus = useEventStore((state) => state.status);

  // Animation hooks
  const shouldAnimateThrottle = useAnimationThrottle();
  const prefersReducedMotion = useReducedMotion();

  // Refs
  const parentRef = useRef<HTMLDivElement>(null);
  const previousEventCountRef = useRef<number>(events.length);

  // Track event IDs present on initial mount (these should not animate)
  const initialEventIdsRef = useRef<Set<string> | null>(null);

  // Track which event IDs have already been animated (to avoid re-animating on re-render)
  const animatedEventIdsRef = useRef<Set<string>>(new Set());

  // State
  const [isAutoScrollEnabled, setIsAutoScrollEnabled] = useState<boolean>(true);
  const [newEventCount, setNewEventCount] = useState<number>(0);

  // Initialize the set of initial event IDs on first render
  if (initialEventIdsRef.current === null) {
    initialEventIdsRef.current = new Set(events.map((e) => e.id));
  }

  // Since events are stored newest-first (index 0 is most recent),
  // we reverse the order for display so newest appears at the bottom
  // This matches the natural expectation of a log/stream view
  const displayEvents = [...events].reverse();

  // Virtual scrolling setup
  const virtualizer = useVirtualizer({
    count: displayEvents.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    overscan: 5,
  });

  /**
   * Handle scroll events to detect if user scrolled away from bottom.
   */
  const handleScroll = useCallback(() => {
    const scrollElement = parentRef.current;
    if (scrollElement === null) {
      return;
    }

    const { scrollTop, scrollHeight, clientHeight } = scrollElement;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

    if (distanceFromBottom > AUTO_SCROLL_THRESHOLD) {
      // User scrolled up - disable auto-scroll
      if (isAutoScrollEnabled) {
        setIsAutoScrollEnabled(false);
      }
    } else {
      // User is near the bottom - enable auto-scroll
      if (!isAutoScrollEnabled) {
        setIsAutoScrollEnabled(true);
        setNewEventCount(0);
      }
    }
  }, [isAutoScrollEnabled]);

  /**
   * Jump to the latest (bottom) of the list.
   */
  const handleJumpToLatest = useCallback(() => {
    if (displayEvents.length > 0) {
      virtualizer.scrollToIndex(displayEvents.length - 1, { align: 'end' });
      setIsAutoScrollEnabled(true);
      setNewEventCount(0);
    }
  }, [displayEvents.length, virtualizer]);

  // Auto-scroll to bottom when new events arrive (if enabled)
  useEffect(() => {
    const currentCount = events.length;
    const previousCount = previousEventCountRef.current;

    if (currentCount > previousCount) {
      const addedCount = currentCount - previousCount;

      if (isAutoScrollEnabled) {
        // Scroll to the bottom (last item in displayed list)
        virtualizer.scrollToIndex(displayEvents.length - 1, { align: 'end' });
      } else {
        // Track new events while auto-scroll is disabled
        setNewEventCount((prev) => prev + addedCount);
      }
    }

    previousEventCountRef.current = currentCount;
  }, [events.length, isAutoScrollEnabled, displayEvents.length, virtualizer]);

  // Attach scroll listener
  useEffect(() => {
    const scrollElement = parentRef.current;
    if (scrollElement === null) {
      return;
    }

    scrollElement.addEventListener('scroll', handleScroll, { passive: true });
    return () => scrollElement.removeEventListener('scroll', handleScroll);
  }, [handleScroll]);

  // Handle empty state
  if (displayEvents.length === 0) {
    return (
      <div
        className={`relative bg-gray-900 text-gray-100 overflow-hidden ${className}`}
        role="log"
        aria-label="Event stream"
        aria-live="polite"
      >
        <EmptyState connectionStatus={connectionStatus} />
      </div>
    );
  }

  return (
    <div
      className={`relative bg-gray-900 text-gray-100 overflow-hidden ${className}`}
      role="log"
      aria-label="Event stream"
      aria-live="polite"
    >
      {/* Scrollable container */}
      <div
        ref={parentRef}
        className="h-full overflow-auto"
        role="list"
        aria-label={`${displayEvents.length} events`}
      >
        {/* Virtual content container */}
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {virtualizer.getVirtualItems().map((virtualItem) => {
            const event = displayEvents[virtualItem.index];
            if (event === undefined) {
              return null;
            }

            // Determine if this event should animate
            // An event should animate if:
            // 1. It was NOT present on initial load
            // 2. It's within the age threshold (< 5 seconds old)
            // 3. It hasn't already been animated
            // 4. Animation throttle allows it
            let eventShouldAnimate = false;
            const isInitialEvent =
              initialEventIdsRef.current?.has(event.id) ?? true;
            const hasBeenAnimated = animatedEventIdsRef.current.has(event.id);

            if (!isInitialEvent && !hasBeenAnimated) {
              const isWithinAgeThreshold = shouldEventAnimate(
                event.timestamp,
                Date.now(),
                ANIMATION_TIMING.eventAnimationMaxAgeMs
              );

              if (isWithinAgeThreshold && shouldAnimateThrottle()) {
                eventShouldAnimate = true;
                // Mark this event as animated so we don't re-animate on re-render
                animatedEventIdsRef.current.add(event.id);
              }
            }

            return (
              <div
                key={event.id}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                <EventRow
                  event={event}
                  shouldAnimate={eventShouldAnimate}
                  prefersReducedMotion={prefersReducedMotion}
                />
              </div>
            );
          })}
        </div>
      </div>

      {/* Jump to latest button (shown when auto-scroll is disabled) */}
      {!isAutoScrollEnabled && (
        <JumpToLatestButton
          onClick={handleJumpToLatest}
          newEventCount={newEventCount}
          prefersReducedMotion={prefersReducedMotion}
        />
      )}
    </div>
  );
}
