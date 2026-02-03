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
