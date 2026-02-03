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
  | 'error'
  // New event types for enhanced tracking
  | 'agent_spawn'
  | 'skill_invocation'
  | 'token_usage'
  | 'session_metrics'
  | 'activity_pattern'
  | 'model_distribution'
  | 'todo_progress'
  | 'file_change'
  | 'project_activity';

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
// Enhanced Data Tracking Payload Interfaces
// -----------------------------------------------------------------------------

/**
 * Payload for agent spawn events (Task tool invocations).
 */
export interface AgentSpawnPayload {
  readonly sessionId: string;
  readonly agentType: string;
  readonly description: string;
  readonly timestamp: string;
}

/**
 * Payload for skill/slash command invocation events.
 */
export interface SkillInvocationPayload {
  readonly sessionId: string;
  readonly skillName: string;
  readonly project: string;
  readonly timestamp: string;
}

/**
 * Payload for token usage events (per model).
 */
export interface TokenUsagePayload {
  readonly model: string;
  readonly inputTokens: number;
  readonly outputTokens: number;
  readonly cacheReadTokens: number;
  readonly cacheCreationTokens: number;
}

/**
 * Payload for session metrics events.
 */
export interface SessionMetricsPayload {
  readonly totalSessions: number;
  readonly totalMessages: number;
  readonly totalToolUsage: number;
  readonly longestSession: string;
}

/**
 * Payload for activity pattern events (hourly distribution).
 */
export interface ActivityPatternPayload {
  readonly hourCounts: Record<string, number>;
}

/**
 * Token usage summary for model distribution.
 */
export interface TokenUsageSummary {
  readonly inputTokens: number;
  readonly outputTokens: number;
  readonly cacheReadTokens: number;
  readonly cacheCreationTokens: number;
}

/**
 * Payload for model distribution events.
 */
export interface ModelDistributionPayload {
  readonly modelUsage: Record<string, TokenUsageSummary>;
}

/**
 * Payload for todo progress events.
 */
export interface TodoProgressPayload {
  readonly sessionId: string;
  readonly completed: number;
  readonly inProgress: number;
  readonly pending: number;
  readonly abandoned: boolean;
}

/**
 * Payload for file change events.
 */
export interface FileChangePayload {
  readonly sessionId: string;
  readonly fileHash: string;
  readonly version: number;
  readonly linesAdded: number;
  readonly linesRemoved: number;
  readonly linesModified: number;
  readonly timestamp: string;
}

/**
 * Payload for project activity events.
 */
export interface ProjectActivityPayload {
  readonly projectPath: string;
  readonly sessionId: string;
  readonly isActive: boolean;
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
  | { readonly type: 'error'; readonly payload: ErrorPayload }
  // Enhanced tracking payloads
  | { readonly type: 'agent_spawn'; readonly payload: AgentSpawnPayload }
  | {
      readonly type: 'skill_invocation';
      readonly payload: SkillInvocationPayload;
    }
  | { readonly type: 'token_usage'; readonly payload: TokenUsagePayload }
  | {
      readonly type: 'session_metrics';
      readonly payload: SessionMetricsPayload;
    }
  | {
      readonly type: 'activity_pattern';
      readonly payload: ActivityPatternPayload;
    }
  | {
      readonly type: 'model_distribution';
      readonly payload: ModelDistributionPayload;
    }
  | { readonly type: 'todo_progress'; readonly payload: TodoProgressPayload }
  | { readonly type: 'file_change'; readonly payload: FileChangePayload }
  | {
      readonly type: 'project_activity';
      readonly payload: ProjectActivityPayload;
    };

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
  // Enhanced tracking
  readonly agent_spawn: AgentSpawnPayload;
  readonly skill_invocation: SkillInvocationPayload;
  readonly token_usage: TokenUsagePayload;
  readonly session_metrics: SessionMetricsPayload;
  readonly activity_pattern: ActivityPatternPayload;
  readonly model_distribution: ModelDistributionPayload;
  readonly todo_progress: TodoProgressPayload;
  readonly file_change: FileChangePayload;
  readonly project_activity: ProjectActivityPayload;
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
 * Type guard to check if an event is an agent spawn event.
 */
export function isAgentSpawnEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'agent_spawn'> {
  return event.type === 'agent_spawn';
}

/**
 * Type guard to check if an event is a skill invocation event.
 */
export function isSkillInvocationEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'skill_invocation'> {
  return event.type === 'skill_invocation';
}

/**
 * Type guard to check if an event is a token usage event.
 */
export function isTokenUsageEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'token_usage'> {
  return event.type === 'token_usage';
}

/**
 * Type guard to check if an event is a session metrics event.
 */
export function isSessionMetricsEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'session_metrics'> {
  return event.type === 'session_metrics';
}

/**
 * Type guard to check if an event is an activity pattern event.
 */
export function isActivityPatternEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'activity_pattern'> {
  return event.type === 'activity_pattern';
}

/**
 * Type guard to check if an event is a model distribution event.
 */
export function isModelDistributionEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'model_distribution'> {
  return event.type === 'model_distribution';
}

/**
 * Type guard to check if an event is a todo progress event.
 */
export function isTodoProgressEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'todo_progress'> {
  return event.type === 'todo_progress';
}

/**
 * Type guard to check if an event is a file change event.
 */
export function isFileChangeEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'file_change'> {
  return event.type === 'file_change';
}

/**
 * Type guard to check if an event is a project activity event.
 */
export function isProjectActivityEvent(
  event: VibeteaEvent
): event is VibeteaEvent<'project_activity'> {
  return event.type === 'project_activity';
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
  'agent_spawn',
  'skill_invocation',
  'token_usage',
  'session_metrics',
  'activity_pattern',
  'model_distribution',
  'todo_progress',
  'file_change',
  'project_activity',
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
