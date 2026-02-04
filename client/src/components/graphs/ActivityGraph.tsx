/**
 * Activity graph component for visualizing event frequency over time.
 *
 * Displays a line/area chart using Recharts that shows event counts bucketed
 * by time intervals based on the selected time range. Uses the warm color
 * palette from design tokens with animated transitions.
 *
 * Features:
 * - Area chart with gradient fill using orange accent color
 * - Time bucketing based on selected range (1h, 6h, 24h)
 * - Responsive container for flexible sizing
 * - Animated transitions on data changes (respects reduced motion)
 * - Accessible with proper ARIA labels
 *
 * @module components/graphs/ActivityGraph
 */

import type React from 'react';
import { useMemo } from 'react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';
import { LazyMotion, domAnimation, m } from 'framer-motion';

import { COLORS, SPRING_CONFIGS } from '../../constants/design-tokens';
import { useReducedMotion } from '../../hooks/useReducedMotion';
import type { ConnectionStatus } from '../../hooks/useEventStore';

import type {
  ActivityGraphProps,
  ActivityDataPoint,
  TimeRange,
} from '../../types/graphs';
import type { VibeteaEvent } from '../../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/**
 * Time bucket configurations for each time range.
 * Each range produces 12 data points for consistent visualization.
 */
const BUCKET_CONFIGS: Record<
  TimeRange,
  { bucketSizeMs: number; count: number }
> = {
  '1h': { bucketSizeMs: 5 * 60 * 1000, count: 12 }, // 5-minute buckets
  '6h': { bucketSizeMs: 30 * 60 * 1000, count: 12 }, // 30-minute buckets
  '24h': { bucketSizeMs: 2 * 60 * 60 * 1000, count: 12 }, // 2-hour buckets
};

/**
 * Duration in milliseconds for each time range.
 */
const TIME_RANGE_DURATION_MS: Record<TimeRange, number> = {
  '1h': 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
};

/**
 * Animation duration for chart transitions in milliseconds.
 */
const ANIMATION_DURATION_MS = 300;

/**
 * Unique ID for the gradient definition (to avoid conflicts if multiple charts exist).
 */
const GRADIENT_ID = 'activityGradient';

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Format a timestamp for display based on the time range.
 *
 * @param timestamp - ISO 8601 timestamp string
 * @param timeRange - Currently selected time range
 * @returns Formatted time label (e.g., "2:00 PM" for 1h, "14:00" for longer ranges)
 */
function formatTimeLabel(timestamp: string, timeRange: TimeRange): string {
  const date = new Date(timestamp);

  if (timeRange === '1h') {
    // For 1-hour range, show minutes too: "2:05 PM"
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  }

  if (timeRange === '6h') {
    // For 6-hour range, show time with hour and AM/PM
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    });
  }

  // For 24-hour range, show 24-hour format
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  });
}

/**
 * Bucket events by time interval based on the selected time range.
 *
 * Creates time buckets from (now - range duration) to now, counting events
 * that fall within each bucket.
 *
 * @param events - Array of VibeTea events to bucket
 * @param timeRange - Currently selected time range
 * @returns Array of activity data points with counts per bucket
 */
function bucketEventsByTime(
  events: readonly VibeteaEvent[],
  timeRange: TimeRange
): readonly ActivityDataPoint[] {
  const config = BUCKET_CONFIGS[timeRange];
  const rangeDurationMs = TIME_RANGE_DURATION_MS[timeRange];
  const now = Date.now();
  const startTime = now - rangeDurationMs;

  // Initialize buckets with zero counts
  const buckets: ActivityDataPoint[] = [];

  for (let i = 0; i < config.count; i++) {
    const bucketStart = startTime + i * config.bucketSizeMs;
    const timestamp = new Date(bucketStart).toISOString();
    const label = formatTimeLabel(timestamp, timeRange);

    buckets.push({
      timestamp,
      count: 0,
      label,
    });
  }

  // Count events into buckets
  for (const event of events) {
    const eventTime = new Date(event.timestamp).getTime();

    // Skip events outside the time range
    if (eventTime < startTime || eventTime > now) {
      continue;
    }

    // Calculate which bucket this event belongs to
    const bucketIndex = Math.floor(
      (eventTime - startTime) / config.bucketSizeMs
    );

    // Ensure index is within bounds
    if (bucketIndex >= 0 && bucketIndex < config.count) {
      const bucket = buckets[bucketIndex];
      if (bucket !== undefined) {
        // Create a new object to maintain immutability
        buckets[bucketIndex] = {
          ...bucket,
          count: bucket.count + 1,
        };
      }
    }
  }

  return buckets;
}

