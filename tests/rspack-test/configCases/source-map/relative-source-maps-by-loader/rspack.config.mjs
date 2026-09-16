/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
};
