/**
 * Zustand store for managing WebSocket event state.
 *
 * Provides centralized state management for the VibeTea event stream,
 * with selective subscriptions to prevent unnecessary re-renders
 * during high-frequency event updates.
 */

import { create } from 'zustand';

import type { Session, VibeteaEvent } from '../types/events';

// -----------------------------------------------------------------------------
// Types
// -----------------------------------------------------------------------------

/**
 * WebSocket connection status.
 */
export type ConnectionStatus =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'reconnecting';

/**
 * Event store state and actions.
 */
export interface EventStore {
  /** Current WebSocket connection status */
  readonly status: ConnectionStatus;
  /** Event buffer (last 1000 events, newest first) */
  readonly events: readonly VibeteaEvent[];
  /** Active sessions keyed by sessionId */
  readonly sessions: Map<string, Session>;

  /** Add an event to the store (handles FIFO eviction and session updates) */
  readonly addEvent: (event: VibeteaEvent) => void;
  /** Update connection status */
  readonly setStatus: (status: ConnectionStatus) => void;
  /** Clear all events from the buffer */
  readonly clearEvents: () => void;
}

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Maximum number of events to retain in the buffer */
const MAX_EVENTS = 1000;

// -----------------------------------------------------------------------------
// Store Implementation
// -----------------------------------------------------------------------------

/**
 * Zustand store for event management.
 *
 * @example
 * ```tsx
 * // Subscribe to connection status only
 * const status = useEventStore((state) => state.status);
 *
 * // Subscribe to events only
 * const events = useEventStore((state) => state.events);
 *
 * // Get actions (these don't cause re-renders)
 * const addEvent = useEventStore((state) => state.addEvent);
 * ```
 */
export const useEventStore = create<EventStore>()((set) => ({
  // Initial state
  status: 'disconnected',
  events: [],
  sessions: new Map<string, Session>(),

  addEvent: (event: VibeteaEvent) => {
    set((state) => {
      // Add event to beginning (newest first), enforce max limit with FIFO eviction
      const newEvents = [event, ...state.events].slice(0, MAX_EVENTS);

      // Update session state
      const newSessions = new Map(state.sessions);
      const sessionId = event.payload.sessionId;
      const existingSession = newSessions.get(sessionId);
      const now = new Date();
      const eventTimestamp = new Date(event.timestamp);

      if (existingSession === undefined) {
        // Create new session on first event
        const project =
          'project' in event.payload && event.payload.project !== undefined
            ? event.payload.project
            : 'unknown';

        const newSession: Session = {
          sessionId,
          source: event.source,
          project,
          startedAt: eventTimestamp,
          lastEventAt: now,
          status: 'active',
          eventCount: 1,
        };
        newSessions.set(sessionId, newSession);
      } else {
        // Update existing session
        const isSummaryEvent = event.type === 'summary';
        const project =
          'project' in event.payload && event.payload.project !== undefined
            ? event.payload.project
            : existingSession.project;

        const updatedSession: Session = {
          ...existingSession,
          project,
          lastEventAt: now,
          status: isSummaryEvent ? 'ended' : 'active',
          eventCount: existingSession.eventCount + 1,
        };
        newSessions.set(sessionId, updatedSession);
      }

      return {
        events: newEvents,
        sessions: newSessions,
      };
    });
  },

  setStatus: (status: ConnectionStatus) => {
    set({ status });
  },

  clearEvents: () => {
    set({ events: [], sessions: new Map<string, Session>() });
  },
}));

// -----------------------------------------------------------------------------
// Selector Utilities
// -----------------------------------------------------------------------------

/**
 * Get events for a specific session.
 */
export function selectEventsBySession(
  state: EventStore,
  sessionId: string
): readonly VibeteaEvent[] {
  return state.events.filter((event) => event.payload.sessionId === sessionId);
}

/**
 * Get active sessions (status !== 'ended').
 */
export function selectActiveSessions(state: EventStore): Session[] {
  return Array.from(state.sessions.values()).filter(
    (session) => session.status !== 'ended'
  );
}

/**
 * Get session by ID.
 */
export function selectSession(
  state: EventStore,
  sessionId: string
): Session | undefined {
  return state.sessions.get(sessionId);
}
