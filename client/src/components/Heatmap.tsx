/**
 * Activity heatmap component for visualizing event frequency over time.
 *
 * Displays a grid of cells where each cell represents one hour, with color
 * intensity indicating the number of events. Supports 7-day and 30-day views
 * with timezone-aware hour alignment.
 *
 * Features:
 * - CSS Grid layout with hours on X-axis and days on Y-axis
 * - Color scale from dark (0 events) to bright green (51+ events)
 * - Toggle between 7-day and 30-day views
 * - Timezone-aware hour bucketing using local time
 * - Cell click filtering to select events from a specific hour
 * - Accessible with proper ARIA labels and keyboard navigation
 */

import type React from 'react';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { AnimatePresence, m } from 'framer-motion';

import { COLORS, SPRING_CONFIGS } from '../constants/design-tokens';
import { useEventStore, type ConnectionStatus } from '../hooks/useEventStore';
import { useReducedMotion } from '../hooks/useReducedMotion';

import type { VibeteaEvent } from '../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Hours to display on X-axis */
const HOURS_IN_DAY = 24;

/** Hour labels to display (abbreviated) */
const HOUR_LABELS = [0, 6, 12, 18] as const;

/** Day name abbreviations */
const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as const;

/** View options for the heatmap */
const VIEW_OPTIONS = [7, 30] as const;

/** Maximum glow intensity (number of stacked events) */
const MAX_GLOW_INTENSITY = 5;

/** Duration in ms for glow decay animation */
const GLOW_DECAY_DURATION_MS = 2000;

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the Heatmap component.
 */
interface HeatmapProps {
  /** Additional CSS classes to apply to the container */
  readonly className?: string;
  /** Callback when a cell is clicked (provides time range for filtering) */
  readonly onCellClick?: (startTime: Date, endTime: Date) => void;
}

/**
 * Number of days to display in the heatmap.
 */
type ViewDays = (typeof VIEW_OPTIONS)[number];

/**
 * Information about a hovered cell for tooltip display.
 */
interface HoveredCell {
  readonly date: string;
  readonly hour: number;
  readonly count: number;
  readonly x: number;
  readonly y: number;
}

/**
 * Represents a single cell in the heatmap grid.
 */
interface HeatmapCell {
  readonly key: string;
  readonly date: Date;
  readonly hour: number;
  readonly count: number;
  readonly dayLabel: string;
  readonly dateLabel: string;
}

/**
 * State for managing glow effects on heatmap cells.
 * Tracks intensity (brightness stacking) and last event time for decay.
 *
 * Glow behavior (FR-003):
 * - Cells animate with orange glow (#d97757 from COLORS.grid.glow) when receiving new events
 * - 2s timer restarts per event
 * - Brightness stacks up to MAX_GLOW_INTENSITY (5) events max
 * - Brightness decays over GLOW_DECAY_DURATION_MS (2000ms) when events stop
 */
interface HeatmapGlowState {
  /** Map of cell key to glow intensity (0-5, where 0 = no glow, 5 = max brightness) */
  readonly intensities: Map<string, number>;
  /** Map of cell key to timestamp of last event (for decay timer management) */
  readonly lastEventTimes: Map<string, number>;
}

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Get the heatmap background color based on event count.
 *
 * Color scale:
 * - 0 events: #1a1a2e (dark)
 * - 1-10 events: #2d4a3e
 * - 11-25 events: #3d6b4f
 * - 26-50 events: #4d8c5f
 * - 51+ events: #5dad6f (bright)
 *
 * @param count - Number of events in the hour bucket
 * @returns CSS color string
 */
function getHeatmapColor(count: number): string {
  if (count === 0) return '#1a1a2e';
  if (count <= 10) return '#2d4a3e';
  if (count <= 25) return '#3d6b4f';
  if (count <= 50) return '#4d8c5f';
  return '#5dad6f';
}

/**
 * Create a bucket key for an event timestamp.
 *
 * Uses local timezone for hour alignment to match user expectations.
 *
 * @param timestamp - RFC 3339 timestamp string
 * @returns Bucket key in format "YYYY-MM-DD-HH"
 */
function getBucketKey(timestamp: string): string {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hour = String(date.getHours()).padStart(2, '0');
  return `${year}-${month}-${day}-${hour}`;
}

