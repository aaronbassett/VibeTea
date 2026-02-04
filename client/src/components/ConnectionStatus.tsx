/**
 * Visual indicator showing WebSocket connection state.
 *
 * Displays a colored dot with optional status text:
 * - Green = connected (with glowing pulse animation)
 * - Yellow = connecting/reconnecting
 * - Red = disconnected
 */

import type React from 'react';

import { LazyMotion, domAnimation, m } from 'framer-motion';

import { COLORS } from '../constants/design-tokens';
import { useEventStore } from '../hooks/useEventStore';
import { useReducedMotion } from '../hooks/useReducedMotion';

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
  /**
   * Callback invoked when the disconnected indicator is clicked.
   * Use this to trigger a reconnection attempt.
   */
  readonly onReconnect?: () => void;
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

/**
 * Animation configuration for the connected state glowing pulse.
 *
 * Creates a gentle, calming pulse effect with box-shadow glow
 * that conveys "everything is working" without being jarring.
 */
const CONNECTED_PULSE_ANIMATION = {
  /**
   * Box shadow values for the pulse cycle.
   * Uses design token connected color (#4ade80) with varying opacity/spread.
   */
  boxShadow: [
    `0 0 0 0 ${COLORS.status.connected}00`,
    `0 0 8px 2px ${COLORS.status.connected}60`,
    `0 0 0 0 ${COLORS.status.connected}00`,
  ],
  /**
   * Subtle scale for a breathing effect.
   */
  scale: [1, 1.1, 1],
};

/**
 * Transition configuration for connected pulse animation.
 */
const CONNECTED_PULSE_TRANSITION = {
  duration: 2,
  repeat: Infinity,
  ease: 'easeInOut' as const,
};

/**
 * Animation configuration for the connecting/reconnecting state ring effect.
 *
 * Creates an expanding ring that pulses outward from the indicator dot,
 * conveying "activity in progress" for connection attempts.
 */
const CONNECTING_RING_ANIMATION = {
  /**
   * Scale from dot size to expanded ring.
   * Starts at 1 (same as dot) and expands to 2.5x.
   */
  scale: [1, 2.5],
  /**
   * Opacity fades out as ring expands.
   * Creates the effect of the ring dissipating outward.
   */
  opacity: [0.8, 0],
};

/**
 * Transition configuration for connecting ring animation.
 */
const CONNECTING_RING_TRANSITION = {
  duration: 1.2,
  repeat: Infinity,
  ease: 'easeOut' as const,
};

/**
 * Animation configuration for the disconnected state warning flash.
 *
 * Creates a slow, attention-grabbing blink effect that conveys urgency
 * while remaining non-intrusive. The animation uses opacity and a red
 * glow to stand out distinctly from the calm pulse and active ring animations.
 */
const DISCONNECTED_FLASH_ANIMATION = {
  /**
   * Opacity cycles between full and dimmed to create blink effect.
   * The minimum opacity (0.4) keeps the indicator visible while
   * the full opacity (1.0) draws attention.
   */
  opacity: [1, 0.4, 1],
  /**
   * Red glow that pulses with the blink to enhance warning visual.
   * Uses design token disconnected color (#ef4444).
   */
  boxShadow: [
    `0 0 8px 3px ${COLORS.status.disconnected}80`,
    `0 0 4px 1px ${COLORS.status.disconnected}40`,
    `0 0 8px 3px ${COLORS.status.disconnected}80`,
  ],
};

/**
 * Transition configuration for disconnected flash animation.
 * Slower than other animations (3s cycle) to convey persistent issue
 * without being too jarring or distracting.
 */
