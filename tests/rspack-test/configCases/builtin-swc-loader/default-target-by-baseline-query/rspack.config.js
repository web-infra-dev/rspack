/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
  },
  target: 'browserslist:baseline 2015',
  node: {
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        use: 'builtin:swc-loader',
      },
    ],
  },
};
