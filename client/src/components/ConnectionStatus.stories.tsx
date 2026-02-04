import type { Meta, StoryObj } from '@storybook/react';
import { fn } from '@storybook/test';
import type { ReactNode } from 'react';

import { useEventStore } from '../hooks/useEventStore';
import type { ConnectionStatus as ConnectionStatusType } from '../hooks/useEventStore';

import { ConnectionStatus } from './ConnectionStatus';

/**
 * Wrapper component that sets the connection status in the store.
 * This allows stories to demonstrate different connection states.
 */
function ConnectionStatusWrapper({
  status,
  children,
}: {
  status: ConnectionStatusType;
  children: ReactNode;
}) {
  // Set the status in the store on render
  useEventStore.setState({ status });
  return <>{children}</>;
}

/**
 * ConnectionStatus displays a visual indicator of the WebSocket connection state.
 *
 * It shows different animations based on the connection status:
 * - **Connected**: Green dot with a glowing pulse animation
 * - **Connecting**: Yellow dot with an expanding ring animation
 * - **Reconnecting**: Yellow dot with an expanding ring animation (same as connecting)
 * - **Disconnected**: Red dot with a warning flash animation, optionally clickable
 *
 * The component respects the user's reduced motion preferences and provides
 * static indicators when animations are disabled.
 */
const meta: Meta<typeof ConnectionStatus> = {
  title: 'Components/ConnectionStatus',
  component: ConnectionStatus,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component:
          'Visual indicator showing WebSocket connection state. ' +
          'Displays a colored dot with optional status text and animations ' +
          'that respect reduced motion preferences.',
      },
    },
  },
  decorators: [
    (Story) => (
      <div
        style={{
          backgroundColor: '#1a1a2e',
          padding: '2rem',
          borderRadius: '0.5rem',
        }}
      >
        <Story />
      </div>
    ),
  ],
  argTypes: {
    showLabel: {
      description: 'Whether to show the status text label',
      control: 'boolean',
      table: {
        defaultValue: { summary: 'false' },
      },
    },
    className: {
      description: 'Additional CSS classes to apply to the container',
      control: 'text',
    },
    onReconnect: {
      description:
        'Callback invoked when the disconnected indicator is clicked. ' +
        'Use this to trigger a reconnection attempt.',
      action: 'reconnect-clicked',
    },
  },
};

export default meta;
type Story = StoryObj<typeof ConnectionStatus>;

/**
 * Connected state with green pulse animation.
 *
 * The indicator shows a green dot with a gentle glowing pulse effect
 * that conveys a healthy, active connection.
 */
export const Connected: Story = {
  args: {
    showLabel: true,
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="connected">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
};

/**
 * Connecting state with ring animation.
 *
 * The indicator shows a yellow dot with an expanding ring animation
 * that conveys an active connection attempt in progress.
 */
export const Connecting: Story = {
  args: {
    showLabel: true,
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="connecting">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
};

/**
 * Disconnected state with warning visual.
 *
 * The indicator shows a red dot with a slow blinking animation
 * that draws attention to the connection issue without being jarring.
 */
export const Disconnected: Story = {
  args: {
    showLabel: true,
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="disconnected">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
};

/**
 * Reconnecting state with ring animation.
 *
 * Similar to the connecting state, the indicator shows a yellow dot
 * with an expanding ring animation, but the label indicates a reconnection
 * attempt rather than an initial connection.
 */
export const Reconnecting: Story = {
  args: {
    showLabel: true,
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="reconnecting">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
};

/**
 * Disconnected state with reconnect callback.
 *
 * When disconnected and an `onReconnect` callback is provided,
 * the indicator becomes interactive. Users can click or press
 * Enter/Space to trigger a reconnection attempt.
 *
 * Check the Actions panel to see the callback invocation.
 */
export const DisconnectedWithReconnect: Story = {
  args: {
    showLabel: true,
    onReconnect: fn(),
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="disconnected">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story:
          'When disconnected with an `onReconnect` callback, the indicator becomes ' +
          'clickable. Hover over the red dot to see the cursor change, and click ' +
          'to trigger the reconnection callback. Check the Actions panel to see events.',
      },
    },
  },
};

/**
 * Compact indicator without label.
 *
 * The component can be rendered as a minimal dot indicator
 * without the status text label, useful for tight UI spaces.
 */
export const CompactNoLabel: Story = {
  args: {
    showLabel: false,
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="connected">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
};

/**
 * All states comparison.
 *
 * Shows all four connection states side by side for easy comparison
 * of the visual indicators and animations.
 */
export const AllStates: Story = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <span
          style={{
            color: '#888',
            fontSize: '0.75rem',
            width: '100px',
            textTransform: 'uppercase',
          }}
        >
          Connected
        </span>
        <ConnectionStatusWrapper status="connected">
          <ConnectionStatus showLabel />
        </ConnectionStatusWrapper>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <span
          style={{
            color: '#888',
            fontSize: '0.75rem',
            width: '100px',
            textTransform: 'uppercase',
          }}
        >
          Connecting
        </span>
        <ConnectionStatusWrapper status="connecting">
          <ConnectionStatus showLabel />
        </ConnectionStatusWrapper>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <span
          style={{
            color: '#888',
            fontSize: '0.75rem',
            width: '100px',
            textTransform: 'uppercase',
          }}
        >
          Reconnecting
        </span>
        <ConnectionStatusWrapper status="reconnecting">
          <ConnectionStatus showLabel />
        </ConnectionStatusWrapper>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <span
          style={{
            color: '#888',
            fontSize: '0.75rem',
            width: '100px',
            textTransform: 'uppercase',
          }}
        >
          Disconnected
        </span>
        <ConnectionStatusWrapper status="disconnected">
          <ConnectionStatus showLabel onReconnect={fn()} />
        </ConnectionStatusWrapper>
      </div>
    </div>
  ),
  parameters: {
    docs: {
      description: {
        story:
          'Comparison of all four connection states side by side. ' +
          'Notice the different colors and animations for each state.',
      },
    },
  },
};

/**
 * With custom className.
 *
 * Demonstrates applying custom CSS classes to the component
 * for positioning and additional styling.
 */
export const WithCustomClassName: Story = {
  args: {
    showLabel: true,
    className: 'p-2 bg-gray-800 rounded-lg',
  },
  decorators: [
    (Story) => (
      <ConnectionStatusWrapper status="connected">
        <Story />
      </ConnectionStatusWrapper>
    ),
  ],
  parameters: {
    docs: {
      description: {
        story:
          'The `className` prop allows you to apply additional styles ' +
          'for positioning or visual customization.',
      },
    },
  },
};
