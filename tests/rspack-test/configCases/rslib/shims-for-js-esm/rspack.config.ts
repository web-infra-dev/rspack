import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  entry: {
    index: './index.js',
  },
  target: 'node',
  node: {
    __filename: 'node-module',
    __dirname: 'node-module',
  },
  optimization: {
    concatenateModules: false,
  },
  module: {
    rules: [
      {
        // set every module type to javascript/esm
        type: 'javascript/esm',
      },
    ],
  },
  output: {
    module: true,
    library: {
      type: 'modern-module',
    },
    filename: 'bundle.mjs',
  },
  plugins: [
    new RslibPlugin({
      forceNodeShims: true,
    }),
  ],
});
