import type { Meta, StoryObj } from '@storybook/react';
import type { ReactNode } from 'react';
import { useEffect } from 'react';

import { EventStream } from './EventStream';
import { useEventStore } from '../hooks/useEventStore';
import type { EventType, VibeteaEvent } from '../types/events';

// -----------------------------------------------------------------------------
// Sample Data Generators
// -----------------------------------------------------------------------------

/**
 * Generate a unique event ID.
 */
function generateEventId(index: number): string {
  return `evt-story-${Date.now()}-${index}`;
}

/**
 * Generate a sample VibeteaEvent with the given parameters.
 *
 * @param index - Index for unique ID generation
 * @param type - Event type
 * @param offsetMs - Milliseconds to subtract from current time (default: 0)
 * @param sessionIndex - Session index for grouping events (default: based on index)
 * @returns A sample VibeteaEvent
 */
function createSampleEvent(
  index: number,
  type: EventType,
  offsetMs: number = 0,
  sessionIndex?: number
): VibeteaEvent {
  const timestamp = new Date(Date.now() - offsetMs).toISOString();
  const sessionId = `session-${sessionIndex ?? Math.floor(index / 5) + 1}`;
  const sources = [
    'claude-agent',
    'vim-plugin',
    'vscode-extension',
    'terminal-client',
  ];
  const source = sources[index % sources.length] ?? 'claude-agent';

  const payloads: Record<EventType, VibeteaEvent['payload']> = {
    session: {
      sessionId,
      action: index % 2 === 0 ? 'started' : 'ended',
      project: 'vibetea',
    },
    activity: {
      sessionId,
      project: 'vibetea',
    },
    tool: {
      sessionId,
      tool: ['Bash', 'Read', 'Write', 'Glob', 'Grep'][index % 5] ?? 'Bash',
      status: index % 2 === 0 ? 'started' : 'completed',
      context: index % 3 === 0 ? 'src/components/EventStream.tsx' : undefined,
      project: 'vibetea',
    },
    agent: {
      sessionId,
      state:
        ['thinking', 'executing', 'waiting', 'idle'][index % 4] ?? 'thinking',
    },
    summary: {
      sessionId,
      summary:
        [
          'Implemented new EventStream component with virtual scrolling support',
          'Fixed performance issue in dashboard rendering pipeline',
          'Refactored authentication flow to use modern patterns',
          'Added comprehensive test coverage for API endpoints',
          'Updated documentation with usage examples and best practices',
        ][index % 5] ?? 'Completed task successfully',
    },
    error: {
      sessionId,
      category:
        ['timeout', 'network', 'validation', 'permission', 'unknown'][
          index % 5
        ] ?? 'unknown',
    },
  };

  return {
    id: generateEventId(index),
    source,
    timestamp,
    type,
    payload: payloads[type],
  } as VibeteaEvent;
}

/**
 * Generate sample events with mixed types.
 *
 * @param count - Number of events to generate
 * @param includeErrors - Whether to include error events
 * @returns Array of sample events
 */
function generateMixedEvents(
  count: number,
  includeErrors: boolean = true
): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  const eventTypes: EventType[] = includeErrors
    ? [
        'activity',
        'tool',
        'tool',
        'activity',
        'agent',
        'session',
        'summary',
        'error',
      ]
    : ['activity', 'tool', 'tool', 'activity', 'agent', 'session', 'summary'];

  for (let i = 0; i < count; i++) {
    // Distribute events across the last 30 minutes
    const offsetMs = Math.random() * 30 * 60 * 1000;
    const eventType = eventTypes[i % eventTypes.length] ?? 'activity';
    events.push(createSampleEvent(i, eventType, offsetMs));
  }

  // Sort by timestamp (newest first) to match store behavior
  return events.sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  );
}

/**
 * Generate high volume events for testing throttling and virtual scrolling.
 *
 * @param count - Number of events to generate
 * @returns Array of sample events
 */