// -----------------------------------------------------------------------------
// Sub-components
// -----------------------------------------------------------------------------

/**
 * Custom tooltip component for the area chart.
 * Displays time and event count in a styled tooltip.
 */
interface CustomTooltipProps {
  readonly active?: boolean;
  readonly payload?: readonly { readonly value: number }[];
  readonly label?: string;
}

function CustomTooltip({ active, payload, label }: CustomTooltipProps) {
  if (!active || payload === undefined || payload.length === 0) {
    return null;
  }

  const count = payload[0]?.value ?? 0;
  const eventText = count === 1 ? 'event' : 'events';

  return (
    <div
      style={{
        backgroundColor: COLORS.background.secondary,
        borderWidth: 1,
        borderStyle: 'solid',
        borderColor: COLORS.background.tertiary,
        borderRadius: 8,
        padding: '8px 12px',
        boxShadow: `0 4px 12px rgba(0, 0, 0, 0.4)`,
      }}
    >
      <p
        style={{
          color: COLORS.text.primary,
          fontWeight: 500,
          margin: 0,
          marginBottom: 2,
        }}
      >
        {count} {eventText}
      </p>
      <p
        style={{
          color: COLORS.text.secondary,
          fontSize: '0.875rem',
          margin: 0,
        }}
      >
        {label}
      </p>
    </div>
  );
}

/**
 * Props for the EmptyState component.
 */
interface EmptyStateComponentProps {
  /** Currently selected time range */
  readonly timeRange: TimeRange;
  /** WebSocket connection status */
  readonly connectionStatus: ConnectionStatus;
}

/**
 * Get context-aware empty state message based on connection status and time range.
 *
 * @param connectionStatus - Current WebSocket connection status
 * @param timeRange - Currently selected time range
 * @returns Object with primary message and call-to-action text
 */
function getEmptyStateMessages(
  connectionStatus: ConnectionStatus,
  timeRange: TimeRange
): {
  readonly primary: string;
  readonly callToAction: string;
} {
  const rangeText =
    timeRange === '1h' ? 'hour' : timeRange === '6h' ? '6 hours' : '24 hours';

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
        primary: `No activity in the last ${rangeText}`,
        callToAction: 'Start a Claude Code session to see activity trends',
      };
  }
}

/**
 * Empty state when no events are available for the selected time range.
 * Displays context-aware messages based on connection status.
 */
function EmptyState({ timeRange, connectionStatus }: EmptyStateComponentProps) {
  const { primary, callToAction } = getEmptyStateMessages(
    connectionStatus,
    timeRange
  );

  return (
    <div
      className="flex flex-col items-center justify-center h-full min-h-[200px]"
      style={{ color: COLORS.text.muted }}
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
          d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
        />
      </svg>
      <p className="text-sm font-medium">{primary}</p>
      <p className="text-xs mt-1 text-center max-w-xs">{callToAction}</p>
    </div>
  );
}

/**
 * Available time range options for the toggle.
 */
const TIME_RANGE_OPTIONS: readonly TimeRange[] = ['1h', '6h', '24h'];

/**
 * Display labels for each time range option.
 */
const TIME_RANGE_LABELS: Record<TimeRange, string> = {
  '1h': '1h',
  '6h': '6h',
  '24h': '24h',
};

/**
 * Props for the TimeRangeToggle sub-component.
 */
interface TimeRangeToggleProps {
  /** Currently selected time range. */
  readonly timeRange: TimeRange;
  /** Callback when a different time range is selected. */
  readonly onTimeRangeChange?: (range: TimeRange) => void;
  /** Whether user prefers reduced motion. */
  readonly prefersReducedMotion: boolean;
}

/**
 * Segmented control for selecting the time range.
 *
 * Displays a pill-shaped toggle with 1h, 6h, and 24h options.
 * Uses spring animation for the selection indicator and hover feedback (FR-007).
 * Fully keyboard accessible with proper focus states.
 */
