/** @type {import("@rspack/core").Configuration} */
export default {
  devtool: 'source-map',
  optimization: {
    minimize: true,
    concatenateModules: false,
  },
  externals: ['source-map'],
  externalsType: 'commonjs',
};
