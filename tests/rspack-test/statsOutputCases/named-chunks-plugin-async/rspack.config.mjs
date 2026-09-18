/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  optimization: { chunkIds: 'named' },
  entry: {
    entry: './entry',
  },
  stats: {
    assets: true,
    modules: true,
  },
};