const DISCONNECTED_FLASH_TRANSITION = {
  duration: 3,
  repeat: Infinity,
  ease: 'easeInOut' as const,
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
  onReconnect,
}: ConnectionStatusProps) {
  // Respect user's reduced motion preference (FR-008)
  const prefersReducedMotion = useReducedMotion();

  // Selective subscription: only re-render when status changes
  const status = useEventStore((state) => state.status);

  const config = STATUS_CONFIG[status];
  const isConnected = status === 'connected';
  const isConnecting = status === 'connecting' || status === 'reconnecting';
  const isDisconnected = status === 'disconnected';

  /**
   * Handle click on the disconnected indicator.
   * Triggers the reconnection callback if provided.
   */
  const handleReconnectClick = () => {
    if (isDisconnected && onReconnect !== undefined) {
      onReconnect();
    }
  };

  /**
   * Handle keyboard interaction for accessibility.
   * Allows reconnection via Enter or Space key.
   */
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (
      isDisconnected &&
      onReconnect !== undefined &&
      (event.key === 'Enter' || event.key === ' ')
    ) {
      event.preventDefault();
      onReconnect();
    }
  };

  // Determine if the indicator should be interactive (clickable)
  const isInteractive = isDisconnected && onReconnect !== undefined;

  return (
    <LazyMotion features={domAnimation}>
      <div
        className={`inline-flex items-center gap-2 ${className}`}
        role="status"
        aria-label={`Connection status: ${config.label}${isInteractive ? '. Click to reconnect.' : ''}`}
      >
        {isConnected ? (
          prefersReducedMotion ? (
            /* Static green dot for reduced motion users */
            <span
              className="h-2.5 w-2.5 rounded-full"
              aria-hidden="true"
              style={{
                backgroundColor: COLORS.status.connected,
              }}
            />
          ) : (
            /* Animated pulse for standard users */
            <m.span
              className={`h-2.5 w-2.5 rounded-full ${config.color}`}
              aria-hidden="true"
              animate={CONNECTED_PULSE_ANIMATION}
              transition={CONNECTED_PULSE_TRANSITION}
              style={{
                backgroundColor: COLORS.status.connected,
              }}
            />
          )
        ) : isConnecting ? (
          prefersReducedMotion ? (
            /* Static yellow dot for reduced motion users */
            <span
              className="h-2.5 w-2.5 rounded-full"
              aria-hidden="true"
              style={{
                backgroundColor: COLORS.status.connecting,
              }}
            />
          ) : (
            /* Animated ring for standard users */
            <span
              className="relative inline-flex h-2.5 w-2.5"
              aria-hidden="true"
            >
              {/* Animated expanding ring */}
              <m.span
                className="absolute inset-0 rounded-full"
                animate={CONNECTING_RING_ANIMATION}
                transition={CONNECTING_RING_TRANSITION}
                style={{
                  backgroundColor: COLORS.status.connecting,
                }}
              />
              {/* Static indicator dot */}
              <span
                className="relative h-2.5 w-2.5 rounded-full"
                style={{
                  backgroundColor: COLORS.status.connecting,
                }}
              />
            </span>
          )
        ) : prefersReducedMotion ? (
          /* Static red dot for reduced motion users - maintains interactivity */
          <span
            className={`h-2.5 w-2.5 rounded-full ${isInteractive ? 'cursor-pointer hover:brightness-125 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 focus:ring-offset-transparent' : ''}`}
            aria-hidden={!isInteractive}
            role={isInteractive ? 'button' : undefined}
            tabIndex={isInteractive ? 0 : undefined}
            aria-label={isInteractive ? 'Reconnect to server' : undefined}
            onClick={isInteractive ? handleReconnectClick : undefined}
            onKeyDown={isInteractive ? handleKeyDown : undefined}
            style={{
              backgroundColor: COLORS.status.disconnected,
            }}
            title={isInteractive ? 'Click to reconnect' : undefined}
          />
        ) : (
          /* Disconnected state: animated warning flash with optional click-to-reconnect */
          <m.span
            className={`h-2.5 w-2.5 rounded-full ${isInteractive ? 'cursor-pointer hover:scale-125 focus:outline-none focus:ring-2 focus:ring-red-400 focus:ring-offset-2 focus:ring-offset-transparent' : ''}`}
            aria-hidden={!isInteractive}
            role={isInteractive ? 'button' : undefined}
            tabIndex={isInteractive ? 0 : undefined}
            aria-label={isInteractive ? 'Reconnect to server' : undefined}
            onClick={isInteractive ? handleReconnectClick : undefined}
            onKeyDown={isInteractive ? handleKeyDown : undefined}
            animate={DISCONNECTED_FLASH_ANIMATION}
            transition={DISCONNECTED_FLASH_TRANSITION}
            whileHover={isInteractive ? { scale: 1.3 } : undefined}
            whileTap={isInteractive ? { scale: 0.9 } : undefined}
            style={{
              backgroundColor: COLORS.status.disconnected,
            }}
            title={isInteractive ? 'Click to reconnect' : undefined}
          />
        )}
        {showLabel && (
          <span
            className={`text-sm ${isDisconnected ? 'text-red-400' : 'text-gray-600 dark:text-gray-400'} ${isInteractive ? 'cursor-pointer hover:text-red-300' : ''}`}
            onClick={isInteractive ? handleReconnectClick : undefined}
          >
            {config.label}
            {isInteractive && (
              <span className="ml-1 text-xs opacity-70">
                (click to reconnect)
              </span>
            )}
          </span>
        )}
      </div>
    </LazyMotion>
  );
}
