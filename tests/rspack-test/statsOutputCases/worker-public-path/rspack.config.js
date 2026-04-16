/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: './index.js',
  output: {
    compareBeforeEmit: false,
    filename: '[name]-[contenthash].js',
  },
  stats: {
    assets: true,
    modules: true,
  },
};
