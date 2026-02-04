/**
 * Skeleton loader components for the VibeTea dashboard.
 *
 * These components display placeholder UI during initial data fetch,
 * matching the layout of actual components with animated pulse effects.
 * Used primarily when the WebSocket is connecting (FR-016).
 *
 * Features:
 * - Layouts match actual component structures
 * - Subtle pulse animation using animate-pulse
 * - Warm dark color palette matching design tokens
 * - Accessible with proper ARIA attributes
 *
 * @module components/skeletons
 */

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Common props for skeleton components.
 */
interface SkeletonProps {
  /** Additional CSS classes to apply to the container */
  readonly className?: string;
}

/**
 * Props for EventStreamSkeleton component.
 */
interface EventStreamSkeletonProps extends SkeletonProps {
  /** Number of event row skeletons to display (default: 6) */
  readonly rowCount?: number;
}

/**
 * Props for SessionOverviewSkeleton component.
 */
interface SessionOverviewSkeletonProps extends SkeletonProps {
  /** Number of session card skeletons to display (default: 3) */
  readonly cardCount?: number;
}

// -----------------------------------------------------------------------------
// Base Skeleton Element
// -----------------------------------------------------------------------------

/**
 * Base skeleton element with pulse animation.
 * A reusable building block for skeleton layouts.
 */
function SkeletonElement({ className = '' }: { readonly className?: string }) {
  return (
    <div
      className={`animate-pulse rounded bg-[#2a2a2a] ${className}`}
      aria-hidden="true"
    />
  );
}

// -----------------------------------------------------------------------------
// Session Card Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for a session card.
 *
 * Matches the layout of a session card with:
 * - Project name placeholder (top-left)
 * - Status badge placeholder (top-right)
 * - Source text placeholder (below header)
 * - Activity indicator dot and duration (bottom-left)
 * - Event count placeholder (bottom-right)
 */
