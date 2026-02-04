import type { Meta, StoryObj } from '@storybook/react';
import type { ReactNode } from 'react';
import { useEffect } from 'react';
import { fn } from '@storybook/test';

import { SessionOverview } from './SessionOverview';
import { useEventStore } from '../hooks/useEventStore';
import type { Session, VibeteaEvent, EventType } from '../types/events';

// -----------------------------------------------------------------------------
// Sample Data Generators
// -----------------------------------------------------------------------------

/**
 * Create a sample session with the given parameters.
 *
 * @param id - Session identifier number
 * @param status - Session status (active, inactive, ended)
 * @param project - Project name
 * @param minutesAgo - How many minutes ago the session started
 * @param lastActivityMinutesAgo - How many minutes ago the last event occurred
 * @param eventCount - Number of events in this session
 * @returns A sample Session object
 */
function createSampleSession(
  id: number,
  status: 'active' | 'inactive' | 'ended',
  project: string,
  minutesAgo: number,
  lastActivityMinutesAgo: number,
  eventCount: number
): Session {
  const now = Date.now();
  return {
    sessionId: `session-${id}`,
    source: 'claude-agent',
    project,
    startedAt: new Date(now - minutesAgo * 60 * 1000),
    lastEventAt: new Date(now - lastActivityMinutesAgo * 60 * 1000),
    status,
    eventCount,
  };
}

/**
 * Generate a unique event ID.
 */
function generateEventId(sessionId: string, index: number): string {
  return `evt-${sessionId}-${Date.now()}-${index}`;
}

/**
 * Generate sample events for activity indicators.
 *
 * @param sessionId - Session ID to associate events with
 * @param count - Number of events to generate
 * @param withinSeconds - Time window in seconds to distribute events
 * @returns Array of VibeteaEvent objects
 */
function generateRecentEvents(
  sessionId: string,
  count: number,
  withinSeconds: number
): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  const now = Date.now();
  const eventTypes: EventType[] = ['activity', 'tool', 'agent'];

  for (let i = 0; i < count; i++) {
    const offsetMs = Math.random() * withinSeconds * 1000;
    const eventType = eventTypes[i % eventTypes.length] ?? 'activity';

    const timestamp = new Date(now - offsetMs).toISOString();
    const payloads: Record<EventType, VibeteaEvent['payload']> = {
      session: { sessionId, action: 'started', project: 'vibetea' },
      activity: { sessionId, project: 'vibetea' },
      tool: {
        sessionId,
        tool: 'Bash',
        status: 'completed',
        project: 'vibetea',
      },
      agent: { sessionId, state: 'thinking' },
      summary: { sessionId, summary: 'Completed task successfully' },
      error: { sessionId, category: 'timeout' },
      // Enhanced tracking event payloads
      agent_spawn: {
        sessionId,
        agentType: 'Explore',
        description: 'Exploring codebase',
        timestamp,
      },
      skill_invocation: {
        sessionId,
        skillName: 'commit',
        project: 'vibetea',
        timestamp,
      },
      token_usage: {
        model: 'claude-3-sonnet',
        inputTokens: 1000,
        outputTokens: 500,
        cacheReadTokens: 200,
        cacheCreationTokens: 100,
      },
      session_metrics: {
        totalSessions: 10,
        totalMessages: 100,
        totalToolUsage: 50,
        longestSession: '2h 30m',
      },
      activity_pattern: { hourCounts: { '9': 10, '10': 15, '14': 20 } },
      model_distribution: {
        modelUsage: {
          'claude-3-sonnet': {
            inputTokens: 5000,
            outputTokens: 2500,
            cacheReadTokens: 1000,
            cacheCreationTokens: 500,
          },
        },
      },
      todo_progress: {
        sessionId,
        completed: 5,
        inProgress: 2,
        pending: 3,
        abandoned: false,
      },
      file_change: {
        sessionId,
        fileHash: 'abc123',
        version: 1,
        linesAdded: 50,
        linesRemoved: 10,
        linesModified: 20,
        timestamp,
      },
      project_activity: {
        projectPath: '/home/user/projects/vibetea',
        sessionId,
        isActive: true,
      },
    };

    events.push({
      id: generateEventId(sessionId, i),
      source: 'claude-agent',
      timestamp: new Date(now - offsetMs).toISOString(),
      type: eventType,
      payload: payloads[eventType],
    } as VibeteaEvent);
  }

  return events;
}

