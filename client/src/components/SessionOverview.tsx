/**
 * Session overview component displaying active AI assistant sessions.
 *
 * Shows session cards with project information, duration, activity indicators,
 * and status badges. Supports filtering events by clicking on a session card.
 *
 * Features:
 * - Real-time activity indicators with pulse animation based on event volume
 * - Session status badges (Active, Idle, Ended)
 * - Session duration tracking
 * - Dimmed styling for inactive/ended sessions
 * - Accessible with proper ARIA labels and keyboard navigation
 */

import type React from 'react';
import { useCallback, useMemo } from 'react';
import { m, AnimatePresence } from 'framer-motion';

import { useEventStore } from '../hooks/useEventStore';
import { formatDuration, formatRelativeTime } from '../utils/formatting';
import { SPRING_CONFIGS } from '../constants/design-tokens';

import type { Session, SessionStatus, VibeteaEvent } from '../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Window for counting recent events (60 seconds) */
const RECENT_EVENT_WINDOW_MS = 60 * 1000;

/** Low activity threshold (1-5 events = 1Hz pulse) */
const LOW_ACTIVITY_THRESHOLD = 5;

/** Medium activity threshold (6-15 events = 2Hz pulse) */
const MEDIUM_ACTIVITY_THRESHOLD = 15;

/** Pulse animation classes for different activity levels */
const PULSE_ANIMATIONS = {
  none: '',
  low: 'animate-pulse-slow', // 1Hz
  medium: 'animate-pulse-medium', // 2Hz
  high: 'animate-pulse-fast', // 3Hz
} as const;

/** Status badge configuration */
const STATUS_CONFIG: Record<
  SessionStatus,
  { readonly label: string; readonly badgeClass: string }
> = {
  active: {
    label: 'Active',
    badgeClass: 'bg-green-600/20 text-green-400 border-green-500/30',
  },
  inactive: {
    label: 'Idle',
    badgeClass: 'bg-yellow-600/20 text-yellow-400 border-yellow-500/30',
  },
  ended: {
    label: 'Ended',
    badgeClass: 'bg-gray-600/20 text-gray-400 border-gray-500/30',
  },
};

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the SessionOverview component.
 */
interface SessionOverviewProps {
  /** Additional CSS classes to apply to the container */
  readonly className?: string;
  /** Callback when a session card is clicked */
  readonly onSessionClick?: (sessionId: string) => void;
  /** Currently selected session ID for filtering */
  readonly selectedSessionId?: string | null;
}

/**
 * Props for the SessionCard component.
 */
interface SessionCardProps {
  /** Session data to display */
  readonly session: Session;
  /** Number of events in the last 60 seconds */
  readonly recentEventCount: number;
  /** Callback when the card is clicked */
  readonly onClick?: (sessionId: string) => void;
  /** Whether this session is currently selected for filtering */
  readonly isSelected?: boolean;
}

/**
 * Props for the ActivityIndicator component.
 */
interface ActivityIndicatorProps {
  /** Number of recent events to determine pulse rate */
  readonly recentEventCount: number;
  /** Whether the session is active */
  readonly isActive: boolean;
}

/**
 * Activity level based on recent event count.
 */
type ActivityLevel = 'none' | 'low' | 'medium' | 'high';

/**
 * Animation phases for session card transitions.
 */
export type SessionAnimationPhase = 'entering' | 'idle' | 'exiting' | 'statusChange';

/**
 * Animation states for session cards.
 *
 * Tracks the current animation phase, previous status for transitions,
 * and hover state for interactive animations.
 */
export interface SessionCardAnimationState {
  /** Current animation phase */
  readonly phase: SessionAnimationPhase;
  /** Previous status (for status change animation) */
  readonly previousStatus: SessionStatus | null;
  /** Whether hover state is active */
  readonly isHovered: boolean;
}

// -----------------------------------------------------------------------------
// Animation Variants
// -----------------------------------------------------------------------------

/**
 * Animation variants for session card entry/exit/layout transitions.
 * Uses the expressive spring config (stiffness: 260, damping: 20) from design tokens.
 */
const cardVariants = {
  initial: { opacity: 0, y: 20, scale: 0.95 },
  animate: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: SPRING_CONFIGS.expressive,
  },
  exit: {
    opacity: 0,
    y: -10,
    scale: 0.95,
    transition: { duration: 0.2 },
  },
};

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Determine the activity level based on recent event count.
 *
 * @param recentEventCount - Number of events in the last 60 seconds
 * @param isActive - Whether the session is currently active
 * @returns Activity level for pulse animation
 */
function getActivityLevel(
  recentEventCount: number,
  isActive: boolean
): ActivityLevel {
  // No pulse for inactive sessions or no recent events
  if (!isActive || recentEventCount === 0) {
    return 'none';
  }

  // 1-5 events: 1Hz pulse (slow)
  if (recentEventCount <= LOW_ACTIVITY_THRESHOLD) {
    return 'low';
  }

  // 6-15 events: 2Hz pulse (medium)
  if (recentEventCount <= MEDIUM_ACTIVITY_THRESHOLD) {
    return 'medium';
  }

  // 16+ events: 3Hz pulse (fast)
  return 'high';
}

