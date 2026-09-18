/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'webworker',
  devtool: false,
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
};
