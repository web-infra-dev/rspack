/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  stats: {
    assets: true,
    modules: true,
    colors: true,
    hash: false,
    entrypoints: true,
  },
  performance: {
    hints: 'error',
    maxAssetSize: 200 * 1024,
    maxEntrypointSize: 200 * 1024,
  },
};
