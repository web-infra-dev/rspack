import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  experiments: {
    runtimeMode: 'rspack',
  },
  target: 'async-node',
  output: {
    publicPath: '/assets/',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'runtime_mode_public_path',
      shared: {
        'shared-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
};
