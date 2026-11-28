import { defineConfig } from '@rspack/cli';
import { container, HotModuleReplacementPlugin } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  externals: {
    './container.js': 'commonjs ./container.js',
  },
  plugins: [
    // The federation entry and container execute with distinct HMR child contexts.
    new HotModuleReplacementPlugin(),
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: ['./module'],
    }),
  ],
});
