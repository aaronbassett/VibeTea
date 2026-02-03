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
import { useCallback, useMemo, useState } from 'react';

import { useEventStore } from '../hooks/useEventStore';

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
 */
function ViewToggle({
  viewDays,
  onViewChange,
}: {
  readonly viewDays: ViewDays;
  readonly onViewChange: (days: ViewDays) => void;
}) {
  return (
    <div className="flex gap-1" role="group" aria-label="View range selector">
      {VIEW_OPTIONS.map((days) => (
        <button
          key={days}
          type="button"
          onClick={() => onViewChange(days)}
          className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900 ${
            viewDays === days
              ? 'bg-blue-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }`}
          aria-pressed={viewDays === days}
        >
          {days} Days
        </button>
      ))}
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
 */
function CellTooltip({ cell }: { readonly cell: HoveredCell }) {
  const dateTime = formatCellDateTime(new Date(cell.date), cell.hour);
  const eventText = cell.count === 1 ? 'event' : 'events';

  return (
    <div
      className="absolute z-50 px-3 py-2 bg-gray-800 border border-gray-700 rounded-lg shadow-xl text-sm pointer-events-none"
      style={{
        left: cell.x,
        top: cell.y,
        transform: 'translate(-50%, -100%) translateY(-8px)',
      }}
      role="tooltip"
    >
      <div className="font-medium text-white">
        {cell.count} {eventText}
      </div>
      <div className="text-gray-400">{dateTime}</div>
    </div>
  );
}

/**
 * Individual heatmap cell component.
 */
function HeatmapCellComponent({
  cell,
  onHover,
  onLeave,
  onClick,
}: {
  readonly cell: HeatmapCell;
  readonly onHover: (cell: HeatmapCell, event: React.MouseEvent) => void;
  readonly onLeave: () => void;
  readonly onClick: (cell: HeatmapCell) => void;
}) {
  const backgroundColor = getHeatmapColor(cell.count);
  const eventText = cell.count === 1 ? 'event' : 'events';
  const ariaLabel = `${cell.count} ${eventText} on ${cell.dateLabel} at ${String(cell.hour).padStart(2, '0')}:00`;

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onClick(cell);
    }
  };

  return (
    <div
      role="gridcell"
      tabIndex={0}
      aria-label={ariaLabel}
      className="aspect-square rounded-sm cursor-pointer transition-transform hover:scale-110 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-inset"
      style={{ backgroundColor }}
      onMouseEnter={(e) => onHover(cell, e)}
      onMouseLeave={onLeave}
      onClick={() => onClick(cell)}
      onKeyDown={handleKeyDown}
    />
  );
}

/**
 * Empty state when no events are available.
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
          d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
        />
      </svg>
      <p className="text-sm">No activity data</p>
      <p className="text-xs mt-1">Events will appear here as they occur</p>
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
  // Selective subscription: only re-render when events change
  const events = useEventStore((state) => state.events);

  // State
  const [viewDays, setViewDays] = useState<ViewDays>(7);
  const [hoveredCell, setHoveredCell] = useState<HoveredCell | null>(null);

  // Compute event counts by hour bucket
  const eventCounts = useMemo(() => countEventsByHour(events), [events]);

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
        <ViewToggle viewDays={viewDays} onViewChange={handleViewChange} />
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
          {hoveredCell !== null && <CellTooltip cell={hoveredCell} />}

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
        <EmptyState />
      )}
    </div>
  );
}