// -----------------------------------------------------------------------------
// Pre-generated Sample Data
// -----------------------------------------------------------------------------

/** Sample sessions demonstrating different statuses */
const SAMPLE_SESSIONS: Session[] = [
  createSampleSession(1, 'active', 'vibetea-dashboard', 45, 0, 127),
  createSampleSession(2, 'active', 'api-gateway', 120, 2, 89),
  createSampleSession(3, 'inactive', 'data-pipeline', 180, 15, 43),
  createSampleSession(4, 'ended', 'frontend-v2', 360, 120, 256),
];

/** Mixed status sessions for demonstrating status badges */
const MIXED_STATUS_SESSIONS: Session[] = [
  createSampleSession(1, 'active', 'main-project', 30, 0, 42),
  createSampleSession(2, 'inactive', 'sidecar-service', 60, 10, 18),
  createSampleSession(3, 'active', 'cli-tool', 90, 1, 73),
  createSampleSession(4, 'ended', 'legacy-api', 240, 180, 156),
  createSampleSession(5, 'inactive', 'docs-generator', 45, 8, 25),
  createSampleSession(6, 'ended', 'test-runner', 300, 200, 89),
];

/** Generate sample events for activity indicators */
const SAMPLE_EVENTS: VibeteaEvent[] = [
  // High activity for session-1 (20 events in last 60s = 3Hz pulse)
  ...generateRecentEvents('session-1', 20, 60),
  // Medium activity for session-2 (10 events in last 60s = 2Hz pulse)
  ...generateRecentEvents('session-2', 10, 60),
  // Low activity for session-3 (3 events in last 60s = 1Hz pulse)
  ...generateRecentEvents('session-3', 3, 60),
  // No recent activity for session-4 (ended)
];

// -----------------------------------------------------------------------------
// Store Decorator
// -----------------------------------------------------------------------------

/**
 * Decorator component that populates the event store with sample sessions and events.
 * This approach allows stories to work with the actual SessionOverview component
 * which reads from the Zustand store.
 */
interface SessionStoreDecoratorProps {
  readonly sessions: Session[];
  readonly events: VibeteaEvent[];
  readonly children: ReactNode;
}

function SessionStoreDecorator({
  sessions,
  events,
  children,
}: SessionStoreDecoratorProps) {
  const clearEvents = useEventStore((state) => state.clearEvents);
  const addEvent = useEventStore((state) => state.addEvent);
  const clearFilters = useEventStore((state) => state.clearFilters);

  useEffect(() => {
    // Reset store state
    clearEvents();
    clearFilters();

    // Directly set sessions in the store via setState
    const sessionsMap = new Map(sessions.map((s) => [s.sessionId, s]));
    useEventStore.setState({ sessions: sessionsMap });

    // Add events in reverse order (oldest first) since addEvent prepends
    // This ensures the display order matches what we expect
    // Note: We add events without triggering session updates since we set sessions directly
    const reversedEvents = [...events].reverse();
    for (const event of reversedEvents) {
      // Use setState directly to add events without session updates
      useEventStore.setState((state) => ({
        events: [event, ...state.events].slice(0, 1000),
      }));
    }

    // Cleanup on unmount
    return () => {
      clearEvents();
      clearFilters();
    };
  }, [sessions, events, clearEvents, addEvent, clearFilters]);

  return <>{children}</>;
}

// -----------------------------------------------------------------------------
// Storybook Meta
// -----------------------------------------------------------------------------

/**
 * SessionOverview displays AI assistant sessions with real-time activity indicators.
 *
 * The component shows session cards with project information, duration tracking,
 * activity indicators, and status badges. Sessions can be clicked to filter events.
 *
 * Features:
 * - Real-time activity indicators with pulse animation based on event volume
 * - Session status badges (Active, Idle, Ended)
 * - Session duration tracking
 * - Dimmed styling for inactive/ended sessions
 * - Click to filter events by session
 * - Accessible with proper ARIA labels and keyboard navigation
 */
