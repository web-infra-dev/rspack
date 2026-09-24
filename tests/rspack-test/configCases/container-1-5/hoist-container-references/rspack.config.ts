import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    splitChunks: {
      chunks: 'all',
    },
    moduleIds: 'named',
  },
  plugins: [new ModuleFederationPlugin({})],
});
