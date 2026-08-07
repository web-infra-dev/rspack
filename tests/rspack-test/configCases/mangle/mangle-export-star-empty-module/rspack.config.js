/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  module: {
    rules: [
      {
        test: /loader-empty\.js$/,
        use: './empty-loader.js',
      },
    ],
  },
  output: {
    pathinfo: 'verbose',
  },
  optimization: {
    minimize: false,
    concatenateModules: false,
    inlineExports: false,
    providedExports: true,
    usedExports: true,
    mangleExports: 'size',
    sideEffects: false,
  },
};
