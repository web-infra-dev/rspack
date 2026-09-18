/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  externals: {
    test: 'commonjs test',
  },
  stats: {
    assets: true,
    modules: true,
  },
};
