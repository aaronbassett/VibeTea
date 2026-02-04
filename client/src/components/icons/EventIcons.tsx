/**
 * SVG icon components for VibeTea event types.
 *
 * Each icon is designed to be simple, recognizable at small sizes (16x16 or 20x20),
 * and uses currentColor for stroke/fill to inherit text color from parent elements.
 */

import type { ComponentPropsWithoutRef, FC } from 'react';

import type { EventType } from '../../types/events';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Common props for all event icon components.
 * Extends standard SVG element props to allow className, style, etc.
 */
type IconProps = ComponentPropsWithoutRef<'svg'>;

/**
 * Function component type for event icons.
 */
type IconComponent = FC<IconProps>;

// -----------------------------------------------------------------------------
// Icon Components
// -----------------------------------------------------------------------------

/**
 * Tool icon - Wrench/hammer representing tool usage events.
 * Simple wrench design optimized for small sizes.
 */
export function ToolIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Wrench head */}
      <path d="M10.5 2.5a3 3 0 0 1 3 3c0 .79-.3 1.5-.8 2l-5.2 5.2a1.5 1.5 0 0 1-2.1 0l-.6-.6a1.5 1.5 0 0 1 0-2.1l5.2-5.2a3 3 0 0 1 .5-2.3" />
      {/* Wrench handle */}
      <path d="M3.5 12.5l2-2" />
    </svg>
  );
}

/**
 * Activity icon - Pulse/heartbeat wave representing activity heartbeat events.
 * Simple pulse line optimized for small sizes.
 */
export function ActivityIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Pulse/heartbeat line */}
      <path d="M1 8h2.5l1.5-3 2 6 2-4 1.5 1h4.5" />
    </svg>
  );
}

/**
 * Session icon - Rocket representing session lifecycle events.
 * Simplified rocket design optimized for small sizes.
 */
export function SessionIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Rocket body */}
      <path d="M8 1.5c3 0 5.5 4 5.5 7a5.5 5.5 0 0 1-11 0c0-3 2.5-7 5.5-7z" />
      {/* Window */}
      <circle cx="8" cy="6.5" r="1.5" />
      {/* Flames */}
      <path d="M6 12.5l-1 2M8 13l0 2M10 12.5l1 2" />
    </svg>
  );
}

/**
 * Summary icon - Clipboard/document representing summary events.
 * Simple clipboard design optimized for small sizes.
 */
export function SummaryIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Clipboard outline */}
      <rect x="2.5" y="3" width="11" height="11.5" rx="1.5" />
      {/* Clipboard clip */}
      <path d="M5.5 3V2a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v1" />
      {/* Text lines */}
      <line x1="5" y1="7" x2="11" y2="7" />
      <line x1="5" y1="9.5" x2="11" y2="9.5" />
      <line x1="5" y1="12" x2="8.5" y2="12" />
    </svg>
  );
}

/**
 * Error icon - Warning triangle representing error events.
 * Classic alert triangle design optimized for small sizes.
 */
export function ErrorIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Warning triangle */}
      <path d="M8 1.5l6.5 12H1.5L8 1.5z" />
      {/* Exclamation mark */}
      <line x1="8" y1="5.5" x2="8" y2="9" />
      <circle cx="8" cy="11" r="0.5" fill="currentColor" stroke="none" />
    </svg>
  );
}

/**
 * Agent icon - Robot/bot representing agent state change events.
 * Simplified robot head design optimized for small sizes.
 */
export function AgentIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Robot head */}
      <rect x="3" y="4" width="10" height="9" rx="2" />
      {/* Antenna */}
      <line x1="8" y1="1" x2="8" y2="4" />
      <circle cx="8" cy="1" r="0.75" fill="currentColor" stroke="none" />
      {/* Eyes */}
      <circle cx="5.5" cy="7.5" r="1" />
      <circle cx="10.5" cy="7.5" r="1" />
      {/* Mouth */}
      <line x1="5.5" y1="10.5" x2="10.5" y2="10.5" />
    </svg>
  );
}

// -----------------------------------------------------------------------------
// Enhanced Tracking Icon Components
// -----------------------------------------------------------------------------

/**
 * Agent Spawn icon - Seedling/sprout representing new agent spawned.
 */
export function AgentSpawnIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Stem */}
      <path d="M8 14V7" />
      {/* Left leaf */}
      <path d="M8 7C6 7 4 5 4 3c2 0 4 2 4 4" />
      {/* Right leaf */}
      <path d="M8 9C10 9 12 7 12 5c-2 0-4 2-4 4" />
    </svg>
  );
}

