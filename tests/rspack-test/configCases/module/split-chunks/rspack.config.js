/** @type {import("@rspack/core").Configuration} */
module.exports = {
  output: {
    module: true,
    filename: '[name].mjs',
    library: {
      type: 'module',
    },
  },
  target: ['web', 'es2020'],
  optimization: {
    minimize: true,
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        separate: {
          test: /separate/,
          chunks: 'all',
          filename: 'separate.mjs',
          enforce: true,
        },
      },
    },
    // Avoid the default export of separate.js is inlined, which causes the splitChunks separate cache group disappear.
    inlineExports: false,
  },
  externals: {
    'external-self': './main.mjs',
  },
};
