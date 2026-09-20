import { defineConfig } from '@rspack/cli';

import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  output: {
    strictModuleExceptionHandling: true,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      library: { type: 'commonjs-module' },
      filename: 'container.js',
      exposes: ['./module'],
      remotes: {
        remote: './container.js',
        invalid: './invalid.js',
      },
    }),
  ],
});
