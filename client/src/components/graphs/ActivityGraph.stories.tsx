import type { Meta, StoryObj } from '@storybook/react';
import { fn } from '@storybook/test';

import { ActivityGraph } from './ActivityGraph';
import type { VibeteaEvent } from '../../types/events';
import type { TimeRange } from '../../types/graphs';

// -----------------------------------------------------------------------------
// Sample Data Generators
// -----------------------------------------------------------------------------

/**
 * Generate a unique event ID.
 */
function generateEventId(index: number): string {
  return `evt-${Date.now()}-${index}`;
}

/**
 * Generate a sample VibeteaEvent with the given timestamp offset from now.
 *
 * @param index - Index for unique ID generation
 * @param offsetMs - Milliseconds to subtract from current time
 * @param type - Event type (defaults to 'activity')
 * @returns A sample VibeteaEvent
 */
function createSampleEvent(
  index: number,
  offsetMs: number,
  type:
    | 'session'
    | 'activity'
    | 'tool'
    | 'agent'
    | 'summary'
    | 'error' = 'activity'
): VibeteaEvent {
  const timestamp = new Date(Date.now() - offsetMs).toISOString();
  const sessionId = `session-${Math.floor(index / 5) + 1}`;

  const payloads = {
    session: { sessionId, action: 'started' as const, project: 'vibetea' },
    activity: { sessionId, project: 'vibetea' },
    tool: {
      sessionId,
      tool: 'Bash',
      status: 'completed' as const,
      project: 'vibetea',
    },
    agent: { sessionId, state: 'thinking' },
    summary: { sessionId, summary: 'Completed task successfully' },
    error: { sessionId, category: 'timeout' },
  };

  return {
    id: generateEventId(index),
    source: 'claude-agent',
    timestamp,
    type,
    payload: payloads[type],
  } as VibeteaEvent;
}

/**
 * Generate sample events distributed across a time range.
 *
 * @param count - Number of events to generate
 * @param rangeMs - Time range in milliseconds from now
 * @returns Array of sample events
 */
function generateSampleEvents(count: number, rangeMs: number): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  const eventTypes: Array<
    'session' | 'activity' | 'tool' | 'agent' | 'summary' | 'error'
  > = ['activity', 'activity', 'tool', 'activity', 'tool', 'agent'];

  for (let i = 0; i < count; i++) {
    // Distribute events randomly within the time range
    const offset = Math.random() * rangeMs;
    const eventType = eventTypes[i % eventTypes.length] ?? 'activity';
    events.push(createSampleEvent(i, offset, eventType));
  }

  return events;
}

/**
 * Generate high-activity sample data with bursts of events.
 * Creates clusters of events to simulate peak activity periods.
 *
 * @param rangeMs - Time range in milliseconds
 * @returns Array of events with activity bursts
 */
function generateHighActivityEvents(rangeMs: number): VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  let eventIndex = 0;

  // Create 4-5 bursts of activity
  const burstCount = 4 + Math.floor(Math.random() * 2);
  const burstSize = 15 + Math.floor(Math.random() * 10);

  for (let burst = 0; burst < burstCount; burst++) {
    // Position each burst at a different point in the time range
    const burstCenter = (rangeMs / burstCount) * (burst + 0.5);

    // Create events clustered around the burst center
    for (let i = 0; i < burstSize; i++) {
      const variance = (rangeMs / burstCount) * 0.3;
      const offset = burstCenter + (Math.random() - 0.5) * variance;
      const eventTypes: Array<'tool' | 'activity' | 'agent'> = [
        'tool',
        'activity',
        'agent',
      ];
      const eventType =
        eventTypes[eventIndex % eventTypes.length] ?? 'activity';
      events.push(
        createSampleEvent(eventIndex, Math.max(0, offset), eventType)
      );
      eventIndex++;
    }
  }

  return events;
}

// -----------------------------------------------------------------------------
// Pre-generated Sample Data
// -----------------------------------------------------------------------------

/** Sample events for 1-hour time range (moderate activity) */
const SAMPLE_EVENTS_1H = generateSampleEvents(25, 60 * 60 * 1000);

/** Sample events for 6-hour time range (moderate activity) */
const SAMPLE_EVENTS_6H = generateSampleEvents(40, 6 * 60 * 60 * 1000);

/** Sample events for 24-hour time range (moderate activity) */
const SAMPLE_EVENTS_24H = generateSampleEvents(60, 24 * 60 * 60 * 1000);

/** High activity sample events (bursts of activity) */
const HIGH_ACTIVITY_EVENTS = generateHighActivityEvents(6 * 60 * 60 * 1000);

// -----------------------------------------------------------------------------
// Storybook Meta
// -----------------------------------------------------------------------------

/**
 * ActivityGraph displays event frequency over time as an area chart.
 *
 * The component visualizes event counts in time buckets based on the selected time range:
 * - 1h: 5-minute buckets (12 data points)
 * - 6h: 30-minute buckets (12 data points)
 * - 24h: 2-hour buckets (12 data points)
 *
 * Features:
 * - Area chart with gradient fill using the warm orange accent color
 * - Interactive time range toggle (1h, 6h, 24h)
 * - Custom tooltip showing event count and time
 * - Empty state when no events are present
 * - Respects user's reduced motion preference
 * - Fully accessible with ARIA labels and keyboard navigation
 */
