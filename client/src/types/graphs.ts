/**
 * VibeTea Graph Data Types
 *
 * TypeScript types for graph and chart components used to visualize
 * event activity and distribution data in the dashboard.
 */

import type { EventType, VibeteaEvent } from './events';

// -----------------------------------------------------------------------------
// Time Range Types
// -----------------------------------------------------------------------------

/**
 * Available time range options for filtering activity data.
 * - '1h': Last 1 hour
 * - '6h': Last 6 hours
 * - '24h': Last 24 hours
 */
export type TimeRange = '1h' | '6h' | '24h';

// -----------------------------------------------------------------------------
// Activity Graph Types
// -----------------------------------------------------------------------------

/**
 * A single data point for the activity graph, representing event count
 * within a specific time bucket.
 */
export interface ActivityDataPoint {
  /**
   * ISO 8601 formatted timestamp for the start of this time bucket.
   */
  readonly timestamp: string;

  /**
   * Number of events that occurred within this time bucket.
   */
  readonly count: number;

  /**
   * Human-readable label for this time bucket (e.g., "2:00 PM", "14:00").
   */
  readonly label: string;
}

/**
 * Props for the ActivityGraph component that displays event activity
 * over time as a time series visualization.
 */
export interface ActivityGraphProps {
  /**
   * Array of VibeTea events to visualize. Uses readonly for immutability.
   */
  readonly events: readonly VibeteaEvent[];

  /**
   * Currently selected time range for filtering and bucketing events.
   */
  readonly timeRange: TimeRange;

  /**
   * Optional callback invoked when the user selects a different time range.
   * @param range - The newly selected time range
   */
  readonly onTimeRangeChange?: (range: TimeRange) => void;
}

// -----------------------------------------------------------------------------
// Event Distribution Chart Types
// -----------------------------------------------------------------------------

/**
 * Distribution data for a single event type, used to render
 * a segment in the event distribution chart.
 */
export interface EventTypeDistribution {
  /**
   * The event type this distribution represents.
   */
  readonly type: EventType;

  /**
   * Total count of events for this type.
   */
  readonly count: number;

  /**
   * Percentage of total events represented by this type (0-100).
   */
  readonly percentage: number;

  /**
   * Color value (hex, rgb, or CSS color name) for rendering
   * this segment in the chart.
   */
  readonly color: string;
}

/**
 * Props for the EventDistributionChart component that displays
 * the breakdown of events by type.
 */
export interface EventDistributionChartProps {
  /**
   * Array of VibeTea events to analyze and visualize. Uses readonly for immutability.
   */
  readonly events: readonly VibeteaEvent[];
}
