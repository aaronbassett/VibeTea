import type { Meta, StoryObj } from '@storybook/react';
import { fn } from '@storybook/test';
import React, { useState } from 'react';

import { AnimationErrorBoundary } from './ErrorBoundary';

/**
 * Helper component that throws an error when shouldThrow is true.
 * Used to demonstrate the error boundary's fallback behavior.
 */
function ErrorThrowingComponent({ shouldThrow }: { shouldThrow: boolean }) {
  if (shouldThrow) {
    throw new Error('Simulated animation error for Storybook demonstration');
  }
  return (
    <div
      style={{
        padding: '1rem',
        backgroundColor: '#1a1a2e',
        borderRadius: '0.5rem',
        color: '#e0e0e0',
      }}
    >
      Animated content renders successfully
    </div>
  );
}

/**
 * Interactive wrapper that allows toggling the error state.
 * This component provides a button to trigger an error and demonstrate
 * the error boundary catching it.
 */
function InteractiveErrorDemo({
  fallback,
  onError,
}: {
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}) {
  const [shouldThrow, setShouldThrow] = useState(false);
  const [key, setKey] = useState(0);

  const handleTriggerError = () => {
    setShouldThrow(true);
  };

  const handleReset = () => {
    setShouldThrow(false);
    // Increment key to force remount of error boundary
    setKey((prev) => prev + 1);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <button
          onClick={handleTriggerError}
          disabled={shouldThrow}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: shouldThrow ? '#555' : '#dc3545',
            color: 'white',
            border: 'none',
            borderRadius: '0.25rem',
            cursor: shouldThrow ? 'not-allowed' : 'pointer',
          }}
        >
          Trigger Error
        </button>
        <button
          onClick={handleReset}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: '#28a745',
            color: 'white',
            border: 'none',
            borderRadius: '0.25rem',
            cursor: 'pointer',
          }}
        >
          Reset
        </button>
      </div>
      <AnimationErrorBoundary key={key} fallback={fallback} onError={onError}>
        <ErrorThrowingComponent shouldThrow={shouldThrow} />
      </AnimationErrorBoundary>
    </div>
  );
}

const meta = {
  title: 'Animated/ErrorBoundary',
  component: AnimationErrorBoundary,
  parameters: {
    layout: 'centered',
    docs: {
      description: {
        component:
          'Error boundary component for catching and handling errors in animated components. ' +
          'When an error occurs in any child component, it catches the error, logs it for debugging, ' +
          'and renders a static fallback UI to ensure the application remains functional.',
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    children: {
      description: 'Child components to render (typically animated components)',
      control: false,
    },
    fallback: {
      description: 'Optional custom fallback UI to render when an error occurs',
      control: false,
    },
    onError: {
      description: 'Optional callback invoked when an error is caught',
      action: 'error-caught',
    },
  },
} satisfies Meta<typeof AnimationErrorBoundary>;

export default meta;
type Story = StoryObj<typeof meta>;
type RenderOnlyStory = Omit<Story, 'args'> & { args?: Partial<Story['args']> };

/**
 * Default story showing children rendering normally without any errors.
 * The error boundary passes through children when no error occurs.
 */
export const Default: Story = {
  args: {
    children: (
      <div
        style={{
          padding: '1rem',
          backgroundColor: '#1a1a2e',
          borderRadius: '0.5rem',
          color: '#e0e0e0',
        }}
      >
        Animated content renders successfully
      </div>
    ),
  },
};

/**
 * Story demonstrating the default fallback UI when an error is caught.
 * Click "Trigger Error" to simulate an error and see the fallback UI.
 */
export const WithErrorState: RenderOnlyStory = {
  render: () => <InteractiveErrorDemo onError={fn()} />,
  parameters: {
    docs: {
      description: {
        story:
          'Interactive demonstration of the error boundary. Click "Trigger Error" to throw an error ' +
          'and see the default fallback UI. Click "Reset" to restore the component.',
      },
    },
  },
};

/**
 * Story demonstrating a custom fallback UI when an error is caught.
 * The custom fallback replaces the default "Animation unavailable" message.
 */
export const WithCustomFallback: RenderOnlyStory = {
  render: () => (
    <InteractiveErrorDemo
      fallback={
        <div
          style={{
            padding: '1.5rem',
            backgroundColor: '#2d1b1b',
            border: '1px solid #dc3545',
            borderRadius: '0.5rem',
            color: '#ff6b6b',
            textAlign: 'center',
          }}
        >
          <strong>Custom Error Fallback</strong>
          <p style={{ margin: '0.5rem 0 0', fontSize: '0.875rem' }}>
            Something went wrong with the animation.
          </p>
        </div>
      }
      onError={fn()}
    />
  ),
  parameters: {
    docs: {
      description: {
        story:
          'Demonstrates using a custom fallback component. Click "Trigger Error" to see the custom ' +
          'fallback UI instead of the default message.',
      },
    },
  },
};

/**
 * Story demonstrating the onError callback functionality.
 * The callback is invoked with error details when an error is caught.
 * Check the Actions panel in Storybook to see the callback invocation.
 */
export const WithErrorCallback: RenderOnlyStory = {
  render: () => {
    const handleError = fn((error: Error, errorInfo: React.ErrorInfo) => {
      console.log('Error caught:', error.message);
      console.log('Component stack:', errorInfo.componentStack);
    });
    return <InteractiveErrorDemo onError={handleError} />;
  },
  parameters: {
    docs: {
      description: {
        story:
          'Demonstrates the onError callback. Click "Trigger Error" and check the Actions panel ' +
          'to see the error details passed to the callback.',
      },
    },
  },
};
