/**
 * Zustand store for managing WebSocket event state.
 *
 * Provides centralized state management for the VibeTea event stream,
 * with selective subscriptions to prevent unnecessary re-renders
 * during high-frequency event updates.
 *
 * Session State Machine:
 * - New File Detected -> Active (first event)
 * - Active -> Inactive (no events for 5 minutes)
 * - Inactive -> Active (event received)
 * - Active/Inactive -> Ended (summary event received)
 * - Ended/Inactive -> Removed (30 minutes since last event)
 */

import { create } from 'zustand';

import type { HourlyAggregate, Session, VibeteaEvent } from '../types/events';

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
 * Status of historic data fetch operation.
 */
export type HistoricDataStatus = 'idle' | 'loading' | 'error' | 'success';

/**
 * Time range filter for events.
 */
export interface TimeRangeFilter {
  readonly start: Date;
  readonly end: Date;
}

/**
 * Active filters for the event stream.
 */
export interface EventFilters {
  /** Filter by session ID (null = no filter) */
  readonly sessionId: string | null;
  /** Filter by time range (null = no filter) */
  readonly timeRange: TimeRangeFilter | null;
}

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
  /** Active filters for the event stream */
  readonly filters: EventFilters;

  // Historic data state
  /** Cached historic aggregates for heatmap visualization */
  readonly historicData: readonly HourlyAggregate[];
  /** Status of the historic data fetch operation */
  readonly historicDataStatus: HistoricDataStatus;
  /** Timestamp when historic data was last fetched (null if never fetched) */
  readonly historicDataFetchedAt: number | null;
  /** Error message if historic data fetch failed (null if no error) */
  readonly historicDataError: string | null;

  /** Add an event to the store (handles FIFO eviction and session updates) */
  readonly addEvent: (event: VibeteaEvent) => void;
  /** Update connection status */
  readonly setStatus: (status: ConnectionStatus) => void;
  /** Clear all events from the buffer */
  readonly clearEvents: () => void;
  /** Update session states based on time thresholds (called periodically) */
  readonly updateSessionStates: () => void;
  /** Set session filter (null to clear) */
  readonly setSessionFilter: (sessionId: string | null) => void;
  /** Set time range filter (null to clear) */
  readonly setTimeRangeFilter: (timeRange: TimeRangeFilter | null) => void;
  /** Clear all filters */
  readonly clearFilters: () => void;

  // Historic data actions
  /** Fetch historic aggregate data for the specified number of days */
  readonly fetchHistoricData: (days: 7 | 30) => Promise<void>;
  /** Clear historic data and reset to initial state */
  readonly clearHistoricData: () => void;
}

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** Maximum number of events to retain in the buffer */
const MAX_EVENTS = 1000;

/** Time threshold for transitioning from 'active' to 'inactive' (5 minutes) */
export const INACTIVE_THRESHOLD_MS = 5 * 60 * 1000;

/** Time threshold for removing sessions from display (30 minutes) */
export const REMOVAL_THRESHOLD_MS = 30 * 60 * 1000;

/** Interval for checking session states (30 seconds) */
export const SESSION_CHECK_INTERVAL_MS = 30 * 1000;

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
/** Default filter state (no filters active) */
const DEFAULT_FILTERS: EventFilters = {
  sessionId: null,
  timeRange: null,
};

