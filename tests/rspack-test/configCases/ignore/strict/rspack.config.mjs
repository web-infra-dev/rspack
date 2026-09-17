/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
  },
  entry: './index.mjs',
  resolve: {
    alias: {
      './ignored-module': false,
    },
  },
  output: {
    iife: false,
  },
};
