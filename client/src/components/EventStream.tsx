/**
 * Virtual scrolling event stream component.
 *
 * Displays VibeTea events with efficient rendering using @tanstack/react-virtual,
 * supporting 1000+ events with auto-scroll behavior and jump-to-latest functionality.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';

import { useEventStore } from '../hooks/useEventStore';

import type { EventType, VibeteaEvent } from '../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Estimated height of each event row in pixels */
const ESTIMATED_ROW_HEIGHT = 64;

/** Distance from bottom (in pixels) to disable auto-scroll when user scrolls up */
const AUTO_SCROLL_THRESHOLD = 50;

/** Icon mapping for each event type */
const EVENT_TYPE_ICONS: Record<EventType, string> = {
  tool: '\u{1F527}', // 🔧
  activity: '\u{1F4AC}', // 💬
  session: '\u{1F680}', // 🚀
  summary: '\u{1F4CB}', // 📋
  error: '\u{26A0}\u{FE0F}', // ⚠️
  agent: '\u{1F916}', // 🤖
};

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
 * Renders a single event row.
 */
function EventRow({ event }: { readonly event: VibeteaEvent }) {
  const icon = EVENT_TYPE_ICONS[event.type];
  const colorClass = EVENT_TYPE_COLORS[event.type];
  const description = getEventDescription(event);
  const formattedTime = formatTimestamp(event.timestamp);

  return (
    <div
      className="group flex items-center gap-3 px-4 py-3 hover:bg-gray-800/50 transition-colors border-b border-gray-800/50"
      role="listitem"
      aria-label={`${event.type} event at ${formattedTime}: ${description}`}
    >
      {/* Event type icon and badge */}
      <div
        className={`flex items-center gap-2 px-2 py-1 rounded-md border ${colorClass}`}
      >
        <span className="text-base" aria-hidden="true">
          {icon}
        </span>
        <span className="text-xs font-medium capitalize">{event.type}</span>
      </div>

      {/* Event description */}
      <div className="flex-1 min-w-0">
        <p className="text-sm text-gray-100 truncate">{description}</p>
        <p className="text-xs text-gray-500 truncate">
          {event.source} | {event.payload.sessionId.slice(0, 8)}...
        </p>
      </div>

      {/* Timestamp */}
      <time
        dateTime={event.timestamp}
        className="text-xs text-gray-400 font-mono shrink-0"
      >
        {formattedTime}
      </time>
    </div>
  );
}

/**
 * Button to jump back to the latest events.
 */
function JumpToLatestButton({
  onClick,
  newEventCount,
}: {
  readonly onClick: () => void;
  readonly newEventCount: number;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-full shadow-lg transition-all focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900"
      aria-label={`Jump to latest events. ${newEventCount} new events available.`}
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
    </button>
  );
}

/**
 * Empty state when no events are available.
 */
function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center h-full text-gray-500">
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
      <p className="text-sm">No events yet</p>
      <p className="text-xs mt-1">Events will appear here when received</p>
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
  // Selective subscription: only re-render when events change
  const events = useEventStore((state) => state.events);

  // Refs
  const parentRef = useRef<HTMLDivElement>(null);
  const previousEventCountRef = useRef<number>(events.length);

  // State
  const [isAutoScrollEnabled, setIsAutoScrollEnabled] = useState<boolean>(true);
  const [newEventCount, setNewEventCount] = useState<number>(0);

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
        <EmptyState />
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
                <EventRow event={event} />
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
        />
      )}
    </div>
  );
}