function TimeRangeToggle({
  timeRange,
  onTimeRangeChange,
  prefersReducedMotion,
}: TimeRangeToggleProps) {
  // Calculate the selected option index for positioning the indicator
  const selectedIndex = TIME_RANGE_OPTIONS.indexOf(timeRange);

  /**
   * Handle option click.
   */
  const handleOptionClick = (range: TimeRange) => {
    if (onTimeRangeChange !== undefined) {
      onTimeRangeChange(range);
    }
  };

  /**
   * Handle keyboard navigation within the toggle group.
   */
  const handleKeyDown = (event: React.KeyboardEvent, index: number) => {
    if (onTimeRangeChange === undefined) return;

    let newIndex = index;

    switch (event.key) {
      case 'ArrowLeft':
      case 'ArrowUp':
        event.preventDefault();
        newIndex = index === 0 ? TIME_RANGE_OPTIONS.length - 1 : index - 1;
        break;
      case 'ArrowRight':
      case 'ArrowDown':
        event.preventDefault();
        newIndex = index === TIME_RANGE_OPTIONS.length - 1 ? 0 : index + 1;
        break;
      case 'Home':
        event.preventDefault();
        newIndex = 0;
        break;
      case 'End':
        event.preventDefault();
        newIndex = TIME_RANGE_OPTIONS.length - 1;
        break;
      default:
        return;
    }

    const newRange = TIME_RANGE_OPTIONS[newIndex];
    if (newRange !== undefined) {
      onTimeRangeChange(newRange);
      // Focus the new option
      const newButton = event.currentTarget.parentElement?.querySelector(
        `[data-range-index="${newIndex}"]`
      ) as HTMLButtonElement | null;
      newButton?.focus();
    }
  };

  // Spring-based hover animation for unselected options (FR-007)
  const getHoverProps = (isSelected: boolean, isDisabled: boolean) =>
    prefersReducedMotion || isDisabled || isSelected
      ? undefined
      : {
          scale: 1.1,
          transition: SPRING_CONFIGS.gentle,
        };

  const getTapProps = (isDisabled: boolean) =>
    prefersReducedMotion || isDisabled ? undefined : { scale: 0.95 };

  return (
    <LazyMotion features={domAnimation}>
      <div
        role="radiogroup"
        aria-label="Select time range"
        className="relative inline-flex rounded-full p-0.5"
        style={{
          backgroundColor: COLORS.background.tertiary,
        }}
      >
        {/* Animated selection indicator */}
        {prefersReducedMotion ? (
          <span
            className="absolute top-0.5 bottom-0.5 rounded-full"
            style={{
              backgroundColor: COLORS.accent.orange,
              width: `calc((100% - 4px) / ${TIME_RANGE_OPTIONS.length})`,
              left: `calc(2px + (100% - 4px) / ${TIME_RANGE_OPTIONS.length} * ${selectedIndex})`,
            }}
            aria-hidden="true"
          />
        ) : (
          <m.span
            className="absolute top-0.5 bottom-0.5 rounded-full"
            style={{
              backgroundColor: COLORS.accent.orange,
              width: `calc((100% - 4px) / ${TIME_RANGE_OPTIONS.length})`,
            }}
            animate={{
              left: `calc(2px + (100% - 4px) / ${TIME_RANGE_OPTIONS.length} * ${selectedIndex})`,
            }}
            transition={SPRING_CONFIGS.standard}
            aria-hidden="true"
          />
        )}

        {/* Toggle options */}
        {TIME_RANGE_OPTIONS.map((range, index) => {
          const isSelected = range === timeRange;
          const isDisabled = onTimeRangeChange === undefined;

          return (
            <m.button
              key={range}
              type="button"
              role="radio"
              aria-checked={isSelected}
              data-range-index={index}
              disabled={isDisabled}
              tabIndex={isSelected ? 0 : -1}
              onClick={() => handleOptionClick(range)}
              onKeyDown={(e) => handleKeyDown(e, index)}
              className="relative z-10 px-3 py-1 text-xs font-medium rounded-full focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-1 disabled:cursor-not-allowed disabled:opacity-50"
              style={{
                color: isSelected ? COLORS.text.primary : COLORS.text.secondary,
                // Focus ring uses the accent color
                // @ts-expect-error CSS custom property for focus ring
                '--tw-ring-color': COLORS.accent.orange,
                '--tw-ring-offset-color': COLORS.background.tertiary,
              }}
              whileHover={getHoverProps(isSelected, isDisabled)}
              whileTap={getTapProps(isDisabled)}
            >
              {TIME_RANGE_LABELS[range]}
            </m.button>
          );
        })}
      </div>
    </LazyMotion>
  );
}

// -----------------------------------------------------------------------------
// Main Component
// -----------------------------------------------------------------------------

