/** @type {import("@rspack/core").Configuration[]} */
module.exports = ['source-map', 'cheap-module-source-map'].map((devtool) => ({
  mode: 'development',
  devtool,
  externals: ['source-map'],
  externalsType: 'commonjs',
  optimization: { concatenateModules: false },
}));
