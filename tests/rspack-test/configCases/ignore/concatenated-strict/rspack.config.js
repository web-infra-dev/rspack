/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
  optimization: {
    concatenateModules: true,
  },
};
