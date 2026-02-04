import type { StorybookConfig } from '@storybook/react-vite';

const config: StorybookConfig = {
  stories: ['../src/**/*.mdx', '../src/**/*.stories.@(js|jsx|mjs|ts|tsx)'],
  addons: [
    '@storybook/addon-essentials',
    '@storybook/addon-onboarding',
    '@storybook/addon-interactions',
  ],
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
  viteFinal: async (config) => {
    // Tailwind CSS 4 works out of the box with Vite
    // The @tailwindcss/vite plugin is already configured in the main vite.config.ts
    // Storybook's react-vite builder automatically picks up the Vite config
    return config;
  },
};

export default config;