/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    module: true,
    library: { type: 'module' },
    iife: false,
    chunkFormat: 'module',
    filename: 'bundle0.mjs',
  },
  target: 'node',
  optimization: {
    minimize: true,
  },
};
