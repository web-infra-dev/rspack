/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: {
    react: './react',
  },
  optimization: {
    minimize: true,
    chunkIds: 'named',
  },
  stats: {
    assets: true,
    modules: true,
  },
};
