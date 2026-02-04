import type { Meta, StoryObj } from '@storybook/react';
import { fn } from '@storybook/test';
import { useEffect } from 'react';

import { Heatmap } from './Heatmap';
import { useEventStore } from '../hooks/useEventStore';
import type { VibeteaEvent, EventType } from '../types/events';

// -----------------------------------------------------------------------------
// Sample Data Generators
// -----------------------------------------------------------------------------

/**
 * Generate a unique event ID.
 */
function generateEventId(index: number): string {
  return `heatmap-evt-${Date.now()}-${index}`;
}

/**
 * Creates a mock VibeteaEvent with the specified type and timestamp.
 *
 * @param index - Index for unique ID generation
 * @param timestamp - ISO timestamp string
 * @param type - Event type (defaults to 'activity')
 * @returns A sample VibeteaEvent
 */
function createMockEvent(
  index: number,
  timestamp: string,
  type: EventType = 'activity'
): VibeteaEvent {
  const sessionId = `session-${Math.floor(index / 10) + 1}`;

  const payloads: Record<EventType, VibeteaEvent['payload']> = {
    session: { sessionId, action: 'started' as const, project: 'vibetea' },
    activity: { sessionId, project: 'vibetea' },
    tool: {
      sessionId,
      tool: 'Read',
      status: 'completed' as const,
      project: 'vibetea',
    },
    agent: { sessionId, state: 'thinking' },
    summary: { sessionId, summary: 'Task completed' },
    error: { sessionId, category: 'network' },
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

  return {
    id: generateEventId(index),
    source: 'storybook-test',
    timestamp,
    type,
    payload: payloads[type],
  } as VibeteaEvent;
}

/**
 * Generate sample events distributed across the past N days.
 * Creates events at specific hours to demonstrate the heatmap visualization.
 *
 * @param days - Number of days to generate events for
 * @param eventsPerDay - Average number of events per day
 * @returns Array of sample events
 */
function generateHeatmapEvents(
  days: number,
  eventsPerDay: number
): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  const now = new Date();
  let eventIndex = 0;

  const eventTypes: EventType[] = [
    'activity',
    'activity',
    'tool',
    'activity',
    'tool',
    'agent',
  ];

  for (let dayOffset = 0; dayOffset < days; dayOffset++) {
    // Generate events for this day with some randomization
    const eventCount =
      eventsPerDay + Math.floor((Math.random() - 0.5) * eventsPerDay * 0.5);

    for (let i = 0; i < eventCount; i++) {
      const date = new Date(now);
      date.setDate(date.getDate() - dayOffset);

      // Distribute events across working hours (8am-8pm) with higher concentration
      // during peak hours (10am-12pm, 2pm-5pm)
      const hour = getWeightedHour();
      const minute = Math.floor(Math.random() * 60);

      date.setHours(hour, minute, 0, 0);

      const eventType =
        eventTypes[eventIndex % eventTypes.length] ?? 'activity';
      events.push(createMockEvent(eventIndex, date.toISOString(), eventType));
      eventIndex++;
    }
  }

  return events;
}

/**
 * Get a weighted random hour favoring typical working hours.
 */
function getWeightedHour(): number {
  const weights = [
    { start: 0, end: 7, weight: 0.05 }, // Late night/early morning
    { start: 8, end: 9, weight: 0.15 }, // Morning ramp-up
    { start: 10, end: 12, weight: 0.3 }, // Peak morning
    { start: 13, end: 13, weight: 0.1 }, // Lunch
    { start: 14, end: 17, weight: 0.25 }, // Afternoon peak
    { start: 18, end: 20, weight: 0.1 }, // Evening wind-down
    { start: 21, end: 23, weight: 0.05 }, // Late evening
  ];

  const random = Math.random();
  let cumulative = 0;

  for (const range of weights) {
    cumulative += range.weight;
    if (random <= cumulative) {
      return (
        range.start + Math.floor(Math.random() * (range.end - range.start + 1))
      );
    }
  }

  return 12; // Fallback to noon
}

/**
 * Generate high activity events with many events clustered in specific hours.
 * Creates visible "hot spots" on the heatmap.
 *
 * @param days - Number of days
 * @returns Array of high-activity events
 */
