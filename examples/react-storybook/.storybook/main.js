/** @type { import('@storybook/react-webpack5').StorybookConfig } */
const config = {
  stories: ['../src/stories/**/*.mdx', '../src/stories/**/*.stories.@(js|jsx|ts|tsx)'],
  addons: [
    '@storybook/addon-links',
    '@storybook/addon-essentials',
    '@storybook/addon-interactions',
  ],
  framework: {
    name: 'storybook-react-rspack',
    // name: '@storybook/react-webpack5',
    options: {
      fastRefresh: true,
    },
  },
  docs: {
    autodocs: 'tag',
  },
  typescript: {
    reactDocgen: 'react-docgen',
    // reactDocgen: true,
  },
};
export default config;
