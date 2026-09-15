/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    './lazy4_js.bundle0.js': 'commonjs ./lazy4_js.bundle0.js',
  },
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        css: {
          type: 'css/auto',
          enforce: true,
          name: 'css',
        },
      },
    },
  },
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  },
};
