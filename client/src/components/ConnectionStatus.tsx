/**
 * Visual indicator showing WebSocket connection state.
 *
 * Displays a colored dot with optional status text:
 * - Green = connected
 * - Yellow = connecting/reconnecting
 * - Red = disconnected
 */

import { useEventStore } from '../hooks/useEventStore';

import type { ConnectionStatus as ConnectionStatusType } from '../hooks/useEventStore';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Props for the ConnectionStatus component.
 */
interface ConnectionStatusProps {
  /** Whether to show the status text label. Defaults to false. */
  readonly showLabel?: boolean;
  /** Additional CSS classes to apply to the container. */
  readonly className?: string;
}

/**
 * Animation visual style for each connection state.
 *
 * - `pulse`: Glowing pulse effect for connected state
 * - `ring`: Expanding ring animation for connecting/reconnecting states
 * - `warning`: Static warning visual for disconnected state
 * - `none`: No animation (used when animations are disabled)
 */
export type ConnectionAnimationStyle = 'pulse' | 'ring' | 'warning' | 'none';

/**
 * Animation phase within a cycle.
 *
 * - `idle`: Animation is at rest or not active
 * - `animating`: Animation is actively playing
 * - `completing`: Animation is finishing its current cycle
 */
export type ConnectionAnimationPhase = 'idle' | 'animating' | 'completing';

/**
 * Tracks the animation state for the connection status indicator.
 *
 * Used to coordinate visual feedback based on WebSocket connection state:
 * - Connected: Glowing pulse animation indicating healthy connection
 * - Connecting/Reconnecting: Ring animation showing active connection attempt
 * - Disconnected: Warning visual indicating connection loss
 *
 * Supports reduced motion preferences by allowing animations to be paused
 * while maintaining the current visual style for accessibility.
 *
 * @example
 * ```ts
 * const animationState: ConnectionStatusAnimationState = {
 *   style: 'pulse',
 *   phase: 'animating',
 *   isActive: true,
 *   intensity: 1.0,
 *   prefersReducedMotion: false,
 * };
 * ```
 */
export interface ConnectionStatusAnimationState {
  /**
   * The visual animation style based on connection status.
   * Maps to connection state: connected -> pulse, connecting/reconnecting -> ring,
   * disconnected -> warning.
   */
  readonly style: ConnectionAnimationStyle;

  /**
   * Current phase within the animation cycle.
   * Useful for coordinating multi-stage animations or cleanup.
   */
  readonly phase: ConnectionAnimationPhase;

  /**
   * Whether animations are currently active (playing) or paused.
   * When false, the visual style is maintained but animation playback stops.
   */
  readonly isActive: boolean;

  /**
   * Animation intensity from 0.0 to 1.0.
   * Used to scale animation effects (e.g., glow strength, ring expansion rate).
   * A value of 0 effectively disables visual animation effects.
   */
  readonly intensity: number;

  /**
   * Whether the user prefers reduced motion.
   * When true, animations should be subtle or disabled entirely,
   * falling back to static visual indicators.
   */
  readonly prefersReducedMotion: boolean;
}

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/**
 * Status configuration mapping connection state to display properties.
 */
const STATUS_CONFIG: Record<
  ConnectionStatusType,
  { readonly color: string; readonly label: string }
> = {
  connected: {
    color: 'bg-green-500',
    label: 'Connected',
  },
  connecting: {
    color: 'bg-yellow-500',
    label: 'Connecting',
  },
  reconnecting: {
    color: 'bg-yellow-500',
    label: 'Reconnecting',
  },
  disconnected: {
    color: 'bg-red-500',
    label: 'Disconnected',
  },
};

// -----------------------------------------------------------------------------
// Component
// -----------------------------------------------------------------------------

/**
 * Displays the current WebSocket connection status.
 *
 * Uses selective Zustand subscription to only re-render when status changes,
 * preventing unnecessary updates during high-frequency event streams.
 *
 * @example
 * ```tsx
 * // Compact indicator only
 * <ConnectionStatus />
 *
 * // With status text
 * <ConnectionStatus showLabel />
 *
 * // With custom styling
 * <ConnectionStatus className="absolute top-4 right-4" showLabel />
 * ```
 */
export function ConnectionStatus({
  showLabel = false,
  className = '',
}: ConnectionStatusProps) {
  // Selective subscription: only re-render when status changes
  const status = useEventStore((state) => state.status);

  const config = STATUS_CONFIG[status];

  return (
    <div
      className={`inline-flex items-center gap-2 ${className}`}
      role="status"
      aria-label={`Connection status: ${config.label}`}
    >
      <span
        className={`h-2.5 w-2.5 rounded-full ${config.color}`}
        aria-hidden="true"
      />
      {showLabel && (
        <span className="text-sm text-gray-600 dark:text-gray-400">
          {config.label}
        </span>
      )}
    </div>
  );
}
