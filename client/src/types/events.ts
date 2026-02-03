/**
 * VibeTea Event Types
 *
 * TypeScript types matching the Rust data model for event-driven communication
 * between the client and server.
 */

// -----------------------------------------------------------------------------
// Primitive Union Types
// -----------------------------------------------------------------------------

/**
 * Discriminator for event types in the VibeTea system.
 */
export type EventType =
  | 'session'
  | 'activity'
  | 'tool'
  | 'agent'
  | 'summary'
  | 'error';

/**
 * Actions that can occur during a session lifecycle.
 */
export type SessionAction = 'started' | 'ended';

/**
 * Status of a tool invocation.
 */
export type ToolStatus = 'started' | 'completed';

/**
 * Status of a client session from the UI perspective.
 */
export type SessionStatus = 'active' | 'inactive' | 'ended';

// -----------------------------------------------------------------------------
// Event Payload Interfaces
// -----------------------------------------------------------------------------

/**
 * Payload for session lifecycle events.
 */
export interface SessionPayload {
  readonly sessionId: string;
  readonly action: SessionAction;
  readonly project: string;
}

/**
 * Payload for activity heartbeat events.
 */
export interface ActivityPayload {
  readonly sessionId: string;
  readonly project?: string;
}

/**
 * Payload for tool usage events.
 */
export interface ToolPayload {
  readonly sessionId: string;
  readonly tool: string;
  readonly status: ToolStatus;
  readonly context?: string;
  readonly project?: string;
}

/**
 * Payload for agent state change events.
 */
export interface AgentPayload {
  readonly sessionId: string;
  readonly state: string;
}

/**
 * Payload for summary events.
 */
export interface SummaryPayload {
  readonly sessionId: string;
  readonly summary: string;
}

/**
 * Payload for error events.
 */
export interface ErrorPayload {
  readonly sessionId: string;
  readonly category: string;
}

// -----------------------------------------------------------------------------
// Discriminated Union for Event Payloads
// -----------------------------------------------------------------------------

/**
 * Discriminated union mapping event types to their corresponding payloads.
 * Use this when you need to work with payloads in a type-safe manner
 * based on the event type discriminator.
 */
export type EventPayload =
  | { readonly type: 'session'; readonly payload: SessionPayload }
  | { readonly type: 'activity'; readonly payload: ActivityPayload }
  | { readonly type: 'tool'; readonly payload: ToolPayload }
  | { readonly type: 'agent'; readonly payload: AgentPayload }
  | { readonly type: 'summary'; readonly payload: SummaryPayload }
  | { readonly type: 'error'; readonly payload: ErrorPayload };

// -----------------------------------------------------------------------------
// VibeTea Event Interface
// -----------------------------------------------------------------------------

/**
 * Maps event types to their corresponding payload types for type-safe access.
 */
export interface EventPayloadMap {
  readonly session: SessionPayload;
  readonly activity: ActivityPayload;
  readonly tool: ToolPayload;
  readonly agent: AgentPayload;
  readonly summary: SummaryPayload;
  readonly error: ErrorPayload;
}

/**
 * Generic VibeTea event with type-safe payload inference.
 *
 * @template T - The event type, defaults to EventType for general use
 */
export interface VibeteaEvent<T extends EventType = EventType> {
  /** Unique identifier for this event */
  readonly id: string;
  /** Source identifier (e.g., agent name or client ID) */
  readonly source: string;
  /** RFC 3339 formatted timestamp */
  readonly timestamp: string;
  /** Event type discriminator */
  readonly type: T;
  /** Event payload, typed based on the event type */
  readonly payload: EventPayloadMap[T];
}

// -----------------------------------------------------------------------------
// Client-Side Derived Types
// -----------------------------------------------------------------------------

/**
 * Client-side representation of a session, derived from aggregating events.
 * Used for displaying session state in the UI.
 */
export interface Session {
  /** Unique session identifier */
  readonly sessionId: string;
  /** Source that created this session */
  readonly source: string;
  /** Project associated with this session */
  readonly project: string;
  /** When the session was started */
  readonly startedAt: Date;
  /** When the last event was received for this session */
  readonly lastEventAt: Date;
  /** Current status of the session */
  readonly status: SessionStatus;
  /** Total number of events received for this session */
  readonly eventCount: number;
}

/**
 * Hourly aggregate of events for heatmap visualization.
 * Returned by the query edge function from get_hourly_aggregates().
 */
export interface HourlyAggregate {
  /** Monitor identifier */
  readonly source: string;
  /** Date in YYYY-MM-DD format (UTC) */
  readonly date: string;
  /** Hour of day 0-23 (UTC) */
  readonly hour: number;
  /** Count of events in this hour */
  readonly eventCount: number;
}

// -----------------------------------------------------------------------------
// Type Guards
// -----------------------------------------------------------------------------

/**
 * Type guard to check if an event is a session event.
 */
export function isSessionEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'session'> {
  return event.type === 'session';
}

/**
 * Type guard to check if an event is an activity event.
 */
export function isActivityEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'activity'> {
  return event.type === 'activity';
}

/**
 * Type guard to check if an event is a tool event.
 */
export function isToolEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'tool'> {
  return event.type === 'tool';
}

/**
 * Type guard to check if an event is an agent event.
 */
export function isAgentEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'agent'> {
  return event.type === 'agent';
}

/**
 * Type guard to check if an event is a summary event.
 */
export function isSummaryEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'summary'> {
  return event.type === 'summary';
}

/**
 * Type guard to check if an event is an error event.
 */
export function isErrorEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'error'> {
  return event.type === 'error';
}

/**
 * Valid event type values for runtime validation.
 */
const VALID_EVENT_TYPES = [
  'session',
  'activity',
  'tool',
  'agent',
  'summary',
  'error',
] as const;

/**
 * Type guard to validate that a value is a valid EventType.
 */
export function isValidEventType(value: unknown): value is EventType {
  return (
    typeof value === 'string' &&
    (VALID_EVENT_TYPES as readonly string[]).indexOf(value) !== -1
  );
}