export function SessionCardSkeleton({ className = '' }: SkeletonProps) {
  return (
    <div
      className={`border border-[#2a2a2a] rounded-lg p-4 bg-[#1a1a1a] ${className}`}
      role="status"
      aria-label="Loading session card"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading session...</span>

      {/* Header row: Project name and status badge */}
      <div className="flex items-start justify-between gap-2 mb-2">
        {/* Project name placeholder */}
        <SkeletonElement className="h-4 w-32" />
        {/* Status badge placeholder */}
        <SkeletonElement className="h-5 w-14 rounded-md" />
      </div>

      {/* Source identifier placeholder */}
      <SkeletonElement className="h-3 w-24 mb-3" />

      {/* Footer row: Activity indicator, duration, and event count */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {/* Activity indicator dot */}
          <SkeletonElement className="h-2.5 w-2.5 rounded-full" />
          {/* Duration placeholder */}
          <SkeletonElement className="h-3 w-12" />
        </div>
        {/* Event count placeholder */}
        <SkeletonElement className="h-3 w-16" />
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Session Overview Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for the SessionOverview component.
 *
 * Displays header with title/count placeholder and multiple session card skeletons.
 */
export function SessionOverviewSkeleton({
  className = '',
  cardCount = 3,
}: SessionOverviewSkeletonProps) {
  return (
    <div
      className={`bg-[#131313] text-gray-100 ${className}`}
      role="status"
      aria-label="Loading session overview"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading sessions...</span>

      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        {/* Title placeholder */}
        <SkeletonElement className="h-6 w-20" />
        {/* Count placeholder */}
        <SkeletonElement className="h-4 w-16" />
      </div>

      {/* Session cards */}
      <div className="space-y-3" aria-hidden="true">
        {Array.from({ length: cardCount }, (_, index) => (
          <SessionCardSkeleton key={index} />
        ))}
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Event Row Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for a single event row.
 *
 * Matches the layout of an event row with:
 * - Event type badge with icon placeholder (left)
 * - Event description and source (center)
 * - Timestamp (right)
 */
export function EventRowSkeleton({ className = '' }: SkeletonProps) {
  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 border-b border-[#1a1a1a] ${className}`}
      role="status"
      aria-label="Loading event"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading event...</span>

      {/* Event type badge placeholder */}
      <SkeletonElement className="h-7 w-[90px] rounded-md" />

      {/* Event description area */}
      <div className="flex-1 min-w-0 space-y-1.5">
        {/* Description line */}
        <SkeletonElement className="h-4 w-3/4 max-w-[300px]" />
        {/* Source/session ID line */}
        <SkeletonElement className="h-3 w-1/2 max-w-[180px]" />
      </div>

      {/* Timestamp placeholder */}
      <div className="flex items-center shrink-0 pl-3 ml-2 border-l border-[#2a2a2a]">
        <SkeletonElement className="h-3 w-16" />
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Event Stream Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for the EventStream component.
 *
 * Displays multiple event row skeletons to simulate a loading event list.
 */
export function EventStreamSkeleton({
  className = '',
  rowCount = 6,
}: EventStreamSkeletonProps) {
  return (
    <div
      className={`bg-[#131313] text-gray-100 overflow-hidden ${className}`}
      role="status"
      aria-label="Loading event stream"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading events...</span>

      {/* Event rows */}
      <div aria-hidden="true">
        {Array.from({ length: rowCount }, (_, index) => (
          <EventRowSkeleton key={index} />
        ))}
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Heatmap Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for the Heatmap component.
 *
 * Displays:
 * - Header with title and view toggle placeholders
 * - Grid pattern representing the heatmap cells
 * - Legend placeholder at bottom
 */
export function HeatmapSkeleton({ className = '' }: SkeletonProps) {
  // Create a simplified grid pattern (7 rows x 24 columns for 7-day view)
  const rows = 7;
  const columns = 24;

  return (
    <div
      className={`bg-[#131313] text-gray-100 ${className}`}
      role="status"
      aria-label="Loading activity heatmap"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading activity data...</span>

      {/* Header with title and view toggle */}
      <div className="flex items-center justify-between mb-4">
        {/* Title placeholder */}
        <SkeletonElement className="h-6 w-16" />
        {/* View toggle placeholder (7 Days / 30 Days buttons) */}
        <div className="flex gap-1">
          <SkeletonElement className="h-8 w-16 rounded-md" />
          <SkeletonElement className="h-8 w-20 rounded-md" />
        </div>
      </div>

      {/* Heatmap grid placeholder */}
      <div className="space-y-0.5" aria-hidden="true">
        {/* Hour labels row */}
        <div className="flex gap-0.5 mb-1">
          {/* Empty cell for row labels */}
          <div className="w-8 shrink-0" />
          {/* Hour indicators (simplified - show a few) */}
          {Array.from({ length: columns }, (_, hour) => (
            <div
              key={hour}
              className="flex-1 h-3 flex items-center justify-center"
            >
              {hour % 6 === 0 && <SkeletonElement className="h-2 w-3" />}
            </div>
          ))}
        </div>

        {/* Grid rows */}
        {Array.from({ length: rows }, (_, rowIndex) => (
          <div key={rowIndex} className="flex gap-0.5 items-center">
            {/* Day label placeholder */}
            <div className="w-8 shrink-0 flex justify-end pr-1">
              <SkeletonElement className="h-3 w-6" />
            </div>
            {/* Hour cells */}
            {Array.from({ length: columns }, (_, colIndex) => (
              <div
                key={colIndex}
                className="flex-1 aspect-square animate-pulse rounded-sm bg-[#1a1a2e]"
              />
            ))}
          </div>
        ))}
      </div>

      {/* Legend placeholder */}
      <div className="flex items-center justify-end gap-2 mt-4">
        <SkeletonElement className="h-3 w-8" />
        <div className="flex gap-0.5">
          {Array.from({ length: 5 }, (_, index) => (
            <SkeletonElement key={index} className="w-3 h-3 rounded-sm" />
          ))}
        </div>
        <SkeletonElement className="h-3 w-8" />
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Graph Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for graph components (ActivityGraph, EventDistributionChart).
 *
 * Displays a generic graph-like skeleton with:
 * - Optional title area
 * - Y-axis placeholder
 * - X-axis placeholder
 * - Graph area with subtle wave pattern suggestion
 */
export function GraphSkeleton({ className = '' }: SkeletonProps) {
  return (
    <div
      className={`relative w-full h-full min-h-[200px] bg-[#1a1a1a] ${className}`}
      role="status"
      aria-label="Loading graph"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading chart data...</span>

      {/* Time range toggle placeholder (top-right, like ActivityGraph) */}
      <div className="absolute top-2 right-2 z-10" aria-hidden="true">
        <SkeletonElement className="h-7 w-20 rounded-full" />
      </div>

      {/* Graph area */}
      <div
        className="absolute inset-0 flex flex-col"
        style={{ padding: '40px 20px 20px 40px' }}
        aria-hidden="true"
      >
        {/* Y-axis ticks */}
        <div className="absolute left-2 top-10 bottom-8 flex flex-col justify-between">
          {Array.from({ length: 5 }, (_, index) => (
            <SkeletonElement key={index} className="h-2 w-4" />
          ))}
        </div>

        {/* Main graph area with wave-like shape suggestion */}
        <div className="flex-1 flex items-end justify-around gap-1 pb-6">
          {/* Simulated bar/area chart segments */}
          {Array.from({ length: 12 }, (_, index) => {
            // Create varying heights to suggest a graph shape
            const heights = [30, 45, 60, 50, 70, 85, 65, 55, 75, 60, 40, 35];
            const height = heights[index] ?? 50;
            return (
              <div
                key={index}
                className="flex-1 animate-pulse rounded-t bg-[#2a2a2a]"
                style={{ height: `${height}%` }}
              />
            );
          })}
        </div>

        {/* X-axis ticks */}
        <div className="h-4 flex justify-between px-2">
          {Array.from({ length: 4 }, (_, index) => (
            <SkeletonElement key={index} className="h-2 w-8" />
          ))}
        </div>
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Donut Chart Skeleton
// -----------------------------------------------------------------------------

/**
 * Skeleton loader for donut/pie chart components (EventDistributionChart).
 *
 * Displays a circular skeleton with:
 * - Donut ring placeholder
 * - Center text placeholder
 * - Legend items placeholder
 */
export function DonutChartSkeleton({ className = '' }: SkeletonProps) {
  return (
    <div
      className={`relative w-full h-full min-h-[200px] bg-[#1a1a1a] flex flex-col items-center justify-center ${className}`}
      role="status"
      aria-label="Loading chart"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading event distribution...</span>

      {/* Donut ring placeholder */}
      <div
        className="relative w-32 h-32 rounded-full animate-pulse"
        style={{
          background: `conic-gradient(
            #2a2a2a 0deg 90deg,
            #242424 90deg 180deg,
            #2a2a2a 180deg 270deg,
            #242424 270deg 360deg
          )`,
        }}
        aria-hidden="true"
      >
        {/* Inner circle (donut hole) */}
        <div className="absolute inset-0 m-auto w-[55%] h-[55%] rounded-full bg-[#1a1a1a]">
          {/* Center text placeholders */}
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <SkeletonElement className="h-6 w-8 mb-1" />
            <SkeletonElement className="h-3 w-10" />
          </div>
        </div>
      </div>

      {/* Legend placeholder */}
      <div
        className="flex flex-wrap justify-center gap-x-4 gap-y-1 mt-6"
        aria-hidden="true"
      >
        {Array.from({ length: 4 }, (_, index) => (
          <div key={index} className="flex items-center gap-1.5">
            <SkeletonElement className="w-2.5 h-2.5 rounded-full" />
            <SkeletonElement className="h-3 w-12" />
          </div>
        ))}
      </div>
    </div>
  );
}

// -----------------------------------------------------------------------------
// Dashboard Skeleton (Composite)
// -----------------------------------------------------------------------------

/**
 * Full dashboard skeleton combining all component skeletons.
 * Useful for initial page load states.
 */
export function DashboardSkeleton({ className = '' }: SkeletonProps) {
  return (
    <div
      className={`bg-[#131313] p-4 space-y-4 ${className}`}
      role="status"
      aria-label="Loading dashboard"
    >
      {/* Screen reader text */}
      <span className="sr-only">Loading dashboard...</span>

      {/* Top row: Sessions and Heatmap */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4" aria-hidden="true">
        <SessionOverviewSkeleton className="lg:col-span-1" />
        <HeatmapSkeleton className="lg:col-span-2" />
      </div>

      {/* Middle row: Event Stream */}
      <EventStreamSkeleton className="h-64" />

      {/* Bottom row: Graphs */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4" aria-hidden="true">
        <GraphSkeleton className="h-48" />
        <DonutChartSkeleton className="h-48" />
      </div>
    </div>
  );
}
