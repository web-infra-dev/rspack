/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
  optimization: {
    minimize: true,
  },
};
