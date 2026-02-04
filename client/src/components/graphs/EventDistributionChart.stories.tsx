import type { Meta, StoryObj } from '@storybook/react';

import { EventDistributionChart } from './EventDistributionChart';

import type { VibeteaEvent, EventType } from '../../types/events';

// -----------------------------------------------------------------------------
// Helper Functions for Creating Test Data
// -----------------------------------------------------------------------------

/**
 * Creates a mock VibeteaEvent with the specified type.
 */
function createMockEvent(
  type: EventType,
  id: string,
  timestamp?: string
): VibeteaEvent {
  const baseTimestamp = timestamp ?? new Date().toISOString();

  const payloads: Record<EventType, VibeteaEvent['payload']> = {
    session: { sessionId: 'sess-1', action: 'started', project: 'vibetea' },
    activity: { sessionId: 'sess-1', project: 'vibetea' },
    tool: { sessionId: 'sess-1', tool: 'Read', status: 'completed' },
    agent: { sessionId: 'sess-1', state: 'thinking' },
    summary: { sessionId: 'sess-1', summary: 'Task completed' },
    error: { sessionId: 'sess-1', category: 'network' },
  };

  return {
    id,
    source: 'storybook-test',
    timestamp: baseTimestamp,
    type,
    payload: payloads[type],
  } as VibeteaEvent;
}

/**
 * Creates an array of mock events with the specified distribution.
 */
function createMockEvents(
  distribution: Partial<Record<EventType, number>>
): readonly VibeteaEvent[] {
  const events: VibeteaEvent[] = [];
  let idCounter = 0;
  const baseTime = new Date('2024-01-15T12:00:00Z');

  for (const [type, count] of Object.entries(distribution)) {
    for (let i = 0; i < count; i++) {
      const timestamp = new Date(
        baseTime.getTime() + idCounter * 60000
      ).toISOString();
      events.push(createMockEvent(type as EventType, `evt-${idCounter}`, timestamp));
      idCounter++;
    }
  }

  return events;
}

// -----------------------------------------------------------------------------
// Sample Data Sets
// -----------------------------------------------------------------------------

/**
 * Balanced distribution across all event types.
 */
const balancedDistribution = createMockEvents({
  session: 5,
  activity: 8,
  tool: 12,
  agent: 6,
  summary: 4,
  error: 2,
});

/**
 * Skewed distribution with mostly tool events.
 */
const skewedDistribution = createMockEvents({
  session: 1,
  activity: 2,
  tool: 45,
  agent: 3,
  summary: 1,
  error: 0,
});

/**
 * Distribution with many different event types (all types present).
 */
const manyTypesDistribution = createMockEvents({
  session: 15,
  activity: 25,
  tool: 35,
  agent: 20,
  summary: 18,
  error: 12,
});

/**
 * Single event type for edge case testing.
 */
const singleTypeDistribution = createMockEvents({
  tool: 10,
});

/**
 * Two event types only.
 */
const twoTypesDistribution = createMockEvents({
  session: 5,
  error: 3,
});

/**
 * Error-heavy distribution.
 */
const errorHeavyDistribution = createMockEvents({
  session: 2,
  activity: 3,
  tool: 5,
  agent: 1,
  summary: 1,
  error: 25,
});

// -----------------------------------------------------------------------------
// Story Configuration
// -----------------------------------------------------------------------------

/**
 * EventDistributionChart displays a donut chart visualization showing the
 * breakdown of VibeTea events by type. It automatically calculates the
 * distribution from the provided events array and renders each type as
 * a colored segment.
 *
 * Features:
 * - Donut chart with animated transitions
 * - Custom tooltip showing event type, count, and percentage
 * - Legend displaying all present event types
 * - Center label showing total event count
 * - Respects reduced motion preference
 * - Accessible with proper ARIA labels
 */