function generateHighVolumeEvents(count: number): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  const eventTypes: EventType[] = ['activity', 'tool', 'agent'];

  for (let i = 0; i < count; i++) {
    // Create events with very recent timestamps to test animation throttling
    // Events are spread across the last 10 seconds
    const offsetMs = (i / count) * 10000;
    const eventType = eventTypes[i % eventTypes.length] ?? 'activity';
    events.push(
      createSampleEvent(i, eventType, offsetMs, Math.floor(i / 20) + 1)
    );
  }

  // Sort by timestamp (newest first)
  return events.sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  );
}

/**
 * Generate events for each event type (one of each).
 *
 * @returns Array with one event of each type
 */
function generateOneOfEachType(): VibeteaEvent[] {
  const types: EventType[] = [
    'session',
    'activity',
    'tool',
    'agent',
    'summary',
    'error',
  ];
  return types.map((type, index) =>
    createSampleEvent(index, type, index * 1000, 1)
  );
}

// -----------------------------------------------------------------------------
// Pre-generated Sample Data
// -----------------------------------------------------------------------------

/** Default sample events (moderate activity, mixed types) */
const DEFAULT_EVENTS = generateMixedEvents(15);

/** Mixed event types demonstrating all categories */
const MIXED_TYPE_EVENTS = generateOneOfEachType();

/** High volume events for testing throttling (200 events) */
const HIGH_VOLUME_EVENTS = generateHighVolumeEvents(200);

/** Very high volume events for stress testing (500 events) */
const STRESS_TEST_EVENTS = generateHighVolumeEvents(500);

// -----------------------------------------------------------------------------
// Store Decorator
// -----------------------------------------------------------------------------

/**
 * Decorator component that populates the event store with sample data.
 * This approach allows stories to work with the actual EventStream component
 * which reads from the Zustand store.
 */
interface EventStoreDecoratorProps {
  readonly events: VibeteaEvent[];
  readonly children: ReactNode;
}

function EventStoreDecorator({ events, children }: EventStoreDecoratorProps) {
  const clearEvents = useEventStore((state) => state.clearEvents);
  const addEvent = useEventStore((state) => state.addEvent);
  const clearFilters = useEventStore((state) => state.clearFilters);

  useEffect(() => {
    // Reset store state
    clearEvents();
    clearFilters();

    // Add events in reverse order (oldest first) since addEvent prepends
    // This ensures the display order matches what we expect
    const reversedEvents = [...events].reverse();
    for (const event of reversedEvents) {
      addEvent(event);
    }

    // Cleanup on unmount
    return () => {
      clearEvents();
      clearFilters();
    };
  }, [events, clearEvents, addEvent, clearFilters]);

  return <>{children}</>;
}

// -----------------------------------------------------------------------------
// Storybook Meta
// -----------------------------------------------------------------------------

/**
 * EventStream displays VibeTea events with efficient virtual scrolling.
 *
 * The component efficiently renders 1000+ events using @tanstack/react-virtual,
 * with automatic scroll-to-bottom behavior for new events and manual jump-to-latest
 * functionality when the user scrolls up.
 *
 * Features:
 * - Virtual scrolling for efficient rendering of large event lists
 * - Auto-scroll to show new events (pauses when user scrolls up 50px+)
 * - Jump to latest button when auto-scroll is disabled
 * - Event type icons with color-coded badges
 * - Entrance animations for new events (respects reduced motion)
 * - Animation throttling (max 10 animations per second)
 * - Fully accessible with ARIA attributes and semantic markup
 */
