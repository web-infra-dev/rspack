import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';
import packageJson from './package.json' with { type: 'json' };

const { dependencies } = packageJson;
const { ModuleFederationPlugin } = container;

export default defineConfig({
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
  },
  plugins: [
    new ModuleFederationPlugin({
      remoteType: 'commonjs-module',
      remotes: {
        service: '../0-eager-shared/container.js',
      },
      shared: {
        'tiny-emitter': {
          eager: true,
          singleton: true,
          requiredVersion: dependencies['tiny-emitter'],
        },
      },
    }),
  ],
});
