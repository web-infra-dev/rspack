/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  target: 'web',
  optimization: {
    runtimeChunk: true,
  },
};
