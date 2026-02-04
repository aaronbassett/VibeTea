/**
 * Event distribution chart component for visualizing event type breakdown.
 *
 * Displays a donut chart using Recharts that shows the distribution of events
 * by type. Uses the warm color palette from design tokens with distinct colors
 * for each event type.
 *
 * Features:
 * - Donut chart with animated transitions
 * - Custom tooltip showing type, count, and percentage
 * - Legend displaying all event types
 * - Center label showing total event count
 * - Respects reduced motion preference
 * - Accessible with proper ARIA labels
 *
 * @module components/graphs/EventDistributionChart
 */

import { useMemo } from 'react';
import {
  PieChart,
  Pie,
  Cell,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from 'recharts';

import { COLORS } from '../../constants/design-tokens';
import { useReducedMotion } from '../../hooks/useReducedMotion';
import type { ConnectionStatus } from '../../hooks/useEventStore';

import type {
  EventDistributionChartProps,
  EventTypeDistribution,
} from '../../types/graphs';
import type { EventType, VibeteaEvent } from '../../types/events';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/**
 * Color mapping for each event type.
 * Uses the warm palette with distinct colors for visual differentiation.
 */
const EVENT_TYPE_COLORS: Record<EventType, string> = {
  session: COLORS.accent.orange,
  activity: COLORS.accent.orangeLight,
  tool: '#64b5f6', // Blue for tools - contrast
  agent: '#81c784', // Green for agents
  summary: '#ba68c8', // Purple for summaries
  error: COLORS.status.error,
  // Enhanced tracking event colors
  agent_spawn: '#34d399', // Emerald for agent spawn
  skill_invocation: '#a78bfa', // Violet for skills
  token_usage: '#fcd34d', // Amber for tokens
  session_metrics: '#818cf8', // Indigo for metrics
  activity_pattern: '#2dd4bf', // Teal for patterns
  model_distribution: '#fb923c', // Orange for distribution
  todo_progress: '#84cc16', // Lime for progress
  file_change: '#f472b6', // Pink for file changes
  project_activity: '#38bdf8', // Sky for project activity
};

/**
 * Display labels for each event type.
 */
const EVENT_TYPE_LABELS: Record<EventType, string> = {
  session: 'Session',
  activity: 'Activity',
  tool: 'Tool',
  agent: 'Agent',
  summary: 'Summary',
  error: 'Error',
  // Enhanced tracking event labels
  agent_spawn: 'Agent Spawn',
  skill_invocation: 'Skill',
  token_usage: 'Token Usage',
  session_metrics: 'Metrics',
  activity_pattern: 'Pattern',
  model_distribution: 'Model Dist.',
  todo_progress: 'Todo',
  file_change: 'File Change',
  project_activity: 'Project',
};

/**
 * All event types in display order.
 */
const EVENT_TYPES: readonly EventType[] = [
  'session',
  'activity',
  'tool',
  'agent',
  'summary',
  'error',
  // Enhanced tracking event types
  'agent_spawn',
  'skill_invocation',
  'token_usage',
  'session_metrics',
  'activity_pattern',
  'model_distribution',
  'todo_progress',
  'file_change',
  'project_activity',
];

/**
 * Animation duration for chart transitions in milliseconds.
 */
const ANIMATION_DURATION_MS = 400;

/**
 * Unique ID prefix for the gradient definitions.
 */
const GRADIENT_ID_PREFIX = 'distributionGradient';

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Calculate distribution of events by type.
 *
 * Groups events by their type and calculates count and percentage for each.
 *
 * @param events - Array of VibeTea events to analyze
 * @returns Array of EventTypeDistribution objects with count and percentage
 */
function calculateDistribution(
  events: readonly VibeteaEvent[]
): readonly EventTypeDistribution[] {
  const totalCount = events.length;

  if (totalCount === 0) {
    return [];
  }

  // Count events by type
  const countsByType = new Map<EventType, number>();

  for (const event of events) {
    const currentCount = countsByType.get(event.type) ?? 0;
    countsByType.set(event.type, currentCount + 1);
  }

  // Build distribution array with all types that have events
  const distribution: EventTypeDistribution[] = [];

  for (const type of EVENT_TYPES) {
    const count = countsByType.get(type);
    if (count !== undefined && count > 0) {
      const percentage = (count / totalCount) * 100;
      distribution.push({
        type,
        count,
        percentage: Math.round(percentage * 10) / 10, // Round to 1 decimal
        color: EVENT_TYPE_COLORS[type],
      });
    }
  }

  // Sort by count descending for better visual hierarchy
  distribution.sort((a, b) => b.count - a.count);

  return distribution;
}

// -----------------------------------------------------------------------------
// Sub-components
// -----------------------------------------------------------------------------

/**
 * Custom tooltip component for the donut chart.
 * Displays event type, count, and percentage.
 */
interface CustomTooltipProps {
  readonly active?: boolean;
  readonly payload?: readonly {
    readonly payload: EventTypeDistribution;
  }[];
}

function CustomTooltip({ active, payload }: CustomTooltipProps) {
  if (!active || payload === undefined || payload.length === 0) {
    return null;
  }

  const data = payload[0]?.payload;
  if (data === undefined) {
    return null;
  }

  const label = EVENT_TYPE_LABELS[data.type];
  const eventText = data.count === 1 ? 'event' : 'events';

  return (
    <div
      style={{
        backgroundColor: COLORS.background.secondary,
        borderWidth: 1,
        borderStyle: 'solid',
        borderColor: COLORS.background.tertiary,
        borderRadius: 8,
        padding: '8px 12px',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.4)',
      }}
    >
      <div className="flex items-center gap-2 mb-1">
        <span
          className="w-3 h-3 rounded-full"
          style={{ backgroundColor: data.color }}
          aria-hidden="true"
        />
        <span
          style={{
            color: COLORS.text.primary,
            fontWeight: 500,
          }}
        >
          {label}
        </span>
      </div>
      <p
        style={{
          color: COLORS.text.secondary,
          fontSize: '0.875rem',
          margin: 0,
        }}
      >
        {data.count} {eventText} ({data.percentage}%)
      </p>
    </div>
  );
}