/**
 * Activity graph displaying event frequency over time as an area chart.
 *
 * Visualizes event counts in time buckets based on the selected time range:
 * - 1h: 5-minute buckets (12 points)
 * - 6h: 30-minute buckets (12 points)
 * - 24h: 2-hour buckets (12 points)
 *
 * Uses the warm color palette with orange accent for the area fill and stroke.
 * Respects user's reduced motion preference by disabling animations when set.
 *
 * @example
 * ```tsx
 * // Basic usage
 * <ActivityGraph events={events} timeRange="1h" />
 *
 * // With time range change callback
 * <ActivityGraph
 *   events={events}
 *   timeRange={selectedRange}
 *   onTimeRangeChange={setSelectedRange}
 * />
 * ```
 */
export function ActivityGraph({
  events,
  timeRange,
  onTimeRangeChange,
  connectionStatus = 'connected',
}: ActivityGraphProps) {
  // Respect user's reduced motion preference
  const prefersReducedMotion = useReducedMotion();

  // Bucket events by time
  const data = useMemo(
    () => bucketEventsByTime(events, timeRange),
    [events, timeRange]
  );

  // Check if there are any events in the current time range
  const hasEvents = useMemo(
    () => data.some((point) => point.count > 0),
    [data]
  );

  // Calculate the maximum count for Y-axis domain
  const maxCount = useMemo(() => {
    const max = Math.max(...data.map((point) => point.count));
    // Ensure minimum of 5 for Y-axis to avoid cramped display
    return Math.max(max, 5);
  }, [data]);

  // Animation settings based on reduced motion preference
  const animationDuration = prefersReducedMotion ? 0 : ANIMATION_DURATION_MS;

  return (
    <div
      className="relative w-full h-full min-h-[200px]"
      role="img"
      aria-label={`Activity graph showing event counts over the last ${timeRange === '1h' ? 'hour' : timeRange === '6h' ? '6 hours' : '24 hours'}`}
      style={{ backgroundColor: COLORS.background.secondary }}
    >
      {/* Time range toggle in top-right corner */}
      <div className="absolute top-2 right-2 z-10">
        <TimeRangeToggle
          timeRange={timeRange}
          onTimeRangeChange={onTimeRangeChange}
          prefersReducedMotion={prefersReducedMotion}
        />
      </div>

      {hasEvents ? (
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={data as ActivityDataPoint[]}
            margin={{ top: 40, right: 20, left: 0, bottom: 20 }}
            accessibilityLayer
          >
            {/* Gradient definition for area fill */}
            <defs>
              <linearGradient id={GRADIENT_ID} x1="0" y1="0" x2="0" y2="1">
                <stop
                  offset="0%"
                  stopColor={COLORS.accent.orange}
                  stopOpacity={0.6}
                />
                <stop
                  offset="95%"
                  stopColor={COLORS.accent.orange}
                  stopOpacity={0.05}
                />
              </linearGradient>
            </defs>

            {/* X-axis with time labels */}
            <XAxis
              dataKey="label"
              axisLine={{ stroke: COLORS.grid.line }}
              tickLine={{ stroke: COLORS.grid.line }}
              tick={{ fill: COLORS.text.secondary, fontSize: 12 }}
              interval="preserveStartEnd"
              minTickGap={30}
            />

            {/* Y-axis with event counts */}
            <YAxis
              axisLine={{ stroke: COLORS.grid.line }}
              tickLine={{ stroke: COLORS.grid.line }}
              tick={{ fill: COLORS.text.secondary, fontSize: 12 }}
              domain={[0, maxCount]}
              allowDecimals={false}
              width={40}
            />

            {/* Tooltip */}
            <Tooltip
              content={<CustomTooltip />}
              cursor={{ stroke: COLORS.accent.orange, strokeOpacity: 0.3 }}
            />

            {/* Area chart with gradient fill */}
            <Area
              type="monotone"
              dataKey="count"
              stroke={COLORS.accent.orange}
              strokeWidth={2}
              fill={`url(#${GRADIENT_ID})`}
              animationDuration={animationDuration}
              animationEasing="ease-out"
              isAnimationActive={!prefersReducedMotion}
              style={{
                filter: `drop-shadow(0 0 4px ${COLORS.grid.glow})`,
              }}
            />
          </AreaChart>
        </ResponsiveContainer>
      ) : (
        <EmptyState timeRange={timeRange} connectionStatus={connectionStatus} />
      )}
    </div>
  );
}