const meta = {
  title: 'Components/EventStream',
  component: EventStream,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component: `
EventStream displays VibeTea events in a virtual scrolling list with efficient rendering.

**Key Features:**
- Handles 1000+ events efficiently using @tanstack/react-virtual
- Auto-scrolls to show new events (disables when user scrolls up)
- Jump to latest button appears when not at bottom
- Color-coded event type badges with icons
- Entrance animations for new events (throttled to 10/second)
- Respects user's reduced motion preference
- Full keyboard accessibility
        `,
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
        defaultValue: { summary: "''" },
      },
    },
  },
  decorators: [
    (Story, context) => {
      // Get events from story args or use default
      const events =
        (context.args as { _events?: VibeteaEvent[] })._events ??
        DEFAULT_EVENTS;
      return (
        <EventStoreDecorator events={events}>
          <div
            style={{
              width: '700px',
              height: '400px',
              backgroundColor: '#0a0a0a',
              borderRadius: '12px',
              overflow: 'hidden',
            }}
          >
            <Story />
          </div>
        </EventStoreDecorator>
      );
    },
  ],
} satisfies Meta<typeof EventStream>;

export default meta;
type Story = StoryObj<typeof meta>;

// -----------------------------------------------------------------------------
// Stories
// -----------------------------------------------------------------------------

/**
 * Default state with sample events showing typical activity.
 * Displays a mix of event types with moderate activity levels.
 */
export const Default: Story = {
  args: {
    className: 'h-full',
    _events: DEFAULT_EVENTS,
  } as Record<string, unknown>,
};

/**
 * Empty state when no events are present.
 * Displays a helpful message indicating no events have been received yet.
 */
export const EmptyState: Story = {
  args: {
    className: 'h-full',
    _events: [],
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
When no events are present, the component displays an empty state with:

- A document icon indicating the empty list
- Primary text: "No events yet"
- Secondary text explaining events will appear when received

This state is shown for:
- Newly connected clients before any events arrive
- Filtered views that exclude all events
- After clearing the event buffer
        `,
      },
    },
  },
};

/**
 * Mixed event types demonstrating all available event categories.
 * Shows one event of each type: session, activity, tool, agent, summary, and error.
 */
export const MixedEventTypes: Story = {
  args: {
    className: 'h-full',
    _events: MIXED_TYPE_EVENTS,
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
Demonstrates all six event types with their distinctive styling:

| Event Type | Color | Description |
|------------|-------|-------------|
| **session** | Purple | Session lifecycle events (started/ended) |
| **activity** | Green | Activity heartbeat events |
| **tool** | Blue | Tool invocation events (started/completed) |
| **agent** | Amber | Agent state change events |
| **summary** | Cyan | Session summary events |
| **error** | Red | Error events with category |

Each event type has:
- Unique icon
- Color-coded badge background
- Appropriate text color
- Border accent matching the type
        `,
      },
    },
  },
};

/**
 * High volume of events demonstrating virtual scrolling and animation throttling.
 * Contains 200 events to test performance and the animation throttle (max 10/second).
 */
export const HighVolume: Story = {
  args: {
    className: 'h-full',
    _events: HIGH_VOLUME_EVENTS,
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
Tests the component's ability to handle high event volumes efficiently.

**Performance features demonstrated:**

1. **Virtual Scrolling**: Only visible rows are rendered in the DOM, keeping
   performance smooth even with hundreds of events.

2. **Animation Throttling**: New events animate at most 10 times per second.
   This prevents performance degradation during bursts of activity.

3. **Age-based Animation Skip**: Events older than 5 seconds don't animate,
   preventing a flood of animations when loading historical data.

**Scroll behavior:**
- Scroll up to see the "Jump to Latest" button appear
- The button shows the count of new events since scrolling away
- Click the button to instantly scroll to the newest events
        `,
      },
    },
  },
};

/**
 * Stress test with 500 events to verify virtual scrolling performance.
 * Demonstrates that the component maintains smooth scrolling with large datasets.
 */
export const StressTest: Story = {
  args: {
    className: 'h-full',
    _events: STRESS_TEST_EVENTS,
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
Stress test with 500 events to verify virtual scrolling performance.

This story demonstrates that the EventStream component maintains smooth
scrolling and responsive interactions even with a large number of events.

**Technical details:**

- Only ~10-15 rows are rendered at any time (based on viewport height)
- Virtual scrolling calculates positions for all 500 items but only
  materializes DOM nodes for visible items plus a small overscan buffer
- Memory usage remains constant regardless of total event count
- Scroll performance stays smooth at 60fps

**Try:**
- Scrolling quickly through the list
- Jumping to the bottom with the "Jump to Latest" button
- Observing the consistent rendering performance
        `,
      },
    },
  },
};