/**
 * Custom legend component for the donut chart.
 * Displays event types with their colors.
 */
interface CustomLegendProps {
  readonly payload?: readonly {
    readonly value: string;
    readonly color: string;
    readonly payload: EventTypeDistribution;
  }[];
}

function CustomLegend({ payload }: CustomLegendProps) {
  if (payload === undefined || payload.length === 0) {
    return null;
  }

  return (
    <ul className="flex flex-wrap justify-center gap-x-4 gap-y-1 mt-2">
      {payload.map((entry) => {
        const label = EVENT_TYPE_LABELS[entry.payload.type];
        return (
          <li key={entry.value} className="flex items-center gap-1.5">
            <span
              className="w-2.5 h-2.5 rounded-full"
              style={{ backgroundColor: entry.color }}
              aria-hidden="true"
            />
            <span
              style={{
                color: COLORS.text.secondary,
                fontSize: '0.75rem',
              }}
            >
              {label}
            </span>
          </li>
        );
      })}
    </ul>
  );
}

/**
 * Center label component displaying total event count.
 */
interface CenterLabelProps {
  readonly totalCount: number;
  readonly cx: number;
  readonly cy: number;
}

function CenterLabel({ totalCount, cx, cy }: CenterLabelProps) {
  const eventText = totalCount === 1 ? 'event' : 'events';

  return (
    <g>
      <text
        x={cx}
        y={cy - 8}
        textAnchor="middle"
        dominantBaseline="middle"
        style={{
          fill: COLORS.text.primary,
          fontSize: '1.5rem',
          fontWeight: 600,
        }}
      >
        {totalCount}
      </text>
      <text
        x={cx}
        y={cy + 14}
        textAnchor="middle"
        dominantBaseline="middle"
        style={{
          fill: COLORS.text.secondary,
          fontSize: '0.75rem',
        }}
      >
        {eventText}
      </text>
    </g>
  );
}

/**
 * Props for the EmptyState component.
 */
