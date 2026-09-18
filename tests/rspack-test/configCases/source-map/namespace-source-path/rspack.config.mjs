/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    devtoolNamespace: 'mynamespace',
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'cheap-source-map',
};