/**
 * Calculate the session duration from start to now.
 *
 * @param startedAt - When the session started
 * @returns Duration in milliseconds
 */
function getSessionDuration(startedAt: Date): number {
  return Date.now() - startedAt.getTime();
}

/**
 * Sort sessions: active first, then by lastEventAt descending.
 *
 * @param sessions - Array of sessions to sort
 * @returns Sorted array of sessions
 */
function sortSessions(sessions: readonly Session[]): Session[] {
  return [...sessions].sort((a, b) => {
    // Active sessions come first
    if (a.status === 'active' && b.status !== 'active') return -1;
    if (a.status !== 'active' && b.status === 'active') return 1;

    // Then inactive before ended
    if (a.status === 'inactive' && b.status === 'ended') return -1;
    if (a.status === 'ended' && b.status === 'inactive') return 1;

    // Within same status, sort by lastEventAt descending (most recent first)
    return b.lastEventAt.getTime() - a.lastEventAt.getTime();
  });
}

/**
 * Count recent events per session within the specified time window.
 *
 * Uses the most recent event's timestamp as the reference point to maintain
 * pure render behavior. This provides a stable approximation of "recent"
 * events since the store updates frequently with new events.
 *
 * @param events - Array of events to analyze (newest first)
 * @param windowMs - Time window in milliseconds
 * @returns Map of session IDs to event counts
 */
function countRecentEventsBySession(
  events: readonly VibeteaEvent[],
  windowMs: number
): Map<string, number> {
  const counts = new Map<string, number>();

  // Use the most recent event's timestamp as reference (events are sorted newest first)
  if (events.length === 0) {
    return counts;
  }

  const mostRecentEvent = events[0];
  if (mostRecentEvent === undefined) {
    return counts;
  }

  const referenceTime = new Date(mostRecentEvent.timestamp).getTime();

  for (const event of events) {
    const eventTime = new Date(event.timestamp).getTime();
    const age = referenceTime - eventTime;

    if (age <= windowMs && age >= 0) {
      const sessionId = event.payload.sessionId;
      const currentCount = counts.get(sessionId) ?? 0;
      counts.set(sessionId, currentCount + 1);
    }
  }

  return counts;
}

// -----------------------------------------------------------------------------
// Sub-components
// -----------------------------------------------------------------------------

/**
 * Pulsing activity indicator dot.
 *
 * Pulse rate varies based on event volume:
 * - 1-5 events/min: 1Hz pulse
 * - 6-15 events/min: 2Hz pulse
 * - 16+ events/min: 3Hz pulse
 * - No events for 60s: no pulse
 */
function ActivityIndicator({
  recentEventCount,
  isActive,
}: ActivityIndicatorProps) {
  const activityLevel = getActivityLevel(recentEventCount, isActive);
  const pulseClass = PULSE_ANIMATIONS[activityLevel];

  // Base indicator styling
  const baseClass = isActive ? 'bg-green-500' : 'bg-gray-500';

  return (
    <span
      className={`inline-block h-2.5 w-2.5 rounded-full ${baseClass} ${pulseClass}`}
      aria-hidden="true"
    />
  );
}

/**
 * Status badge showing session state.
 */
