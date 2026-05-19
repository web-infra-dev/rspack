/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: './index',
  performance: {
    hints: 'warning',
    maxAssetSize: 200 * 1024,
    maxEntrypointSize: 200 * 1024,
  },
  stats: {
    assets: true,
    modules: true,
    hash: false,
    colors: true,
  },
};
