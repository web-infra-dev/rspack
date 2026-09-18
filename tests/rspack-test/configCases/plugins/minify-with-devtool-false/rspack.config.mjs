import { SourceMapDevToolPlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  target: 'node',
  entry: {
    main: './index.js',
  },
  optimization: {
    minimize: true,
  },
  plugins: [new SourceMapDevToolPlugin({})],
  devtool: false,
};