export const useEventStore = create<EventStore>()((set) => ({
  // Initial state
  status: 'disconnected',
  events: [],
  sessions: new Map<string, Session>(),
  filters: DEFAULT_FILTERS,

  // Historic data initial state
  historicData: [],
  historicDataStatus: 'idle',
  historicDataFetchedAt: null,
  historicDataError: null,

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

        // Determine new status based on event type and current status:
        // - Summary event: always transition to 'ended'
        // - Non-summary event: transition to 'active' (even if currently 'inactive')
        // Note: Once 'ended', sessions stay 'ended' until removed (30 min rule)
        let newStatus = existingSession.status;
        if (isSummaryEvent) {
          newStatus = 'ended';
        } else if (existingSession.status === 'inactive') {
          // Reactivate inactive session on new non-summary event
          newStatus = 'active';
        }

        const updatedSession: Session = {
          ...existingSession,
          project,
          lastEventAt: now,
          status: newStatus,
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

  updateSessionStates: () => {
    set((state) => {
      const now = Date.now();
      const newSessions = new Map<string, Session>();
      let hasChanges = false;

      for (const [sessionId, session] of state.sessions) {
        const timeSinceLastEvent = now - session.lastEventAt.getTime();

        // Remove sessions that have been inactive/ended for 30+ minutes
        if (timeSinceLastEvent >= REMOVAL_THRESHOLD_MS) {
          hasChanges = true;
          continue; // Don't add to newSessions - effectively removes it
        }

        // Transition active sessions to inactive after 5+ minutes without events
        if (
          session.status === 'active' &&
          timeSinceLastEvent >= INACTIVE_THRESHOLD_MS
        ) {
          hasChanges = true;
          newSessions.set(sessionId, {
            ...session,
            status: 'inactive',
          });
          continue;
        }

        // Keep session unchanged
        newSessions.set(sessionId, session);
      }

      // Only update state if there were actual changes
      if (hasChanges) {
        return { sessions: newSessions };
      }

      return state;
    });
  },

  setSessionFilter: (sessionId: string | null) => {
    set((state) => ({
      filters: { ...state.filters, sessionId },
    }));
  },

  setTimeRangeFilter: (timeRange: TimeRangeFilter | null) => {
    set((state) => ({
      filters: { ...state.filters, timeRange },
    }));
  },

  clearFilters: () => {
    set({ filters: DEFAULT_FILTERS });
  },

  // Historic data actions
  fetchHistoricData: async (days: 7 | 30): Promise<void> => {
    // Validate environment configuration
    const supabaseUrl = import.meta.env.VITE_SUPABASE_URL as string | undefined;
    if (supabaseUrl === undefined || supabaseUrl === '') {
      set({
        historicDataStatus: 'error',
        historicDataError: 'Persistence not configured',
      });
      return;
    }

    const token = import.meta.env.VITE_SUPABASE_TOKEN as string | undefined;
    if (token === undefined || token === '') {
      set({
        historicDataStatus: 'error',
        historicDataError: 'Auth token not configured',
      });
      return;
    }

    // Set loading state
    set({ historicDataStatus: 'loading', historicDataError: null });

    try {
      const response = await fetch(
        `${supabaseUrl}/functions/v1/query?days=${days}`,
        {
          method: 'GET',
          headers: {
            Authorization: `Bearer ${token}`,
            'Content-Type': 'application/json',
          },
        }
      );

      if (!response.ok) {
        // Parse error response to extract message
        let errorMessage = `HTTP ${response.status}`;
        try {
          const errorBody = (await response.json()) as {
            error?: string;
            message?: string;
          };
          if (errorBody.message !== undefined) {
            errorMessage = errorBody.message;
          } else if (errorBody.error !== undefined) {
            errorMessage = errorBody.error;
          }
        } catch {
          // If JSON parsing fails, use status text
          errorMessage = response.statusText || `HTTP ${response.status}`;
        }

        set({
          historicDataStatus: 'error',
          historicDataError: errorMessage,
        });
        return;
      }

      const data = (await response.json()) as {
        aggregates: HourlyAggregate[];
        meta: { totalCount: number; daysRequested: number; fetchedAt: string };
      };

      set({
        historicData: data.aggregates,
        historicDataStatus: 'success',
        historicDataFetchedAt: Date.now(),
        historicDataError: null,
      });
    } catch (error) {
      // Network error or other fetch failure
      const errorMessage =
        error instanceof Error ? error.message : 'Failed to fetch historic data';

      set({
        historicDataStatus: 'error',
        historicDataError: errorMessage,
      });
    }
  },

  clearHistoricData: () => {
    set({
      historicData: [],
      historicDataStatus: 'idle',
      historicDataFetchedAt: null,
      historicDataError: null,
    });
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

/**
 * Check if any filters are currently active.
 */
export function hasActiveFilters(state: EventStore): boolean {
  return state.filters.sessionId !== null || state.filters.timeRange !== null;
}

/**
 * Get events filtered by current filter criteria.
 * Returns all events if no filters are active.
 */
export function selectFilteredEvents(
  state: EventStore
): readonly VibeteaEvent[] {
  const { events, filters } = state;

  // Early return if no filters active
  if (filters.sessionId === null && filters.timeRange === null) {
    return events;
  }

  return events.filter((event) => {
    // Session filter
    if (
      filters.sessionId !== null &&
      event.payload.sessionId !== filters.sessionId
    ) {
      return false;
    }

    // Time range filter
    if (filters.timeRange !== null) {
      const eventTime = new Date(event.timestamp);
      if (
        eventTime < filters.timeRange.start ||
        eventTime > filters.timeRange.end
      ) {
        return false;
      }
    }

    return true;
  });
}

/**
 * Historic data state snapshot for components.
 */
export interface HistoricDataSnapshot {
  readonly data: readonly HourlyAggregate[];
  readonly status: HistoricDataStatus;
  readonly fetchedAt: number | null;
  readonly error: string | null;
}

/**
 * Get historic data state as a snapshot object.
 * Useful for components that need all historic data state together.
 */
export function selectHistoricData(state: EventStore): HistoricDataSnapshot {
  return {
    data: state.historicData,
    status: state.historicDataStatus,
    fetchedAt: state.historicDataFetchedAt,
    error: state.historicDataError,
  };
}