const meta = {
  title: 'Graphs/EventDistributionChart',
  component: EventDistributionChart,
  parameters: {
    layout: 'centered',
    docs: {
      story: {
        inline: false,
        iframeHeight: 400,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    events: {
      description: 'Array of VibeTea events to analyze and visualize',
      control: false,
    },
  },
  decorators: [
    (Story) => (
      <div
        style={{
          width: '400px',
          height: '350px',
          padding: '16px',
          backgroundColor: '#1a1a1a',
          borderRadius: '8px',
        }}
      >
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof EventDistributionChart>;

export default meta;
type Story = StoryObj<typeof meta>;

// -----------------------------------------------------------------------------
// Stories
// -----------------------------------------------------------------------------

/**
 * Default state with a balanced distribution across all event types.
 * This represents a typical usage scenario with a mix of session, activity,
 * tool, agent, summary, and error events.
 */
export const Default: Story = {
  args: {
    events: balancedDistribution,
  },
};

/**
 * Empty state when no events are available.
 * Displays a placeholder message indicating that event distribution
 * will appear once data is available.
 */
export const Empty: Story = {
  args: {
    events: [],
  },
  parameters: {
    docs: {
      description: {
        story:
          'When no events are provided, the component displays an empty state with a helpful message.',
      },
    },
  },
};

/**
 * Skewed distribution with the majority of events being tool events.
 * Demonstrates how the chart handles an unbalanced data set where
 * one event type dominates.
 */
export const SkewedDistribution: Story = {
  args: {
    events: skewedDistribution,
  },
  parameters: {
    docs: {
      description: {
        story:
          'Shows how the chart renders when one event type (tool) dominates the distribution, with smaller segments for other types.',
      },
    },
  },
};

/**
 * Distribution with many events across all types.
 * Demonstrates the chart with a larger data set where all event types
 * have significant counts, testing the legend and tooltip display.
 */
export const ManyEventTypes: Story = {
  args: {
    events: manyTypesDistribution,
  },
  parameters: {
    docs: {
      description: {
        story:
          'A larger data set with all event types having significant counts. Total: 125 events.',
      },
    },
  },
};

/**
 * Single event type present in the distribution.
 * Edge case where only one type of event exists.
 */
export const SingleType: Story = {
  args: {
    events: singleTypeDistribution,
  },
  parameters: {
    docs: {
      description: {
        story:
          'Edge case showing the chart when only a single event type is present. The donut displays as a complete ring.',
      },
    },
  },
};

/**
 * Two event types only.
 * Shows the chart with minimal variety in event types.
 */
export const TwoTypes: Story = {
  args: {
    events: twoTypesDistribution,
  },
  parameters: {
    docs: {
      description: {
        story:
          'Shows the chart with only two event types (session and error), demonstrating minimal data variety.',
      },
    },
  },
};

/**
 * Error-heavy distribution scenario.
 * Demonstrates how the chart displays when error events dominate,
 * which might indicate system issues that need attention.
 */
export const ErrorHeavy: Story = {
  args: {
    events: errorHeavyDistribution,
  },
  parameters: {
    docs: {
      description: {
        story:
          'A distribution where error events dominate. The red error segment is prominently displayed, which could indicate system issues.',
      },
    },
  },
};

/**
 * Responsive behavior demonstration with a wider container.
 * Shows how the chart adapts to different container sizes.
 */
export const WideContainer: Story = {
  args: {
    events: balancedDistribution,
  },
  decorators: [
    (Story) => (
      <div
        style={{
          width: '600px',
          height: '350px',
          padding: '16px',
          backgroundColor: '#1a1a1a',
          borderRadius: '8px',
        }}
      >
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story:
          'Demonstrates the responsive behavior of the chart in a wider container (600px).',
      },
    },
  },
};

/**
 * Compact container demonstration.
 * Shows how the chart renders in a smaller space.
 */
export const CompactContainer: Story = {
  args: {
    events: balancedDistribution,
  },
  decorators: [
    (Story) => (
      <div
        style={{
          width: '280px',
          height: '280px',
          padding: '12px',
          backgroundColor: '#1a1a1a',
          borderRadius: '8px',
        }}
      >
        <Story />
      </div>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story:
          'Shows the chart in a compact container (280px). The chart scales down while maintaining readability.',
      },
    },
  },
};
