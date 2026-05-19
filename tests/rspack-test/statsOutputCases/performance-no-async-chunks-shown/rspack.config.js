/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  entry: {
    main: './index',
    sec: './index2',
  },
  performance: {
    hints: 'warning',
    maxAssetSize: 200 * 1024,
    maxEntrypointSize: 200 * 1024,
  },
  stats: {
    assets: true,
    modules: true,
    colors: true,
    hash: false,
    entrypoints: true,
  },
};
