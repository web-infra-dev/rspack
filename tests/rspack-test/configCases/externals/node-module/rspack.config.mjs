/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  output: {
    module: true,
    chunkFormat: 'module',
    filename: '[name].mjs',
  },
};
