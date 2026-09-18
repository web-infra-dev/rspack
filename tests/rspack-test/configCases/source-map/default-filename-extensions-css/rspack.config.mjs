/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    filename: 'bundle0.css',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
};