/**
 * Skill Invocation icon - Sparkle/magic wand representing skill usage.
 */
export function SkillInvocationIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Main sparkle */}
      <path d="M8 1v3M8 12v3M1 8h3M12 8h3" />
      {/* Diagonal sparkles */}
      <path d="M3.5 3.5l2 2M10.5 10.5l2 2M3.5 12.5l2-2M10.5 5.5l2-2" />
    </svg>
  );
}

/**
 * Token Usage icon - Coin representing token consumption.
 */
export function TokenUsageIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Coin circle */}
      <circle cx="8" cy="8" r="6" />
      {/* Dollar sign */}
      <path d="M8 4v8M6 5.5c0-.8.9-1.5 2-1.5s2 .7 2 1.5-.9 1.5-2 1.5-2 .7-2 1.5.9 1.5 2 1.5 2-.7 2-1.5" />
    </svg>
  );
}

/**
 * Session Metrics icon - Bar chart representing metrics.
 */
export function SessionMetricsIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Chart bars */}
      <rect x="2" y="9" width="3" height="5" rx="0.5" />
      <rect x="6.5" y="5" width="3" height="9" rx="0.5" />
      <rect x="11" y="2" width="3" height="12" rx="0.5" />
    </svg>
  );
}

/**
 * Activity Pattern icon - Line chart representing patterns.
 */
export function ActivityPatternIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Trend line */}
      <path d="M2 12l3-4 3 2 4-6 2 2" />
      {/* Baseline */}
      <line x1="2" y1="14" x2="14" y2="14" />
    </svg>
  );
}

/**
 * Model Distribution icon - Pie chart representing model usage distribution.
 */
export function ModelDistributionIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Pie circle */}
      <circle cx="8" cy="8" r="6" />
      {/* Pie slices */}
      <path d="M8 8V2" />
      <path d="M8 8l4.24 4.24" />
      <path d="M8 8l-5.2 3" />
    </svg>
  );
}

/**
 * Todo Progress icon - Checkmark in box representing task progress.
 */
export function TodoProgressIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Box */}
      <rect x="2" y="2" width="12" height="12" rx="2" />
      {/* Checkmark */}
      <path d="M5 8l2 2 4-4" />
    </svg>
  );
}

/**
 * File Change icon - Document with pencil representing file modifications.
 */
export function FileChangeIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Document */}
      <path d="M3 2.5h6.5l3.5 3.5v8.5a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1v-11a1 1 0 0 1 1-1z" />
      {/* Fold corner */}
      <path d="M9.5 2.5v3.5h3.5" />
      {/* Edit lines */}
      <line x1="5" y1="9" x2="10" y2="9" />
      <line x1="5" y1="11.5" x2="8" y2="11.5" />
    </svg>
  );
}

/**
 * Project Activity icon - Folder representing project-level events.
 */
export function ProjectActivityIcon(props: IconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {/* Folder */}
      <path d="M2 4.5a1 1 0 0 1 1-1h3.5l1.5 1.5h5a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1v-8.5z" />
    </svg>
  );
}

// -----------------------------------------------------------------------------
// Icon Mapping
// -----------------------------------------------------------------------------

/**
 * Record mapping EventType to corresponding icon component.
 * Use this to render the appropriate icon for each event type.
 *
 * @example
 * ```tsx
 * import { EVENT_TYPE_ICONS } from './icons/EventIcons';
 * import type { EventType } from '../types/events';
 *
 * function EventBadge({ type }: { type: EventType }) {
 *   const Icon = EVENT_TYPE_ICONS[type];
 *   return <Icon className="w-4 h-4" />;
 * }
 * ```
 */
export const EVENT_TYPE_ICONS: Record<EventType, IconComponent> = {
  tool: ToolIcon,
  activity: ActivityIcon,
  session: SessionIcon,
  summary: SummaryIcon,
  error: ErrorIcon,
  agent: AgentIcon,
  // Enhanced tracking event icons
  agent_spawn: AgentSpawnIcon,
  skill_invocation: SkillInvocationIcon,
  token_usage: TokenUsageIcon,
  session_metrics: SessionMetricsIcon,
  activity_pattern: ActivityPatternIcon,
  model_distribution: ModelDistributionIcon,
  todo_progress: TodoProgressIcon,
  file_change: FileChangeIcon,
  project_activity: ProjectActivityIcon,
};