/**
 * Create a bucket key from a Date object.
 *
 * @param date - Date object
 * @param hour - Hour (0-23)
 * @returns Bucket key in format "YYYY-MM-DD-HH"
 */
function getBucketKeyFromDate(date: Date, hour: number): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hourStr = String(hour).padStart(2, '0');
  return `${year}-${month}-${day}-${hourStr}`;
}

/**
 * Count events per hour bucket.
 *
 * @param events - Array of VibeTea events
 * @returns Map of bucket keys to event counts
 */
function countEventsByHour(
  events: readonly VibeteaEvent[]
): Map<string, number> {
  const counts = new Map<string, number>();

  for (const event of events) {
    const key = getBucketKey(event.timestamp);
    const currentCount = counts.get(key) ?? 0;
    counts.set(key, currentCount + 1);
  }

  return counts;
}

/**
 * Generate all cells for the heatmap grid.
 *
 * Creates cells for each hour in the specified date range,
 * with most recent day at the bottom.
 *
 * @param days - Number of days to display
 * @param eventCounts - Map of bucket keys to event counts
 * @returns Array of heatmap cells, oldest first (so newest appears at bottom)
 */
function generateHeatmapCells(
  days: ViewDays,
  eventCounts: Map<string, number>
): readonly HeatmapCell[] {
  const cells: HeatmapCell[] = [];
  const now = new Date();

  // Start from (days - 1) days ago, ending with today
  for (let dayOffset = days - 1; dayOffset >= 0; dayOffset--) {
    const date = new Date(now);
    date.setDate(date.getDate() - dayOffset);
    date.setHours(0, 0, 0, 0);

    const dayIndex = date.getDay();
    const dayLabel = DAY_NAMES[dayIndex];
    const dateLabel = date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });

    for (let hour = 0; hour < HOURS_IN_DAY; hour++) {
      const bucketKey = getBucketKeyFromDate(date, hour);
      const count = eventCounts.get(bucketKey) ?? 0;

      cells.push({
        key: bucketKey,
        date,
        hour,
        count,
        dayLabel,
        dateLabel,
      });
    }
  }

  return cells;
}

/**
 * Format a date and hour for display in tooltip.
 *
 * @param date - The date
 * @param hour - The hour (0-23)
 * @returns Formatted string like "Mon, Jan 15 at 14:00"
 */
function formatCellDateTime(date: Date, hour: number): string {
  const dayName = DAY_NAMES[date.getDay()];
  const dateStr = date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  });
  const hourStr = String(hour).padStart(2, '0');
  return `${dayName}, ${dateStr} at ${hourStr}:00`;
}

/**
 * Format an hour for the header label.
 *
 * @param hour - Hour (0-23)
 * @returns Formatted hour string
 */
function formatHourLabel(hour: number): string {
  return String(hour);
}

// -----------------------------------------------------------------------------
// Sub-components
// -----------------------------------------------------------------------------

/**
 * View toggle buttons for switching between 7-day and 30-day views.
 * Uses spring-based hover/focus feedback per FR-007.
 */