function StatusBadge({ status }: { readonly status: SessionStatus }) {
  const config = STATUS_CONFIG[status];

  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-md border text-xs font-medium ${config.badgeClass}`}
    >
      {config.label}
    </span>
  );
}

/**
 * Individual session card component.
 *
 * Displays project name, source, duration, activity indicator, and status.
 * For inactive sessions, shows "Last active: X minutes ago".
 */
function SessionCard({
  session,
  recentEventCount,
  onClick,
  isSelected = false,
}: SessionCardProps) {
  const isActive = session.status === 'active';
  const isEnded = session.status === 'ended';
  const isDimmed = !isActive && !isSelected;

  // Animated glow class for active sessions
  const glowClass = isActive ? 'session-card-glow-active' : '';

  // Calculate display values
  const duration = getSessionDuration(session.startedAt);
  const formattedDuration = formatDuration(duration);
  const lastActiveTime = formatRelativeTime(session.lastEventAt.toISOString());

  /**
   * Handle card click.
   */
  const handleClick = useCallback(() => {
    onClick?.(session.sessionId);
  }, [onClick, session.sessionId]);

  /**
   * Handle keyboard interaction (Enter/Space).
   */
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        onClick?.(session.sessionId);
      }
    },
    [onClick, session.sessionId]
  );

  // Card opacity based on status
  const opacityClass = isDimmed ? 'opacity-70' : 'opacity-100';

  // Border and background styling for selected state
  const selectedClass = isSelected
    ? 'border-purple-500 bg-purple-900/20'
    : 'border-gray-700 bg-gray-800';

  // Hover styling only when clickable
  const hoverClass =
    onClick !== undefined
      ? 'cursor-pointer hover:bg-gray-700/50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-inset'
      : '';

  return (
    <m.div
      role="listitem"
      tabIndex={onClick !== undefined ? 0 : undefined}
      aria-label={`${session.project} session, ${STATUS_CONFIG[session.status].label}, duration ${formattedDuration}${isSelected ? ', selected' : ''}`}
      aria-selected={isSelected}
      className={`border rounded-lg p-4 transition-colors ${selectedClass} ${opacityClass} ${hoverClass} ${glowClass}`}
      onClick={onClick !== undefined ? handleClick : undefined}
      onKeyDown={onClick !== undefined ? handleKeyDown : undefined}
      variants={cardVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      layout
      whileHover={
        onClick !== undefined
          ? {
              scale: 1.02,
              boxShadow: '0 0 12px 2px rgba(217, 119, 87, 0.3)',
              transition: SPRING_CONFIGS.gentle,
            }
          : undefined
      }
    >
      {/* Header row: Project name and status badge */}
      <div className="flex items-start justify-between gap-2 mb-2">
        <h3 className="text-sm font-semibold text-gray-100 truncate flex-1">
          {session.project}
        </h3>
        <StatusBadge status={session.status} />
      </div>

      {/* Source identifier */}
      <p className="text-xs text-gray-500 truncate mb-3">{session.source}</p>

      {/* Footer row: Activity indicator, duration, and last active time */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <ActivityIndicator
            recentEventCount={recentEventCount}
            isActive={isActive}
          />
          <span className="text-xs text-gray-400 font-mono">
            {formattedDuration}
          </span>
        </div>

        {/* Show "Last active" for inactive/ended sessions */}
        {!isActive && !isEnded && (
          <span className="text-xs text-gray-500">
            Last active: {lastActiveTime}
          </span>
        )}

        {/* Show event count for active sessions */}
        {isActive && (
          <span className="text-xs text-gray-500">
            {session.eventCount} event{session.eventCount !== 1 ? 's' : ''}
          </span>
        )}
      </div>
    </m.div>
  );
}

/**
 * Empty state when no sessions are available.
 */
function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-12 text-gray-500">
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
          d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
        />
      </svg>
      <p className="text-sm">No active sessions</p>
      <p className="text-xs mt-1">Sessions will appear here when detected</p>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Main Component
// -----------------------------------------------------------------------------

/**
 * Session overview component displaying AI assistant sessions.
 *
 * Subscribes to the Zustand store to display all sessions with real-time
 * activity indicators. Sessions are sorted with active first, then by
 * most recent activity.
 *
 * Features:
 * - Real-time activity indicators with variable pulse rates
 * - Session duration and last active time tracking
 * - Status badges (Active, Idle, Ended)
 * - Dimmed styling for inactive/ended sessions
 * - Click to filter events by session
 * - Accessible with ARIA labels and keyboard navigation
 *
 * @example
 * ```tsx
 * // Basic usage
 * <SessionOverview />
 *
 * // With click handler for filtering
 * <SessionOverview
 *   onSessionClick={(sessionId) => {
 *     console.log(`Filter to session: ${sessionId}`);
 *   }}
 * />
 *
 * // With custom styling
 * <SessionOverview className="p-4 bg-gray-800 rounded-lg" />
 * ```
 */
export function SessionOverview({
  className = '',
  onSessionClick,
  selectedSessionId,
}: SessionOverviewProps) {
  // Subscribe to sessions from the store
  const sessions = useEventStore((state) => state.sessions);
  const events = useEventStore((state) => state.events);

  // Convert sessions Map to sorted array
  const sortedSessions = useMemo(() => {
    const sessionArray = Array.from(sessions.values());
    return sortSessions(sessionArray);
  }, [sessions]);

  // Calculate recent event counts for each session
  // Uses the most recent event's timestamp as the reference point for "recent"
  // events to maintain pure render behavior.
  const recentEventCounts = useMemo(
    () => countRecentEventsBySession(events, RECENT_EVENT_WINDOW_MS),
    [events]
  );

  // Handle session card click
  const handleSessionClick = useCallback(
    (sessionId: string) => {
      onSessionClick?.(sessionId);
    },
    [onSessionClick]
  );

  // Check if there are any sessions
  const hasSessions = sortedSessions.length > 0;

  return (
    <div
      className={`bg-gray-900 text-gray-100 ${className}`}
      role="region"
      aria-label="Session overview"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-gray-100">Sessions</h2>
        {hasSessions && (
          <span className="text-sm text-gray-500">
            {sortedSessions.length} session
            {sortedSessions.length !== 1 ? 's' : ''}
          </span>
        )}
      </div>

      {/* Sessions list or empty state */}
      {hasSessions ? (
        <div role="list" aria-label="Active sessions" className="space-y-3">
          <AnimatePresence mode="popLayout">
            {sortedSessions.map((session) => (
              <SessionCard
                key={session.sessionId}
                session={session}
                recentEventCount={recentEventCounts.get(session.sessionId) ?? 0}
                onClick={
                  onSessionClick !== undefined ? handleSessionClick : undefined
                }
                isSelected={selectedSessionId === session.sessionId}
              />
            ))}
          </AnimatePresence>
        </div>
      ) : (
        <EmptyState />
      )}
    </div>
  );
}
