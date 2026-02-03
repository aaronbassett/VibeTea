/**
 * WebSocket connection hook for VibeTea client.
 *
 * Provides WebSocket connection management with automatic reconnection
 * using exponential backoff. Integrates with useEventStore for event dispatch.
 */

import { useCallback, useEffect, useRef } from 'react';

import type { VibeteaEvent } from '../types/events';
import { useEventStore } from './useEventStore';

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** localStorage key for authentication token */
export const TOKEN_STORAGE_KEY = 'vibetea_token';

/** Initial reconnection delay in milliseconds */
const INITIAL_BACKOFF_MS = 1000;

/** Maximum reconnection delay in milliseconds */
const MAX_BACKOFF_MS = 60000;

/** Jitter factor for reconnection delay (+-25%) */
const JITTER_FACTOR = 0.25;

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * Return type for the useWebSocket hook.
 */
export interface UseWebSocketReturn {
  /** Manually initiate WebSocket connection */
  readonly connect: () => void;
  /** Manually disconnect WebSocket */
  readonly disconnect: () => void;
  /** Whether the WebSocket is currently connected */
  readonly isConnected: boolean;
}

// -----------------------------------------------------------------------------
// Helper Functions
// -----------------------------------------------------------------------------

/**
 * Calculate reconnection delay with exponential backoff and jitter.
 *
 * @param attempt - Current reconnection attempt number (0-indexed)
 * @returns Delay in milliseconds with jitter applied
 */
function calculateBackoff(attempt: number): number {
  // Exponential backoff: initial * 2^attempt, capped at max
  const exponentialDelay = Math.min(
    INITIAL_BACKOFF_MS * Math.pow(2, attempt),
    MAX_BACKOFF_MS
  );

  // Apply jitter: +-25% randomization
  const jitter = 1 + (Math.random() * 2 - 1) * JITTER_FACTOR;

  return Math.round(exponentialDelay * jitter);
}

/**
 * Build WebSocket URL with authentication token.
 *
 * @param baseUrl - Base WebSocket URL
 * @param token - Authentication token
 * @returns Full WebSocket URL with token query parameter
 */
function buildWebSocketUrl(baseUrl: string, token: string): string {
  const url = new URL(baseUrl);
  url.searchParams.set('token', token);
  return url.toString();
}

/**
 * Get default WebSocket URL based on current location.
 *
 * @returns Default WebSocket URL
 */