/**
 * Session events only, showing session lifecycle (started/ended).
 */
export const SessionEventsOnly: Story = {
  args: {
    className: 'h-full',
    _events: Array.from({ length: 10 }, (_, i) =>
      createSampleEvent(i, 'session', i * 2000, i + 1)
    ),
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
Shows only session lifecycle events, demonstrating the session event display:

- **Session Started**: Indicates when a new coding session begins
- **Session Ended**: Indicates when a session concludes

Session events include the project name and are styled with purple accents
to distinguish them from other event types.
        `,
      },
    },
  },
};

/**
 * Tool events only, showing tool invocations (started/completed).
 */
export const ToolEventsOnly: Story = {
  args: {
    className: 'h-full',
    _events: Array.from({ length: 15 }, (_, i) =>
      createSampleEvent(i, 'tool', i * 1000)
    ),
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
Shows only tool invocation events, demonstrating the tool event display:

- **Tool Started**: When a tool begins execution
- **Tool Completed**: When a tool finishes execution

Tools shown include: Bash, Read, Write, Glob, and Grep.
Some events include context (e.g., the file being operated on).

Tool events are styled with blue accents.
        `,
      },
    },
  },
};

/**
 * Error events demonstrating error display with different categories.
 */
export const ErrorEventsOnly: Story = {
  args: {
    className: 'h-full',
    _events: Array.from({ length: 8 }, (_, i) =>
      createSampleEvent(i, 'error', i * 3000)
    ),
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
Shows error events with various categories:

- **timeout**: Operation exceeded time limit
- **network**: Network connectivity issue
- **validation**: Input validation failure
- **permission**: Access denied
- **unknown**: Unclassified error

Error events are styled with red accents to draw attention
to issues that may require user intervention.
        `,
      },
    },
  },
};

/**
 * Demonstrates reduced motion behavior.
 * When users have "prefers-reduced-motion" enabled, entrance animations are disabled.
 */
export const ReducedMotionBehavior: Story = {
  args: {
    className: 'h-full',
    _events: DEFAULT_EVENTS,
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
This story demonstrates accessibility support for reduced motion preferences.

**How reduced motion works:**

The component uses the \`useReducedMotion\` hook which listens to the
\`prefers-reduced-motion\` media query. When enabled:

- Event entrance animations are disabled
- Events appear instantly without slide-in effect
- No visual motion that could trigger vestibular disorders

**Testing reduced motion:**

1. **macOS**: System Preferences > Accessibility > Display > Reduce motion
2. **Windows**: Settings > Ease of Access > Display > Show animations (toggle off)
3. **Chrome DevTools**: Rendering tab > Emulate CSS media feature prefers-reduced-motion

The component respects these settings automatically.
        `,
      },
    },
  },
};

/**
 * Custom styling example with a border and different background.
 */
export const CustomStyling: Story = {
  args: {
    className: 'h-full border-2 border-blue-500/30 rounded-lg',
    _events: DEFAULT_EVENTS,
  } as Record<string, unknown>,
  parameters: {
    docs: {
      description: {
        story: `
The EventStream component accepts a \`className\` prop for custom styling.

This example adds:
- A semi-transparent blue border
- Rounded corners

You can use Tailwind CSS classes or custom CSS to style the container
to match your application's design system.
        `,
      },
    },
  },
};