function generateHighActivityEvents(days: number): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  const now = new Date();
  let eventIndex = 0;

  const eventTypes: EventType[] = ['tool', 'activity', 'agent', 'tool'];

  // Create hot spots on specific days and hours
  const hotSpots = [
    { dayOffset: 0, hours: [10, 11, 14, 15] }, // Today
    { dayOffset: 1, hours: [9, 10, 11, 16] }, // Yesterday
    { dayOffset: 2, hours: [14, 15, 16, 17] }, // 2 days ago
    { dayOffset: 3, hours: [10, 11] }, // 3 days ago
    { dayOffset: 5, hours: [13, 14, 15] }, // 5 days ago
  ];

  for (const spot of hotSpots) {
    if (spot.dayOffset >= days) continue;

    for (const hour of spot.hours) {
      // Generate 40-60 events per hot spot hour to hit the bright green tier (51+)
      const count = 40 + Math.floor(Math.random() * 20);

      for (let i = 0; i < count; i++) {
        const date = new Date(now);
        date.setDate(date.getDate() - spot.dayOffset);
        date.setHours(hour, Math.floor(Math.random() * 60), 0, 0);

        const eventType =
          eventTypes[eventIndex % eventTypes.length] ?? 'activity';
        events.push(createMockEvent(eventIndex, date.toISOString(), eventType));
        eventIndex++;
      }
    }
  }

  // Add some moderate activity to other hours
  for (let dayOffset = 0; dayOffset < Math.min(days, 7); dayOffset++) {
    const hoursToFill = [8, 9, 12, 13, 17, 18];
    for (const hour of hoursToFill) {
      const count = 5 + Math.floor(Math.random() * 20); // 5-25 events

      for (let i = 0; i < count; i++) {
        const date = new Date(now);
        date.setDate(date.getDate() - dayOffset);
        date.setHours(hour, Math.floor(Math.random() * 60), 0, 0);

        const eventType =
          eventTypes[eventIndex % eventTypes.length] ?? 'activity';
        events.push(createMockEvent(eventIndex, date.toISOString(), eventType));
        eventIndex++;
      }
    }
  }

  return events;
}

// -----------------------------------------------------------------------------
// Pre-generated Sample Data
// -----------------------------------------------------------------------------

/** Sample events for default 7-day view with moderate activity */
const SAMPLE_EVENTS_7_DAYS = generateHeatmapEvents(7, 30);

/** Sample events for 30-day view */
const SAMPLE_EVENTS_30_DAYS = generateHeatmapEvents(30, 20);

/** High activity sample events with visible hot spots */
const HIGH_ACTIVITY_EVENTS = generateHighActivityEvents(7);

/** Low activity sample (sparse data) */
const LOW_ACTIVITY_EVENTS = generateHeatmapEvents(7, 5);

// -----------------------------------------------------------------------------
// Store Wrapper Component
// -----------------------------------------------------------------------------

/**
 * Wrapper component that populates the event store with mock data.
 * This is necessary because Heatmap reads events from useEventStore.
 */
function HeatmapWithMockStore({
  events,
  className,
  onCellClick,
}: {
  readonly events: readonly VibeteaEvent[];
  readonly className?: string;
  readonly onCellClick?: (startTime: Date, endTime: Date) => void;
}) {
  const clearEvents = useEventStore((state) => state.clearEvents);
  const addEvent = useEventStore((state) => state.addEvent);

  useEffect(() => {
    // Clear existing events and populate with mock data
    clearEvents();

    // Add events in reverse order so newest are first
    const sortedEvents = [...events].sort(
      (a, b) =>
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );

    for (const event of sortedEvents) {
      addEvent(event);
    }

    return () => {
      clearEvents();
    };
  }, [events, clearEvents, addEvent]);

  return <Heatmap className={className} onCellClick={onCellClick} />;
}

// -----------------------------------------------------------------------------
// Storybook Meta
// -----------------------------------------------------------------------------

/**
 * Heatmap displays event activity over time in a grid format.
 *
 * The component visualizes events bucketed by hour over the past 7 or 30 days:
 * - X-axis: Hours of the day (0-23)
 * - Y-axis: Days (most recent at bottom)
 * - Color intensity: Number of events (dark = 0, bright green = 51+)
 *
 * Features:
 * - Toggle between 7-day and 30-day views
 * - Color scale indicating event count per hour
 * - Interactive cells with hover tooltips
 * - Click-to-filter functionality
 * - Glow effects for cells receiving new events
 * - Empty state when no events present
 * - Accessible with ARIA labels and keyboard navigation
 * - Respects user's reduced motion preference
 */
