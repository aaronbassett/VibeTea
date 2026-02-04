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

/**
 * CSS styles injected to simulate prefers-reduced-motion: reduce.
 * These styles disable CSS animations and transitions when active.
 */
const REDUCED_MOTION_STYLES = `
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
`;

/**
 * Props for the ReducedMotionWrapper component.
 */
interface ReducedMotionWrapperProps {
  /** Whether reduced motion mode is enabled */
  enabled: boolean;
  /** The children to render */
  children?: React.ReactNode;
}

/**
 * Wrapper component that simulates reduced motion preference.
 * When enabled, it injects CSS that disables animations and patches
 * matchMedia to return true for the reduced-motion query.
 */
function ReducedMotionWrapper({ enabled, children }: ReducedMotionWrapperProps): React.ReactElement {
  React.useEffect(() => {
    if (enabled) {
      // Create and inject style element for CSS-based animation disabling
      const styleEl = document.createElement('style');
      styleEl.id = 'storybook-reduced-motion-styles';
      styleEl.textContent = REDUCED_MOTION_STYLES;
      document.head.appendChild(styleEl);

      // Patch matchMedia to simulate prefers-reduced-motion: reduce
      const originalMatchMedia = window.matchMedia;
      window.matchMedia = (query: string): MediaQueryList => {
        if (query === '(prefers-reduced-motion: reduce)') {
          return {
            matches: true,
            media: query,
            onchange: null,
            addListener: () => {},
            removeListener: () => {},
            addEventListener: () => {},
            removeEventListener: () => {},
            dispatchEvent: () => true,
          } as MediaQueryList;
        }
        return originalMatchMedia(query);
      };

      return () => {
        // Remove injected styles
        const existingStyle = document.getElementById('storybook-reduced-motion-styles');
        if (existingStyle) {
          existingStyle.remove();
        }
        // Restore original matchMedia
        window.matchMedia = originalMatchMedia;
      };
    }
    return undefined;
  }, [enabled]);

  return React.createElement(React.Fragment, null, children);
}

/**
 * Decorator that simulates reduced motion preference for testing.
 * Uses the ReducedMotionWrapper component to properly handle React hooks.
 */
const withReducedMotion: Decorator = (Story, context) => {
  const { reducedMotion } = context.globals;

  return React.createElement(
    ReducedMotionWrapper,
    { enabled: Boolean(reducedMotion) },
    React.createElement(Story)
  );
};

const preview: Preview = {
  globalTypes: {
    reducedMotion: {
      description: 'Simulate prefers-reduced-motion: reduce',
      toolbar: {
        title: 'Reduced Motion',
        icon: 'accessibility',
        items: [
          { value: false, title: 'Motion enabled', icon: 'play' },
          { value: true, title: 'Reduced motion', icon: 'stop' },
        ],
        dynamicTitle: true,
      },
    },
  },
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
  decorators: [withReducedMotion, withLazyMotion],
};

export default preview;