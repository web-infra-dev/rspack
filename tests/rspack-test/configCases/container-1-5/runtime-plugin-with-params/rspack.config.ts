import { defineConfig } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  plugins: [
    new ModuleFederationPlugin({
      name: 'container-runtime-plugin-with-params',
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      shared: {
        react: {
          version: '0.1.2',
          requiredVersion: false,
          singleton: true,
          strictVersion: false,
        },
      },
      runtimePlugins: [
        [
          './plugin-with-params.mjs',
          {
            'custom-params': {
              msg: 'custom-params',
            },
          },
        ],
      ],
    }),
  ],
});
