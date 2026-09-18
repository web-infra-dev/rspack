/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  entry: './index.mjs',
  performance: {
    hints: false,
  },
  optimization: {
    minimize: false,
  },
};