interface EmptyStateProps {
  /** WebSocket connection status */
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
        callToAction: 'Event distribution will load once connected',
      };
    case 'reconnecting':
      return {
        primary: 'Reconnecting to server...',
        callToAction: 'Event distribution will resume once reconnected',
      };
    case 'disconnected':
      return {
        primary: 'Not connected',
        callToAction: 'Click Connect to start collecting event data',
      };
    case 'connected':
    default:
      return {
        primary: 'No events to display',
        callToAction:
          'Start a Claude Code session to see event type distribution',
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
          d="M11 3.055A9.001 9.001 0 1020.945 13H11V3.055z"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={1.5}
          d="M20.488 9H15V3.512A9.025 9.025 0 0120.488 9z"
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
 * Event distribution chart displaying breakdown of events by type as a donut chart.
 *
 * Analyzes the events array and calculates the distribution by event type,
 * displaying each type as a segment in the donut chart with distinct colors.
 *
 * Uses the warm color palette with session and activity using orange tones,
 * while tool, agent, summary, and error use contrasting colors for clarity.
 *
 * @example
 * ```tsx
 * // Basic usage
 * <EventDistributionChart events={events} />
 * ```
 */
export function EventDistributionChart({
  events,
  connectionStatus = 'connected',
}: EventDistributionChartProps) {
  // Respect user's reduced motion preference
  const prefersReducedMotion = useReducedMotion();

  // Calculate distribution data
  const distribution = useMemo(() => calculateDistribution(events), [events]);

  // Calculate total event count
  const totalCount = events.length;

  // Check if there are any events
  const hasEvents = totalCount > 0;

  // Animation settings based on reduced motion preference
  const animationDuration = prefersReducedMotion ? 0 : ANIMATION_DURATION_MS;

  return (
    <div
      className="relative w-full h-full min-h-[200px]"
      role="img"
      aria-label={`Event distribution chart showing breakdown of ${totalCount} events by type`}
      style={{ backgroundColor: COLORS.background.secondary }}
    >
      {hasEvents ? (
        <ResponsiveContainer width="100%" height="100%">
          <PieChart accessibilityLayer>
            {/* Gradient definitions for glow effect */}
            <defs>
              {distribution.map((entry) => (
                <filter
                  key={`${GRADIENT_ID_PREFIX}-${entry.type}`}
                  id={`${GRADIENT_ID_PREFIX}-${entry.type}`}
                  x="-20%"
                  y="-20%"
                  width="140%"
                  height="140%"
                >
                  <feGaussianBlur in="SourceGraphic" stdDeviation="2" />
                </filter>
              ))}
            </defs>

            {/* Donut chart */}
            <Pie
              data={distribution as EventTypeDistribution[]}
              cx="50%"
              cy="50%"
              innerRadius="55%"
              outerRadius="80%"
              paddingAngle={2}
              dataKey="count"
              nameKey="type"
              animationDuration={animationDuration}
              animationEasing="ease-out"
              isAnimationActive={!prefersReducedMotion}
              stroke={COLORS.background.secondary}
              strokeWidth={2}
              label={false}
            >
              {distribution.map((entry) => (
                <Cell
                  key={`cell-${entry.type}`}
                  fill={entry.color}
                  style={{
                    filter: `drop-shadow(0 0 4px ${entry.color}40)`,
                    cursor: 'pointer',
                  }}
                />
              ))}
            </Pie>

            {/* Center label with total count */}
            <text>
              <CenterLabel totalCount={totalCount} cx={0} cy={0} />
            </text>

            {/* Tooltip */}
            <Tooltip content={<CustomTooltip />} />

            {/* Legend */}
            <Legend
              content={<CustomLegend />}
              verticalAlign="bottom"
              align="center"
            />
          </PieChart>
        </ResponsiveContainer>
      ) : (
        <EmptyState connectionStatus={connectionStatus} />
      )}

      {/* Center label overlay - positioned absolutely for proper centering */}
      {hasEvents && (
        <div
          className="absolute inset-0 flex items-center justify-center pointer-events-none"
          aria-hidden="true"
        >
          <div className="text-center" style={{ marginBottom: '20px' }}>
            <div
              style={{
                color: COLORS.text.primary,
                fontSize: '1.5rem',
                fontWeight: 600,
                lineHeight: 1,
              }}
            >
              {totalCount}
            </div>
            <div
              style={{
                color: COLORS.text.secondary,
                fontSize: '0.75rem',
                marginTop: '4px',
              }}
            >
              {totalCount === 1 ? 'event' : 'events'}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
