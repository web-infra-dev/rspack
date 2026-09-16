import { HotModuleReplacementPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  mode: 'development',
  plugins: [new HotModuleReplacementPlugin()],
  module: {
    rules: [
      {
        loader: './loader.js',
      },
    ],
  },
};
