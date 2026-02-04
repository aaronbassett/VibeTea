import type { Preview, Decorator } from '@storybook/react';
import * as React from 'react';
import { LazyMotion, domAnimation } from 'framer-motion';

// Import global CSS with Tailwind CSS 4
import '../src/index.css';

// Import animation CSS
import '../src/styles/animations.css';

/**
 * LazyMotion decorator that wraps all stories with framer-motion's
 * LazyMotion provider using domAnimation features (~15KB bundle).
 * This matches the App.tsx configuration for consistency.
 */
const withLazyMotion: Decorator = (Story) => {
  return React.createElement(
    LazyMotion,
    { features: domAnimation },
    React.createElement(Story)
  );
};

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      default: 'dark',
      values: [
        {
          name: 'dark',
          value: '#131313', // COLORS.background.primary from design-tokens.ts
        },
        {
          name: 'dark-secondary',
          value: '#1a1a1a', // COLORS.background.secondary
        },
        {
          name: 'dark-tertiary',
          value: '#242424', // COLORS.background.tertiary
        },
      ],
    },
  },
  decorators: [withLazyMotion],
};

export default preview;