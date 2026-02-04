import type { Meta, StoryObj } from '@storybook/react';
import { LazyMotion, domAnimation } from 'framer-motion';

import { ASCIIHeader } from './ASCIIHeader';

/**
 * ASCIIHeader displays pre-generated ASCII art with a spring entrance animation.
 * It respects the user's reduced motion preferences for accessibility.
 */
const meta: Meta<typeof ASCIIHeader> = {
  title: 'Animated/ASCIIHeader',
  component: ASCIIHeader,
  tags: ['autodocs'],
  parameters: {
    layout: 'centered',
  },
  decorators: [
    (Story) => (
      <LazyMotion features={domAnimation}>
        <div style={{ backgroundColor: '#1a1a2e', padding: '2rem' }}>
          <Story />
        </div>
      </LazyMotion>
    ),
  ],
  argTypes: {
    text: {
      description: 'Custom ASCII text to display',
      control: 'text',
    },
    animateOnLoad: {
      description: 'Whether to animate on mount',
      control: 'boolean',
    },
    className: {
      description: 'Additional CSS classes',
      control: 'text',
    },
  },
};

export default meta;
type Story = StoryObj<typeof ASCIIHeader>;

/**
 * Default state with animation enabled.
 * The ASCII logo animates in with a spring entrance effect.
 */
export const Default: Story = {
  args: {},
};

/**
 * Animation disabled state.
 * The ASCII logo appears immediately without any entrance animation.
 */
export const WithoutAnimation: Story = {
  args: {
    animateOnLoad: false,
  },
};

/**
 * Custom subtitle text displaying "Welcome".
 */
export const CustomSubtitleWelcome: Story = {
  args: {
    text: ` _    ___ __       ______
| |  / (_) /_  ___/_  __/__  ____ _
| | / / / __ \\/ _ \\/ / / _ \\/ __ \`/
| |/ / / /_/ /  __/ / /  __/ /_/ /
|___/_/_.___/\\___/_/  \\___/\\__,_/

        ~ Welcome ~`,
  },
};

/**
 * Custom subtitle text displaying "Dashboard".
 */
export const CustomSubtitleDashboard: Story = {
  args: {
    text: ` _    ___ __       ______
| |  / (_) /_  ___/_  __/__  ____ _
| | / / / __ \\/ _ \\/ / / _ \\/ __ \`/
| |/ / / /_/ /  __/ / /  __/ /_/ /
|___/_/_.___/\\___/_/  \\___/\\__,_/

       :: Dashboard ::`,
  },
};

/**
 * Custom subtitle text displaying version information.
 */
export const CustomSubtitleVersion: Story = {
  args: {
    text: ` _    ___ __       ______
| |  / (_) /_  ___/_  __/__  ____ _
| | / / / __ \\/ _ \\/ / / _ \\/ __ \`/
| |/ / / /_/ /  __/ / /  __/ /_/ /
|___/_/_.___/\\___/_/  \\___/\\__,_/

          v1.0.0`,
  },
};

/**
 * Minimal custom ASCII art for testing flexibility.
 */
export const MinimalAscii: Story = {
  args: {
    text: `
  ╔═══════════════╗
  ║   VibeTea     ║
  ╚═══════════════╝`,
    animateOnLoad: true,
  },
};
