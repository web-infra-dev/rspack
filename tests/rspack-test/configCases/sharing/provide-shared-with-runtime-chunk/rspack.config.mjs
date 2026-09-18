import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;
/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
  plugins: [
    new ProvideSharedPlugin({
      provides: ['x'],
    }),
  ],
};
