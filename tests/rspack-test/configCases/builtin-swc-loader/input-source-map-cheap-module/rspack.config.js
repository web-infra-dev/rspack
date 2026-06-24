const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    '@rspack/test-tools/helper/util/checkSourceMap':
      'commonjs @rspack/test-tools/helper/util/checkSourceMap',
  },
  mode: 'development',
  devtool: 'cheap-module-source-map',
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /a\.jsx$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              sourceMaps: true,
            },
          },
          './prev-loader',
        ],
      },
    ],
  },
  plugins: [
    new DefinePlugin({
      CONTEXT: JSON.stringify(__dirname),
    }),
  ],
};
