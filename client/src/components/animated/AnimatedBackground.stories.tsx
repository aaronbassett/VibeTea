import type { Meta, StoryObj } from '@storybook/react';

import { AnimatedBackground } from './AnimatedBackground';

/**
 * AnimatedBackground renders a GPU-accelerated animated background featuring
 * a flickering grid and floating particle effects. It automatically pauses
 * when the tab is not visible and respects the user's reduced motion preference.
 *
 * The component uses:
 * - A flickering grid background with 20px cells
 * - Floating particle/twinkle effects (10-20 particles with slow drift)
 * - Page Visibility API to pause animations when tab is hidden
 * - prefers-reduced-motion media query to respect accessibility settings
 */
const meta = {
  title: 'Animated/AnimatedBackground',
  component: AnimatedBackground,
  parameters: {
    // Full-screen layout since this is a background component
    layout: 'fullscreen',
    // Disable padding in the story canvas
    docs: {
      story: {
        inline: false,
        iframeHeight: 400,
      },
    },
  },
  tags: ['autodocs'],
  argTypes: {
    showGrid: {
      control: 'boolean',
      description: 'Show the flickering grid background',
      table: {
        defaultValue: { summary: 'true' },
      },
    },
    showParticles: {
      control: 'boolean',
      description: 'Show floating particle effects',
      table: {
        defaultValue: { summary: 'true' },
      },
    },
    className: {
      control: 'text',
      description: 'Additional CSS classes for the container',
    },
  },
  // Decorator to provide a visible container for the background
  decorators: [
    (Story) => (
      <div
        style={{
          position: 'relative',
          width: '100%',
          height: '100vh',
          minHeight: '400px',
          backgroundColor: '#0a0a0a',
        }}
      >
        <Story />
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            color: '#ffffff',
            fontFamily: 'monospace',
            fontSize: '14px',
            textAlign: 'center',
            zIndex: 1,
            padding: '20px',
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            borderRadius: '8px',
          }}
        >
          <p>AnimatedBackground renders behind this content</p>
          <p style={{ opacity: 0.7, marginTop: '8px' }}>
            (Grid and particles are visible in the background)
          </p>
        </div>
      </div>
    ),
  ],
} satisfies Meta<typeof AnimatedBackground>;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * Default state with both grid and particles enabled.
 * This is the standard configuration for the VibeTea dashboard.
 */
export const Default: Story = {
  args: {
    showGrid: true,
    showParticles: true,
  },
};

/**
 * Grid only variation with particles disabled.
 * Useful for a more subtle background effect or when
 * particle animations may be distracting.
 */
export const GridOnly: Story = {
  args: {
    showGrid: true,
    showParticles: false,
  },
};

/**
 * Particles only variation with grid disabled.
 * Creates a starfield-like effect without the grid structure.
 */
export const ParticlesOnly: Story = {
  args: {
    showGrid: false,
    showParticles: true,
  },
};

/**
 * Demonstrates reduced motion behavior.
 *
 * When the user has enabled "prefers-reduced-motion" in their system settings,
 * the AnimatedBackground component automatically:
 * - Pauses all animations (grid flickering and particle movement)
 * - Reduces the number of rendered grid cells to 0
 * - Reduces the number of particles to 0
 *
 * To test this story properly:
 * 1. Open your system accessibility settings
 * 2. Enable "Reduce motion" (macOS) or "Show animations" off (Windows)
 * 3. The component will automatically respond to this preference
 *
 * Note: This story shows the component with props enabled, but the actual
 * rendering depends on the browser's prefers-reduced-motion media query state.
 */
export const ReducedMotionBehavior: Story = {
  args: {
    showGrid: true,
    showParticles: true,
  },
  parameters: {
    docs: {
      description: {
        story: `
This story demonstrates the reduced motion behavior of the AnimatedBackground component.

**How reduced motion works:**

The component uses the \`useReducedMotion\` hook which listens to the
\`prefers-reduced-motion\` media query. When this preference is enabled:

- Grid cell count is set to 0 (no grid rendered)
- Particle count is set to 0 (no particles rendered)
- All animations are paused

**Testing reduced motion:**

1. **macOS**: System Preferences > Accessibility > Display > Reduce motion
2. **Windows**: Settings > Ease of Access > Display > Show animations (toggle off)
3. **Chrome DevTools**: Open DevTools > Rendering tab > Emulate CSS media feature prefers-reduced-motion

The component respects these settings automatically without any additional configuration.
        `,
      },
    },
  },
  decorators: [
    (Story) => (
      <div
        style={{
          position: 'relative',
          width: '100%',
          height: '100vh',
          minHeight: '400px',
          backgroundColor: '#0a0a0a',
        }}
      >
        <Story />
        <div
          style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            color: '#ffffff',
            fontFamily: 'monospace',
            fontSize: '14px',
            textAlign: 'center',
            zIndex: 1,
            padding: '20px',
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            borderRadius: '8px',
            maxWidth: '400px',
          }}
        >
          <p style={{ fontWeight: 'bold', marginBottom: '12px' }}>
            Reduced Motion Demo
          </p>
          <p style={{ opacity: 0.9, lineHeight: 1.5 }}>
            Enable &quot;prefers-reduced-motion&quot; in your system settings or
            Chrome DevTools to see this component pause all animations.
          </p>
          <p
            style={{ opacity: 0.7, marginTop: '12px', fontSize: '12px' }}
          >
            DevTools: Rendering tab &rarr; Emulate prefers-reduced-motion
          </p>
        </div>
      </div>
    ),
  ],
};