const meta = {
  title: 'Components/SessionOverview',
  component: SessionOverview,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
SessionOverview displays AI assistant sessions with real-time activity indicators.

**Key Features:**
- Real-time activity indicators with variable pulse rates (1Hz, 2Hz, 3Hz)
- Session status badges (Active, Idle, Ended)
- Session duration tracking with formatted display
- Dimmed styling for inactive/ended sessions
- Click handling for session filtering
- Animated card transitions with Framer Motion
- Full keyboard accessibility with ARIA labels
        `,
      },
      story: {
        inline: false,
        iframeHeight: 500,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    className: {
      control: 'text',
      description: 'Additional CSS classes to apply to the container',
      table: {
        type: { summary: 'string' },
      },
    },
    onSessionClick: {
      action: 'onSessionClick',
      description: 'Callback when a session card is clicked',
      table: {
        type: { summary: '(sessionId: string) => void' },
      },
    },
    selectedSessionId: {
      control: 'text',
      description: 'Currently selected session ID for filtering',
      table: {
        type: { summary: 'string | null' },
      },
    },
  },
  args: {
    onSessionClick: fn(),
  },
  decorators: [
    (Story, context) => {
      // Get sessions and events from story args or use defaults
      const storyArgs = context.args as {
        _sessions?: Session[];
        _events?: VibeteaEvent[];
      };
      const sessions = storyArgs._sessions ?? SAMPLE_SESSIONS;
      const events = storyArgs._events ?? SAMPLE_EVENTS;

      return (
        <SessionStoreDecorator sessions={sessions} events={events}>
          <div
            style={{
              width: '320px',
              minHeight: '400px',
              backgroundColor: '#111827',
              borderRadius: '12px',
              padding: '16px',
            }}
          >
            <Story />
          </div>
        </SessionStoreDecorator>
      );
    },
  ],
} satisfies Meta<typeof SessionOverview>;

export default meta;
type Story = StoryObj<typeof meta>;

// -----------------------------------------------------------------------------
// Stories
// -----------------------------------------------------------------------------

/**
 * Default state with sample sessions showing various states and activity levels.
 * Demonstrates the typical appearance with multiple active and inactive sessions.
 */
export const Default: Story = {
  args: {},
  parameters: {
    docs: {
      description: {
        story: `
The default state shows a mix of sessions with different statuses and activity levels:

- **Active sessions**: Green indicator, may have pulse animation based on recent activity
- **Idle sessions**: Yellow status badge, shows "Last active: X ago"
- **Ended sessions**: Gray status badge, dimmed appearance

The activity indicator pulses at different rates based on event frequency:
- 1-5 events/min: 1Hz pulse (slow)
- 6-15 events/min: 2Hz pulse (medium)
- 16+ events/min: 3Hz pulse (fast)
        `,
      },
    },
  },
};

/**
 * Empty state when no sessions are available.
 * Displays a helpful message indicating no active sessions.
 */
export const EmptyState: Story = {
  args: {
    _sessions: [],
    _events: [],
  } as unknown as typeof Default.args,
  parameters: {
    docs: {
      description: {
        story: `
When no sessions are present, the component displays an empty state with:

- A computer monitor icon
- "No active sessions" message
- Helpful text: "Sessions will appear here when detected"

This state handles:
- Initial dashboard load before any sessions are created
- After all sessions have ended and been cleared
- When the server connection is first established
        `,
      },
    },
  },
};

/**
 * Mixed session statuses demonstrating all three states.
 * Shows Active, Idle, and Ended sessions together with their visual differences.
 */
export const MixedStatuses: Story = {
  args: {
    _sessions: MIXED_STATUS_SESSIONS,
    _events: SAMPLE_EVENTS,
  } as unknown as typeof Default.args,
  parameters: {
    docs: {
      description: {
        story: `
This story demonstrates all three session statuses side by side:

**Active** (green badge):
- Full opacity
- Green activity indicator
- Pulse animation based on event frequency
- Shows event count

**Idle/Inactive** (yellow badge):
- Slightly dimmed (70% opacity)
- Gray activity indicator
- Shows "Last active: X ago"

**Ended** (gray badge):
- Dimmed appearance (70% opacity)
- Gray activity indicator
- No pulse animation
- No "Last active" text

Sessions are automatically sorted: Active first, then Idle, then Ended.
Within each group, sessions are sorted by most recent activity.
        `,
      },
    },
  },
};

/**
 * Hover and focus effects demonstration.
 * Shows interactive states when hovering or focusing on session cards.
 */
export const HoverAndFocusEffects: Story = {
  args: {
    onSessionClick: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: `
When \`onSessionClick\` is provided, session cards become interactive with:

**Hover effects** (mouse interaction):
- Slight scale increase (1.02x)
- Warm orange glow shadow
- Background color change
- Smooth spring animation

**Focus effects** (keyboard navigation):
- Blue focus ring (2px inset)
- Full keyboard accessibility (Tab to navigate, Enter/Space to select)

**Active session glow**:
- Active sessions have a subtle animated glow effect
- Uses the warm orange accent color (#D97757)

**Reduced motion support**:
- All animations respect \`prefers-reduced-motion\`
- When enabled, transitions are instant
        `,
      },
    },
  },
};

