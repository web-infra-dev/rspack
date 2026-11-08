import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

export default [false, true].flatMap((eager) =>
  [false, true].map((concatenateModules) =>
    defineConfig({
      mode: 'production',
      optimization: {
        concatenateModules,
        minimize: false,
      },
      plugins: [
        new sharing.SharePlugin({
          shared: {
            './cjs.js': { eager, singleton: true, requiredVersion: false },
            './esm.mjs': { eager, singleton: true, requiredVersion: false },
          },
        }),
      ],
    }),
  ),
);
