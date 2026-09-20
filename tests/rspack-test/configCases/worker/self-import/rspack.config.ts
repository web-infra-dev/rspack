import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig([
  {
    target: 'web',
    plugins: [
      // TODO: support type: "module" injection
      new DefinePlugin({
        MODULE_FLAG: 'undefined',
      }),
    ],
  },
  {
    output: {
      filename: '[name].bundle1.js',
    },
    target: 'web',
    optimization: {
      runtimeChunk: 'single',
    },
    plugins: [
      // TODO: support type: "module" injection
      new DefinePlugin({
        MODULE_FLAG: 'undefined',
      }),
    ],
  },
  {
    target: 'web',
    output: {
      module: true,
    },
    plugins: [
      // TODO: support type: "module" injection
      new DefinePlugin({
        MODULE_FLAG: '"module"',
      }),
    ],
  },
  {
    target: 'web',
    output: {
      filename: '[name].bundle3.mjs',
      module: true,
    },
    optimization: {
      runtimeChunk: 'single',
    },
    plugins: [
      // TODO: support type: "module" injection
      new DefinePlugin({
        MODULE_FLAG: '"module"',
      }),
    ],
  },
]);