/**
 * Selected session state showing the filter highlight.
 * Demonstrates how a selected session appears when filtering events.
 */
export const SelectedSession: Story = {
  args: {
    selectedSessionId: 'session-1',
    onSessionClick: fn(),
  },
  parameters: {
    docs: {
      description: {
        story: `
When a session is selected for filtering:

- **Selected session**: Purple border and background tint
- **Non-selected sessions**: Remain at 70% opacity (dimmed)
- **ARIA**: \`aria-selected="true"\` on the selected card

The selected state is controlled by the \`selectedSessionId\` prop.
Click a session to toggle selection (handled by parent component).
        `,
      },
    },
  },
};

/**
 * Read-only mode without click handler.
 * Sessions are displayed but not interactive.
 */
export const ReadOnly: Story = {
  args: {
    onSessionClick: undefined,
  },
  parameters: {
    docs: {
      description: {
        story: `
When \`onSessionClick\` is not provided:

- Cards are not focusable (no \`tabIndex\`)
- No hover effects or cursor changes
- No click handlers
- Still displays all session information

This mode is useful for:
- Display-only contexts
- When filtering is handled elsewhere
- Embedded views where interaction is not desired
        `,
      },
    },
  },
};

/**
 * High activity scenario with many recent events.
 * Shows fast-pulsing activity indicators on active sessions.
 */
export const HighActivity: Story = {
  args: {
    _sessions: [
      createSampleSession(1, 'active', 'burst-project', 10, 0, 342),
      createSampleSession(2, 'active', 'steady-worker', 30, 0, 156),
    ],
    _events: [
      // Very high activity (25 events = 3Hz fast pulse)
      ...generateRecentEvents('session-1', 25, 60),
      // High activity (18 events = 3Hz fast pulse)
      ...generateRecentEvents('session-2', 18, 60),
    ],
  } as unknown as typeof Default.args,
  parameters: {
    docs: {
      description: {
        story: `
High activity scenario with many recent events per session:

- 16+ events in the last 60 seconds triggers the 3Hz fast pulse
- Both sessions show rapid activity indication
- High event counts are displayed in the footer

This simulates peak usage during intensive coding sessions
where multiple tools are being invoked frequently.
        `,
      },
    },
  },
};

/**
 * Single active session with no other sessions.
 * Clean display for focused work on a single project.
 */
export const SingleSession: Story = {
  args: {
    _sessions: [createSampleSession(1, 'active', 'focused-project', 60, 0, 87)],
    _events: generateRecentEvents('session-1', 8, 60),
  } as unknown as typeof Default.args,
  parameters: {
    docs: {
      description: {
        story: `
A single active session scenario:

- Shows "1 session" in the header (singular form)
- Clean, uncluttered display
- Medium activity pulse (8 events/min)

This represents focused work on a single project without
multiple AI assistants running simultaneously.
        `,
      },
    },
  },
};

/**
 * Custom styling demonstration with className prop.
 * Shows how additional CSS classes affect the component appearance.
 */
export const CustomStyling: Story = {
  args: {
    className: 'p-6 rounded-xl border border-gray-700',
  },
  parameters: {
    docs: {
      description: {
        story: `
The \`className\` prop allows customization of the container:

- Additional padding, borders, or rounded corners
- Override background colors
- Integrate with parent layout systems

In this example, extra padding and a border are added
to demonstrate the composability of the component.
        `,
      },
    },
  },
};
