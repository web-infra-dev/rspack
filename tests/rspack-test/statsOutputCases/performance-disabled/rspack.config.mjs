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
  performance: false,
};
