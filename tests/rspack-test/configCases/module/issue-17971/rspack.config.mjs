/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    path: 'node-commonjs path',
  },
  output: {
    module: true,
  },
  devtool: 'eval-source-map',
  target: 'node',
};