const meta = {
  title: 'Graphs/ActivityGraph',
  component: ActivityGraph,
  parameters: {
    layout: 'centered',
    docs: {
      story: {
        inline: false,
        iframeHeight: 350,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    events: {
      control: false,
      description: 'Array of VibeteaEvent objects to visualize',
      table: {
        type: { summary: 'readonly VibeteaEvent[]' },
      },
    },
    timeRange: {
      control: 'select',
      options: ['1h', '6h', '24h'] satisfies TimeRange[],
      description: 'Currently selected time range for bucketing events',
      table: {
        type: { summary: "'1h' | '6h' | '24h'" },
        defaultValue: { summary: "'1h'" },
      },
    },
    onTimeRangeChange: {
      action: 'onTimeRangeChange',
      description:
        'Callback invoked when the user selects a different time range',
      table: {
        type: { summary: '(range: TimeRange) => void' },
      },
    },
  },
  args: {
    onTimeRangeChange: fn(),
  },
  decorators: [
    (Story) => (
      <div
        style={{
          width: '600px',
          height: '300px',
          backgroundColor: '#1a1a1a',
          borderRadius: '12px',
          padding: '16px',
        }}
      >
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ActivityGraph>;

export default meta;
type Story = StoryObj<typeof meta>;

// -----------------------------------------------------------------------------
// Stories
// -----------------------------------------------------------------------------

/**
 * Default state with sample activity data over a 1-hour time range.
 * Shows typical event distribution with moderate activity levels.
 */
export const Default: Story = {
  args: {
    events: SAMPLE_EVENTS_1H,
    timeRange: '1h',
  },
};

/**
 * Empty state when no events are present in the selected time range.
 * Displays a helpful message indicating no activity has occurred.
 */
export const EmptyState: Story = {
  args: {
    events: [],
    timeRange: '1h',
  },
  parameters: {
    docs: {
      description: {
        story: `
When no events are present within the selected time range, the component displays
an empty state with a chart icon and helpful text. This state gracefully handles:

- Brand new dashboards with no activity yet
- Time ranges with no events (e.g., overnight periods)
- Filtered views that exclude all events
        `,
      },
    },
  },
};

/**
 * Activity graph showing 1-hour time range with 5-minute buckets.
 * Useful for monitoring recent, fine-grained activity patterns.
 */
export const TimeRange1Hour: Story = {
  args: {
    events: SAMPLE_EVENTS_1H,
    timeRange: '1h',
  },
  parameters: {
    docs: {
      description: {
        story: `
The 1-hour time range uses 5-minute buckets (12 data points total).
This view is ideal for:

- Monitoring real-time activity
- Identifying recent spikes or drops
- Fine-grained analysis of current session behavior
        `,
      },
    },
  },
};

/**
 * Activity graph showing 6-hour time range with 30-minute buckets.
 * Balanced view for tracking activity throughout a work session.
 */
export const TimeRange6Hours: Story = {
  args: {
    events: SAMPLE_EVENTS_6H,
    timeRange: '6h',
  },
  parameters: {
    docs: {
      description: {
        story: `
The 6-hour time range uses 30-minute buckets (12 data points total).
This view is ideal for:

- Tracking activity throughout a work session
- Identifying patterns across the morning or afternoon
- Balanced granularity for most use cases
        `,
      },
    },
  },
};

/**
 * Activity graph showing 24-hour time range with 2-hour buckets.
 * Best for understanding daily activity patterns and trends.
 */
export const TimeRange24Hours: Story = {
  args: {
    events: SAMPLE_EVENTS_24H,
    timeRange: '24h',
  },
  parameters: {
    docs: {
      description: {
        story: `
The 24-hour time range uses 2-hour buckets (12 data points total).
This view is ideal for:

- Understanding daily activity patterns
- Identifying peak usage hours
- Comparing activity across different times of day
        `,
      },
    },
  },
};

/**
 * High activity scenario with bursts of events clustered in time.
 * Demonstrates how the chart handles peak activity periods.
 */
export const HighActivity: Story = {
  args: {
    events: HIGH_ACTIVITY_EVENTS,
    timeRange: '6h',
  },
  parameters: {
    docs: {
      description: {
        story: `
High activity scenario showing bursts of events clustered in time.
This demonstrates:

- How the chart scales Y-axis to accommodate high counts
- Visual representation of activity bursts/peaks
- Gradient fill effect at higher activity levels
- Tooltip behavior with larger event counts
        `,
      },
    },
  },
};

/**
 * Read-only mode without the time range change callback.
 * The time range toggle buttons become disabled.
 */
export const ReadOnly: Story = {
  args: {
    events: SAMPLE_EVENTS_6H,
    timeRange: '6h',
    onTimeRangeChange: undefined,
  },
  parameters: {
    docs: {
      description: {
        story: `
When \`onTimeRangeChange\` is not provided, the time range toggle becomes
disabled (read-only mode). This is useful when:

- Displaying a static snapshot of activity
- The time range is controlled by a parent component
- Embedding in contexts where interaction is not desired
        `,
      },
    },
  },
};

/**
 * Demonstrates the reduced motion behavior of the component.
 * Animations are automatically disabled when the user has enabled
 * "prefers-reduced-motion" in their system settings.
 */
export const ReducedMotionBehavior: Story = {
  args: {
    events: SAMPLE_EVENTS_1H,
    timeRange: '1h',
  },
  parameters: {
    docs: {
      description: {
        story: `
This story demonstrates accessibility support for reduced motion preferences.

**How reduced motion works:**

The component uses the \`useReducedMotion\` hook which listens to the
\`prefers-reduced-motion\` media query. When enabled:

- Chart animations are disabled (instant transitions)
- Time range toggle indicator moves without animation
- All visual changes are immediate

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
