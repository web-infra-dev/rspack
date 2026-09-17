/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  optimization: {
    minimize: false,
  },
  output: {
    library: {
      type: 'module',
    },
    filename: '[name].mjs',
    module: true,
    chunkFormat: 'module',
    chunkLoading: 'import',
  },
};
