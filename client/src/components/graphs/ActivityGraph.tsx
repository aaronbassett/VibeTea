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

import { useMemo } from 'react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

import { COLORS } from '../../constants/design-tokens';
import { useReducedMotion } from '../../hooks/useReducedMotion';

import type { ActivityGraphProps, ActivityDataPoint, TimeRange } from '../../types/graphs';
import type { VibeteaEvent } from '../../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/**
 * Time bucket configurations for each time range.
 * Each range produces 12 data points for consistent visualization.
 */
const BUCKET_CONFIGS: Record<TimeRange, { bucketSizeMs: number; count: number }> = {
  '1h': { bucketSizeMs: 5 * 60 * 1000, count: 12 },    // 5-minute buckets
  '6h': { bucketSizeMs: 30 * 60 * 1000, count: 12 },   // 30-minute buckets
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
    const bucketStart = startTime + (i * config.bucketSizeMs);
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
    const bucketIndex = Math.floor((eventTime - startTime) / config.bucketSizeMs);

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
 * Empty state when no events are available for the selected time range.
 */
function EmptyState({ timeRange }: { readonly timeRange: TimeRange }) {
  const rangeText = timeRange === '1h' ? 'hour' : timeRange === '6h' ? '6 hours' : '24 hours';

  return (
    <div
      className="flex flex-col items-center justify-center h-full min-h-[200px]"
      style={{ color: COLORS.text.muted }}
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
      <p className="text-sm">No activity in the last {rangeText}</p>
      <p className="text-xs mt-1">Events will appear here as they occur</p>
    </div>
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
export function ActivityGraph({ events, timeRange }: ActivityGraphProps) {
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
      className="w-full h-full min-h-[200px]"
      role="img"
      aria-label={`Activity graph showing event counts over the last ${timeRange === '1h' ? 'hour' : timeRange === '6h' ? '6 hours' : '24 hours'}`}
      style={{ backgroundColor: COLORS.background.secondary }}
    >
      {hasEvents ? (
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={data as ActivityDataPoint[]}
            margin={{ top: 20, right: 20, left: 0, bottom: 20 }}
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
        <EmptyState timeRange={timeRange} />
      )}
    </div>
  );
}
