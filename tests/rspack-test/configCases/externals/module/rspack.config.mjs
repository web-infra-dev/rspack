/** @type {import("@rspack/core").Configuration} */
export default {
  externals: ['path'],
  externalsType: 'module',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
  },
};
