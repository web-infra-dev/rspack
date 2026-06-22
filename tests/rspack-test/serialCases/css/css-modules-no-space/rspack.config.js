/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    './use-style_js.bundle0.js': 'commonjs ./use-style_js.bundle0.js',
  },
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.my-css$/i,
        type: 'css/auto',
      },
      {
        test: /\.invalid$/i,
        type: 'css/auto',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  node: {
    __dirname: false,
    __filename: false,
  },
};
