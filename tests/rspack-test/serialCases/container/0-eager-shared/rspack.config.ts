import { defineConfig } from '@rspack/cli';

import packageJson from './package.json' with { type: 'json' };
import { container } from '@rspack/core';

const { dependencies } = packageJson;
const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: {
        './emitter': {
          name: 'emitter',
          import: './emitter.js',
        },
      },
      shared: {
        ...dependencies,
      },
    }),
  ],
});
