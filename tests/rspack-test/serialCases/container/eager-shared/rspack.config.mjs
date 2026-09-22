import packageJson from './package.json' with { type: 'json' };
import { container } from '@rspack/core';

const { dependencies } = packageJson;
const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