function ViewToggle({
  viewDays,
  onViewChange,
  prefersReducedMotion,
}: {
  readonly viewDays: ViewDays;
  readonly onViewChange: (days: ViewDays) => void;
  readonly prefersReducedMotion: boolean;
}) {
  // Spring-based hover animation (FR-007)
  const getHoverProps = (isSelected: boolean) =>
    prefersReducedMotion
      ? undefined
      : {
          scale: 1.05,
          boxShadow: isSelected
            ? '0 0 12px 2px rgba(59, 130, 246, 0.4)'
            : '0 0 8px 2px rgba(156, 163, 175, 0.3)',
          transition: SPRING_CONFIGS.gentle,
        };

  const tapProps = prefersReducedMotion ? undefined : { scale: 0.95 };

  return (
    <div className="flex gap-1" role="group" aria-label="View range selector">
      {VIEW_OPTIONS.map((days) => {
        const isSelected = viewDays === days;
        return (
          <m.button
            key={days}
            type="button"
            onClick={() => onViewChange(days)}
            className={`px-3 py-1.5 text-sm font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900 ${
              isSelected
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
            aria-pressed={isSelected}
            whileHover={getHoverProps(isSelected)}
            whileTap={tapProps}
          >
            {days} Days
          </m.button>
        );
      })}
    </div>
  );
}

/**
 * Hour header labels for the grid.
 */
function HourHeader() {
  return (
    <>
      {/* Empty cell for row label column */}
      <div className="text-xs text-gray-500" />
      {Array.from({ length: HOURS_IN_DAY }, (_, hour) => (
        <div
          key={hour}
          className="text-xs text-gray-500 text-center"
          aria-hidden="true"
        >
          {HOUR_LABELS.includes(hour as (typeof HOUR_LABELS)[number])
            ? formatHourLabel(hour)
            : ''}
        </div>
      ))}
    </>
  );
}

/**
 * Tooltip component for showing cell details on hover.
 * Animated with spring physics for smooth entrance/exit transitions.
 * Respects prefers-reduced-motion by using instant transitions.
 */
function CellTooltip({
  cell,
  prefersReducedMotion,
}: {
  readonly cell: HoveredCell;
  readonly prefersReducedMotion: boolean;
}) {
  const dateTime = formatCellDateTime(new Date(cell.date), cell.hour);
  const eventText = cell.count === 1 ? 'event' : 'events';

  // Use instant transitions when reduced motion is preferred
  const transition = prefersReducedMotion
    ? { duration: 0 }
    : SPRING_CONFIGS.standard;

  // Skip entrance/exit animations for reduced motion
  const animationProps = prefersReducedMotion
    ? {
        initial: { opacity: 1, y: 0, scale: 1 },
        animate: { opacity: 1, y: 0, scale: 1 },
        exit: { opacity: 0, y: 0, scale: 1 },
      }
    : {
        initial: { opacity: 0, y: 8, scale: 0.95 },
        animate: { opacity: 1, y: 0, scale: 1 },
        exit: { opacity: 0, y: 8, scale: 0.95 },
      };

  return (
    <m.div
      className="absolute z-50 px-3 py-2 rounded-lg shadow-xl text-sm pointer-events-none"
      style={{
        left: cell.x,
        top: cell.y,
        transform: 'translate(-50%, -100%) translateY(-8px)',
        backgroundColor: COLORS.background.secondary,
        borderWidth: 1,
        borderStyle: 'solid',
        borderColor: COLORS.background.tertiary,
      }}
      {...animationProps}
      transition={transition}
      role="tooltip"
    >
      <div className="font-medium" style={{ color: COLORS.text.primary }}>
        {cell.count} {eventText}
      </div>
      <div style={{ color: COLORS.text.secondary }}>{dateTime}</div>
    </m.div>
  );
}

/**
 * Convert hex color to RGB components.
 *
 * @param hex - Hex color string (e.g., "#d97757")
 * @returns RGB components as { r, g, b }
 */
function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (result === null) {
    return { r: 217, g: 119, b: 87 }; // Fallback to glow color
  }
  return {
    r: parseInt(result[1] ?? 'D9', 16),
    g: parseInt(result[2] ?? '77', 16),
    b: parseInt(result[3] ?? '57', 16),
  };
}

/**
 * Calculate glow effect styles based on intensity.
 *
 * Uses COLORS.grid.glow (#d97757) for the orange glow effect.
 *
 * @param intensity - Glow intensity (0 to MAX_GLOW_INTENSITY)
 * @returns CSS style object with box-shadow and filter
 */
function getGlowStyles(intensity: number): React.CSSProperties {
  if (intensity <= 0) {
    return {};
  }

  // Normalize intensity to 0-1 range
  const normalizedIntensity =
    Math.min(intensity, MAX_GLOW_INTENSITY) / MAX_GLOW_INTENSITY;

  // Box-shadow spread scales from 4px to 12px based on intensity
  const spread = 4 + normalizedIntensity * 8;

  // Opacity of glow scales from 0.3 to 0.8
  const glowOpacity = 0.3 + normalizedIntensity * 0.5;

  // Brightness boost scales from 1.0 to 1.3
  const brightness = 1.0 + normalizedIntensity * 0.3;

  // Use the glow color from design tokens
  const { r, g, b } = hexToRgb(COLORS.grid.glow);

  return {
    boxShadow: `0 0 ${spread}px rgba(${r}, ${g}, ${b}, ${glowOpacity})`,
    filter: `brightness(${brightness})`,
  };
}

/**
 * Individual heatmap cell component.
 * Uses spring-based hover feedback per FR-007.
 * Respects prefers-reduced-motion by showing static glow and disabling transitions.
 */
function HeatmapCellComponent({
  cell,
  glowIntensity,
  prefersReducedMotion,
  onHover,
  onLeave,
  onClick,
}: {
  readonly cell: HeatmapCell;
  readonly glowIntensity: number;
  readonly prefersReducedMotion: boolean;
  readonly onHover: (cell: HeatmapCell, event: React.MouseEvent) => void;
  readonly onLeave: () => void;
  readonly onClick: (cell: HeatmapCell) => void;
}) {
  const backgroundColor = getHeatmapColor(cell.count);
  const eventText = cell.count === 1 ? 'event' : 'events';
  const ariaLabel = `${cell.count} ${eventText} on ${cell.dateLabel} at ${String(cell.hour).padStart(2, '0')}:00`;

  // When reduced motion is preferred:
  // - Show static max glow if there's any glow intensity (no decay animation)
  // - Skip the brightness filter animation
  const effectiveGlowIntensity =
    prefersReducedMotion && glowIntensity > 0
      ? MAX_GLOW_INTENSITY
      : glowIntensity;

  // Get glow styles, but skip brightness filter for reduced motion
  const glowStyles = getGlowStyles(effectiveGlowIntensity);
  if (prefersReducedMotion && glowStyles.filter !== undefined) {
    // Remove brightness animation for reduced motion
    delete glowStyles.filter;
  }

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onClick(cell);
    }
  };

  // Spring-based hover animation (FR-007)
  const hoverProps = prefersReducedMotion
    ? undefined
    : {
        scale: 1.15,
        boxShadow: `0 0 8px 2px ${COLORS.grid.glow}60`,
        transition: SPRING_CONFIGS.gentle,
      };

  const tapProps = prefersReducedMotion ? undefined : { scale: 0.9 };

  return (
    <m.div
      role="gridcell"
      tabIndex={0}
      aria-label={ariaLabel}
      className="aspect-square rounded-sm cursor-pointer focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-inset"
      style={{ backgroundColor, ...glowStyles }}
      onMouseEnter={(e) => onHover(cell, e)}
      onMouseLeave={onLeave}
      onClick={() => onClick(cell)}
      onKeyDown={handleKeyDown}
      whileHover={hoverProps}
      whileTap={tapProps}
    />
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
        callToAction: 'Activity data will load once connected',
      };
    case 'reconnecting':
      return {
        primary: 'Reconnecting to server...',
        callToAction: 'Activity data will resume once reconnected',
      };
    case 'disconnected':
      return {
        primary: 'Not connected',
        callToAction: 'Click Connect to start tracking activity',
      };
    case 'connected':
    default:
      return {
        primary: 'No activity data',
        callToAction: 'Start a Claude Code session to see activity patterns',
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
      className="flex flex-col items-center justify-center py-12 text-gray-500"
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
          d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
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
 * Activity heatmap displaying event frequency over time.
 *
 * Features:
 * - CSS Grid layout with hours on X-axis, days on Y-axis
 * - Color intensity indicates event count per hour
 * - Toggle between 7-day and 30-day views
 * - Timezone-aware hour alignment (uses local time)
 * - Click cells to filter event stream to that hour
 * - Accessible with ARIA labels and keyboard navigation
 *
 * @example
 * ```tsx
 * // Basic usage
 * <Heatmap />
 *
 * // With cell click handler for filtering
 * <Heatmap
 *   onCellClick={(start, end) => {
 *     console.log(`Filter events from ${start} to ${end}`);
 *   }}
 * />
 *
 * // With custom styling
 * <Heatmap className="p-4 bg-gray-800 rounded-lg" />
 * ```
 */
export function Heatmap({ className = '', onCellClick }: HeatmapProps) {
  // Selective subscription: only re-render when events or connection status change
  const events = useEventStore((state) => state.events);
  const connectionStatus = useEventStore((state) => state.status);

  // Respect user's reduced motion preference (FR-008)
  const prefersReducedMotion = useReducedMotion();

  // State
  const [viewDays, setViewDays] = useState<ViewDays>(7);
  const [hoveredCell, setHoveredCell] = useState<HoveredCell | null>(null);
  const [glowState, setGlowState] = useState<HeatmapGlowState>({
    intensities: new Map(),
    lastEventTimes: new Map(),
  });

  // Refs for tracking previous counts and decay timers
  const prevEventCountsRef = useRef<Map<string, number>>(new Map());
  const decayTimersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(
    new Map()
  );

  // Compute event counts by hour bucket
  const eventCounts = useMemo(() => countEventsByHour(events), [events]);

  /**
   * Start the decay animation for a cell.
   * Gradually reduces intensity over the decay duration.
   */
  const startDecay = useCallback((cellKey: string) => {
    // Decay step: reduce intensity by 1 every 400ms (2000ms / 5 steps)
    const decayStepMs = GLOW_DECAY_DURATION_MS / MAX_GLOW_INTENSITY;

    const decayStep = () => {
      setGlowState((prevState) => {
        const currentIntensity = prevState.intensities.get(cellKey) ?? 0;

        if (currentIntensity <= 0) {
          // Fully decayed, remove from maps
          const newIntensities = new Map(prevState.intensities);
          const newLastEventTimes = new Map(prevState.lastEventTimes);
          newIntensities.delete(cellKey);
          newLastEventTimes.delete(cellKey);
          decayTimersRef.current.delete(cellKey);
          return {
            intensities: newIntensities,
            lastEventTimes: newLastEventTimes,
          };
        }

        // Decrease intensity by 1
        const newIntensities = new Map(prevState.intensities);
        newIntensities.set(cellKey, currentIntensity - 1);

        // Schedule next decay step
        const timer = setTimeout(decayStep, decayStepMs);
        decayTimersRef.current.set(cellKey, timer);

        return {
          intensities: newIntensities,
          lastEventTimes: prevState.lastEventTimes,
        };
      });
    };

    decayStep();
  }, []);

  // Detect new events and update glow state
  useEffect(() => {
    const prevCounts = prevEventCountsRef.current;
    const now = Date.now();
    const cellsWithNewEvents: string[] = [];

    // Find cells that have received new events
    eventCounts.forEach((count, key) => {
      const prevCount = prevCounts.get(key) ?? 0;
      if (count > prevCount) {
        cellsWithNewEvents.push(key);
      }
    });

    // Update glow state for cells with new events
    if (cellsWithNewEvents.length > 0) {
      setGlowState((prevState) => {
        const newIntensities = new Map(prevState.intensities);
        const newLastEventTimes = new Map(prevState.lastEventTimes);

        for (const key of cellsWithNewEvents) {
          // Increase intensity (up to MAX_GLOW_INTENSITY)
          const currentIntensity = newIntensities.get(key) ?? 0;
          const newIntensity = Math.min(
            currentIntensity + 1,
            MAX_GLOW_INTENSITY
          );
          newIntensities.set(key, newIntensity);
          newLastEventTimes.set(key, now);

          // Clear existing decay timer for this cell
          const existingTimer = decayTimersRef.current.get(key);
          if (existingTimer !== undefined) {
            clearTimeout(existingTimer);
          }

          // Set new decay timer
          const timer = setTimeout(() => {
            startDecay(key);
          }, GLOW_DECAY_DURATION_MS);
          decayTimersRef.current.set(key, timer);
        }

        return {
          intensities: newIntensities,
          lastEventTimes: newLastEventTimes,
        };
      });
    }

    // Update prev counts ref
    prevEventCountsRef.current = new Map(eventCounts);
  }, [eventCounts, startDecay]);

  // Cleanup timers on unmount
  useEffect(() => {
    const timers = decayTimersRef.current;
    return () => {
      timers.forEach((timer) => clearTimeout(timer));
      timers.clear();
    };
  }, []);

  // Generate all cells for the current view
  const cells = useMemo(
    () => generateHeatmapCells(viewDays, eventCounts),
    [viewDays, eventCounts]
  );

  // Group cells by day for grid rendering
  const rows = useMemo(() => {
    const result: HeatmapCell[][] = [];
    for (let i = 0; i < cells.length; i += HOURS_IN_DAY) {
      result.push(cells.slice(i, i + HOURS_IN_DAY));
    }
    return result;
  }, [cells]);

  // Check if there are any events
  const hasEvents = events.length > 0;

  /**
   * Handle cell hover to show tooltip.
   */
  const handleCellHover = useCallback(
    (cell: HeatmapCell, event: React.MouseEvent) => {
      const rect = event.currentTarget.getBoundingClientRect();
      const containerRect =
        event.currentTarget.parentElement?.parentElement?.getBoundingClientRect();

      if (containerRect !== undefined) {
        setHoveredCell({
          date: cell.date.toISOString(),
          hour: cell.hour,
          count: cell.count,
          x: rect.left - containerRect.left + rect.width / 2,
          y: rect.top - containerRect.top,
        });
      }
    },
    []
  );

  /**
   * Handle cell hover end.
   */
  const handleCellLeave = useCallback(() => {
    setHoveredCell(null);
  }, []);

  /**
   * Handle cell click to filter events.
   */
  const handleCellClick = useCallback(
    (cell: HeatmapCell) => {
      if (onCellClick === undefined) {
        return;
      }

      // Create start and end times for the hour
      const startTime = new Date(cell.date);
      startTime.setHours(cell.hour, 0, 0, 0);

      const endTime = new Date(cell.date);
      endTime.setHours(cell.hour + 1, 0, 0, 0);

      onCellClick(startTime, endTime);
    },
    [onCellClick]
  );

  /**
   * Handle view toggle.
   */
  const handleViewChange = useCallback((days: ViewDays) => {
    setViewDays(days);
  }, []);

  return (
    <div
      className={`bg-gray-900 text-gray-100 ${className}`}
      role="region"
      aria-label="Activity heatmap"
    >
      {/* Header with title and view toggle */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-gray-100">Activity</h2>
        <ViewToggle viewDays={viewDays} onViewChange={handleViewChange} prefersReducedMotion={prefersReducedMotion} />
      </div>

      {/* Heatmap grid or empty state */}
      {hasEvents ? (
        <div className="relative">
          <div
            role="grid"
            aria-label={`Activity heatmap showing ${viewDays} days of event data`}
            className="grid gap-0.5"
            style={{
              gridTemplateColumns: `auto repeat(${HOURS_IN_DAY}, minmax(0, 1fr))`,
            }}
          >
            {/* Hour header row */}
            <HourHeader />

            {/* Data rows */}
            {rows.map((row, rowIndex) => {
              const firstCell = row[0];
              if (firstCell === undefined) return null;

              // Use abbreviated day names for 7-day view, dates for 30-day view
              const rowLabel =
                viewDays === 7 ? firstCell.dayLabel : firstCell.dateLabel;

              return (
                <div
                  key={firstCell.key}
                  role="row"
                  className="contents"
                  aria-rowindex={rowIndex + 2}
                >
                  {/* Row label */}
                  <div
                    className="text-xs text-gray-500 pr-2 flex items-center justify-end"
                    aria-hidden="true"
                  >
                    {rowLabel}
                  </div>

                  {/* Hour cells */}
                  {row.map((cell) => (
                    <HeatmapCellComponent
                      key={cell.key}
                      cell={cell}
                      glowIntensity={glowState.intensities.get(cell.key) ?? 0}
                      prefersReducedMotion={prefersReducedMotion}
                      onHover={handleCellHover}
                      onLeave={handleCellLeave}
                      onClick={handleCellClick}
                    />
                  ))}
                </div>
              );
            })}
          </div>

          {/* Tooltip */}
          <AnimatePresence>
            {hoveredCell !== null && (
              <CellTooltip
                cell={hoveredCell}
                prefersReducedMotion={prefersReducedMotion}
              />
            )}
          </AnimatePresence>

          {/* Legend */}
          <div className="flex items-center justify-end gap-2 mt-4 text-xs text-gray-500">
            <span>Less</span>
            <div className="flex gap-0.5">
              {[0, 5, 15, 35, 60].map((count) => (
                <div
                  key={count}
                  className="w-3 h-3 rounded-sm"
                  style={{ backgroundColor: getHeatmapColor(count) }}
                  aria-hidden="true"
                />
              ))}
            </div>
            <span>More</span>
          </div>
        </div>
      ) : (
        <EmptyState connectionStatus={connectionStatus} />
      )}
    </div>
  );
}