function getDefaultUrl(): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}/ws`;
}

/**
 * Parse and validate incoming WebSocket message as VibeteaEvent.
 *
 * @param data - Raw message data
 * @returns Parsed event or null if invalid
 */
function parseEventMessage(data: unknown): VibeteaEvent | null {
  if (typeof data !== 'string') {
    return null;
  }

  try {
    const parsed: unknown = JSON.parse(data);

    // Basic structural validation
    if (
      parsed !== null &&
      typeof parsed === 'object' &&
      'id' in parsed &&
      'source' in parsed &&
      'timestamp' in parsed &&
      'type' in parsed &&
      'payload' in parsed
    ) {
      return parsed as VibeteaEvent;
    }

    return null;
  } catch {
    return null;
  }
}

// -----------------------------------------------------------------------------
// Hook Implementation
// -----------------------------------------------------------------------------

/**
 * WebSocket connection hook with auto-reconnect and exponential backoff.
 *
 * Manages WebSocket lifecycle and automatically dispatches received events
 * to the event store. Supports manual connection control and automatic
 * reconnection with exponential backoff (1s initial, 60s max, +-25% jitter).
 *
 * @param url - WebSocket URL (defaults to ws://${window.location.host}/ws)
 * @returns Object with connect, disconnect, and isConnected
 *
 * @example
 * ```tsx
 * function EventMonitor() {
 *   const { connect, disconnect, isConnected } = useWebSocket();
 *
 *   useEffect(() => {
 *     connect();
 *     return () => disconnect();
 *   }, [connect, disconnect]);
 *
 *   return <div>Status: {isConnected ? 'Connected' : 'Disconnected'}</div>;
 * }
 * ```
 */
export function useWebSocket(url?: string): UseWebSocketReturn {
  // Refs for WebSocket instance and reconnection state
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(
    null
  );
  const reconnectAttemptRef = useRef<number>(0);
  const shouldReconnectRef = useRef<boolean>(true);

  // Ref to hold the connect function for recursive reconnection
  const connectRef = useRef<() => void>(() => {});

  // Get store actions and status
  const addEvent = useEventStore((state) => state.addEvent);
  const setStatus = useEventStore((state) => state.setStatus);
  const status = useEventStore((state) => state.status);

  // Resolve WebSocket URL
  const wsUrl = url ?? getDefaultUrl();

  /**
   * Clear any pending reconnection timeout.
   */
  const clearReconnectTimeout = useCallback(() => {
    if (reconnectTimeoutRef.current !== null) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  }, []);

  /**
   * Schedule a reconnection attempt with exponential backoff.
   */
  const scheduleReconnect = useCallback(() => {
    if (!shouldReconnectRef.current) {
      return;
    }

    const delay = calculateBackoff(reconnectAttemptRef.current);
    reconnectAttemptRef.current += 1;

    setStatus('reconnecting');

    reconnectTimeoutRef.current = setTimeout(() => {
      reconnectTimeoutRef.current = null;
      connectRef.current();
    }, delay);
  }, [setStatus]);

  /**
   * Establish WebSocket connection.
   */
  const connect = useCallback(() => {
    // Don't connect if already connected or connecting
    if (
      wsRef.current !== null &&
      (wsRef.current.readyState === WebSocket.OPEN ||
        wsRef.current.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }

    // Get token from localStorage
    const token = localStorage.getItem(TOKEN_STORAGE_KEY);
    if (token === null) {
      console.warn(
        '[useWebSocket] No authentication token found in localStorage'
      );
      setStatus('disconnected');
      return;
    }

    // Clear any pending reconnect
    clearReconnectTimeout();

    // Enable reconnection
    shouldReconnectRef.current = true;

    // Build URL with token
    const fullUrl = buildWebSocketUrl(wsUrl, token);

    setStatus('connecting');

    try {
      const ws = new WebSocket(fullUrl);
      wsRef.current = ws;

      ws.onopen = () => {
        // Reset reconnect attempt counter on successful connection
        reconnectAttemptRef.current = 0;
        setStatus('connected');
      };

      ws.onmessage = (event: MessageEvent<unknown>) => {
        const parsedEvent = parseEventMessage(event.data);
        if (parsedEvent !== null) {
          addEvent(parsedEvent);
        }
      };

      ws.onerror = (event: Event) => {
        console.error('[useWebSocket] Connection error:', event);
      };

      ws.onclose = () => {
        wsRef.current = null;
        setStatus('disconnected');

        // Schedule reconnect if we should
        if (shouldReconnectRef.current) {
          scheduleReconnect();
        }
      };
    } catch (error) {
      console.error('[useWebSocket] Failed to create WebSocket:', error);
      setStatus('disconnected');

      // Schedule reconnect on error
      if (shouldReconnectRef.current) {
        scheduleReconnect();
      }
    }
  }, [wsUrl, addEvent, setStatus, clearReconnectTimeout, scheduleReconnect]);

  // Keep connectRef updated with the latest connect function
  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  /**
   * Disconnect WebSocket and stop reconnection attempts.
   */
  const disconnect = useCallback(() => {
    // Disable auto-reconnect
    shouldReconnectRef.current = false;

    // Clear any pending reconnect
    clearReconnectTimeout();

    // Reset reconnect counter
    reconnectAttemptRef.current = 0;

    // Close WebSocket if open
    if (wsRef.current !== null) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setStatus('disconnected');
  }, [clearReconnectTimeout, setStatus]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      shouldReconnectRef.current = false;
      clearReconnectTimeout();
      if (wsRef.current !== null) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, [clearReconnectTimeout]);

  return {
    connect,
    disconnect,
    isConnected: status === 'connected',
  };
}
