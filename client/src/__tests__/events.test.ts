import { describe, it, expect } from 'vitest';
import type { VibeteaEvent, EventType } from '../types/events';

describe('Event Types', () => {
  it('should create a valid session event', () => {
    const event: VibeteaEvent<'session'> = {
      id: 'evt_test123456789012345',
      source: 'test-source',
      timestamp: new Date().toISOString(),
      type: 'session',
      payload: {
        sessionId: '123e4567-e89b-12d3-a456-426614174000',
        action: 'started',
        project: 'test-project',
      },
    };

    expect(event.type).toBe('session');
    expect(event.payload.action).toBe('started');
  });

  it('should create a valid tool event', () => {
    const event: VibeteaEvent<'tool'> = {
      id: 'evt_test123456789012345',
      source: 'test-source',
      timestamp: new Date().toISOString(),
      type: 'tool',
      payload: {
        sessionId: '123e4567-e89b-12d3-a456-426614174000',
        tool: 'Read',
        status: 'completed',
        context: 'file.ts',
        project: 'test-project',
      },
    };

    expect(event.type).toBe('tool');
    expect(event.payload.tool).toBe('Read');
    expect(event.payload.status).toBe('completed');
  });

  it('should support all event types', () => {
    const eventTypes: EventType[] = [
      'session',
      'activity',
      'tool',
      'agent',
      'summary',
      'error',
    ];

    expect(eventTypes).toHaveLength(6);
  });
});
