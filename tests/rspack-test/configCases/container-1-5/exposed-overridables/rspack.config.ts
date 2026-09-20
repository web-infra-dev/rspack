import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = container;

export default defineConfig({
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      exposes: {
        './Button': './Button',
      },
      shared: {
        react: {
          eager: true,
        },
      },
    }),
  ],
});