const meta = {
  title: 'Components/Heatmap',
  component: HeatmapWithMockStore,
  parameters: {
    layout: 'centered',
    docs: {
      story: {
        inline: false,
        iframeHeight: 450,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    events: {
      control: false,
      description: 'Array of VibeteaEvent objects to visualize in the heatmap',
      table: {
        type: { summary: 'readonly VibeteaEvent[]' },
      },
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes to apply to the container',
      table: {
        type: { summary: 'string' },
      },
    },
    onCellClick: {
      action: 'onCellClick',
      description:
        'Callback when a cell is clicked, provides start and end times for the hour',
      table: {
        type: { summary: '(startTime: Date, endTime: Date) => void' },
      },
    },
  },
  args: {
    onCellClick: fn(),
  },
  decorators: [
    (Story) => (
      <div
        style={{
          width: '800px',
          padding: '16px',
          backgroundColor: '#1a1a1a',
          borderRadius: '12px',
        }}
      >
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof HeatmapWithMockStore>;

export default meta;
type Story = StoryObj<typeof meta>;

// -----------------------------------------------------------------------------
// Stories
// -----------------------------------------------------------------------------

/**
 * Default state with sample event data over the past 7 days.
 * Shows typical activity patterns with moderate event counts.
 */
export const Default: Story = {
  args: {
    events: SAMPLE_EVENTS_7_DAYS,
  },
};

/**
 * Empty state when no events are available.
 * Displays a helpful message with a calendar icon indicating
 * that activity will appear once events occur.
 */
export const EmptyState: Story = {
  args: {
    events: [],
  },
  parameters: {
    docs: {
      description: {
        story: `
When no events are present, the heatmap displays an empty state with:

- A calendar icon
- "No activity data" message
- Helpful text explaining events will appear as they occur

This gracefully handles:
- New installations with no activity
- Cleared event buffers
- Time periods with no recorded events
        `,
      },
    },
  },
};

/**
 * High activity scenario with bright green "hot spots" on the heatmap.
 * Demonstrates the full color scale from dark (0 events) to bright green (51+).
 * Also shows the glow effect behavior when cells have recent activity.
 */
export const HighActivity: Story = {
  args: {
    events: HIGH_ACTIVITY_EVENTS,
  },
  parameters: {
    docs: {
      description: {
        story: `
High activity scenario demonstrating:

- **Color Scale**: Full range from dark (#1a1a2e) to bright green (#5dad6f)
  - 0 events: #1a1a2e (dark)
  - 1-10 events: #2d4a3e
  - 11-25 events: #3d6b4f
  - 26-50 events: #4d8c5f
  - 51+ events: #5dad6f (bright)

- **Hot Spots**: Concentrated activity in specific hours across multiple days
- **Glow Effects**: Cells animate with orange glow when receiving new events
  - Brightness stacks up to 5 events
  - 2-second decay timer after activity stops
        `,
      },
    },
  },
};

/**
 * Low activity scenario with sparse data across the week.
 * Shows how the heatmap handles minimal activity with mostly
 * dark cells and occasional light activity.
 */
export const LowActivity: Story = {
  args: {
    events: LOW_ACTIVITY_EVENTS,
  },
  parameters: {
    docs: {
      description: {
        story: `
Low activity scenario showing sparse event distribution.

Demonstrates:
- Mostly dark cells with occasional low-count activity
- How the heatmap remains readable even with minimal data
- Color differentiation for small event counts (1-10 range)
        `,
      },
    },
  },
};

/**
 * Data prepared for the 30-day view toggle.
 * Switch to 30 Days using the toggle to see the extended view.
 */
export const ThirtyDayData: Story = {
  args: {
    events: SAMPLE_EVENTS_30_DAYS,
  },
  parameters: {
    docs: {
      description: {
        story: `
Sample data spanning 30 days to demonstrate the extended view.

Click the "30 Days" button to see:
- More rows displaying the full month of activity
- Date labels instead of day names in the row headers
- Patterns emerging over a longer time period
        `,
      },
    },
  },
};

/**
 * Demonstrates the cell click interaction.
 * Click any cell to see the onCellClick callback fire with
 * the start and end times for that hour.
 */
export const WithCellClickHandler: Story = {
  args: {
    events: SAMPLE_EVENTS_7_DAYS,
    onCellClick: fn((startTime: Date, endTime: Date) => {
      console.log('Cell clicked:', { startTime, endTime });
    }),
  },
  parameters: {
    docs: {
      description: {
        story: `
Interactive cells that trigger the \`onCellClick\` callback when clicked.

The callback receives:
- \`startTime\`: Date object for the start of the hour (e.g., 10:00:00)
- \`endTime\`: Date object for the end of the hour (e.g., 11:00:00)

This enables filtering the event stream to show only events
from that specific hour slot.

Check the Actions panel to see the callback values when clicking cells.
        `,
      },
    },
  },
};

/**
 * Custom styling example with additional CSS classes.
 */
export const CustomStyling: Story = {
  args: {
    events: SAMPLE_EVENTS_7_DAYS,
    className: 'p-6 rounded-xl shadow-2xl',
  },
  parameters: {
    docs: {
      description: {
        story: `
The \`className\` prop allows adding custom CSS classes to the container.

This example adds:
- Extra padding (p-6)
- Rounded corners (rounded-xl)
- Enhanced shadow (shadow-2xl)
        `,
      },
    },
  },
};

/**
 * Demonstrates keyboard navigation and accessibility features.
 * Use Tab to navigate between cells and Enter/Space to select.
 */
export const AccessibilityDemo: Story = {
  args: {
    events: SAMPLE_EVENTS_7_DAYS,
  },
  parameters: {
    docs: {
      description: {
        story: `
The Heatmap component is fully accessible:

**Keyboard Navigation:**
- Tab / Shift+Tab to navigate between cells
- Enter or Space to select a cell (triggers onCellClick)
- Focus ring visible on the focused cell

**Screen Reader Support:**
- \`role="region"\` on the container with descriptive label
- \`role="grid"\` on the heatmap with count of days
- \`role="row"\` and \`role="gridcell"\` for proper table semantics
- Each cell has an \`aria-label\` describing its content
  (e.g., "5 events on Jan 15 at 10:00")

**Reduced Motion:**
- Respects \`prefers-reduced-motion\` system setting
- Disables hover scale animations and glow decay transitions
- Tooltip appears/disappears instantly
        `,
      },
    },
  },
};

/**
 * Session count variations showing different activity levels per cell.
 * Demonstrates all color tiers in the heat scale.
 */
export const SessionCountVariations: Story = {
  args: {
    events: (() => {
      const events: VibeteaEvent[] = [];
      const now = new Date();
      let eventIndex = 0;

      // Create specific count tiers to demonstrate the color scale
      const tiers = [
        { hour: 9, count: 1 }, // 1 event - darkest green
        { hour: 10, count: 5 }, // 5 events
        { hour: 11, count: 10 }, // 10 events
        { hour: 12, count: 15 }, // 15 events - medium
        { hour: 13, count: 25 }, // 25 events
        { hour: 14, count: 35 }, // 35 events
        { hour: 15, count: 50 }, // 50 events
        { hour: 16, count: 60 }, // 60 events - brightest green
      ];

      for (const tier of tiers) {
        for (let i = 0; i < tier.count; i++) {
          const date = new Date(now);
          date.setHours(tier.hour, Math.floor(Math.random() * 60), 0, 0);

          events.push(
            createMockEvent(eventIndex, date.toISOString(), 'activity')
          );
          eventIndex++;
        }
      }

      return events;
    })(),
  },
  parameters: {
    docs: {
      description: {
        story: `
Demonstrates all color tiers of the heatmap with specific event counts:

| Hour | Count | Color | Tier |
|------|-------|-------|------|
| 9:00 | 1 | #2d4a3e | 1-10 events |
| 10:00 | 5 | #2d4a3e | 1-10 events |
| 11:00 | 10 | #2d4a3e | 1-10 events |
| 12:00 | 15 | #3d6b4f | 11-25 events |
| 13:00 | 25 | #3d6b4f | 11-25 events |
| 14:00 | 35 | #4d8c5f | 26-50 events |
| 15:00 | 50 | #4d8c5f | 26-50 events |
| 16:00 | 60 | #5dad6f | 51+ events |

Today's row should show a gradient from darker to brighter green.
        `,
      },
    },
  },
};
