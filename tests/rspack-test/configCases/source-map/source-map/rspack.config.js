const { rspack } = require('@rspack/core');
/** @type {import("@rspack/core").Configuration} */
module.exports = {
  devtool: 'source-map',
  externals: ['source-map'],
  externalsType: 'commonjs',
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.jsx$/,
        loader: 'builtin:swc-loader',
        options: {
          detectSyntax: 'auto',
        },
      },
    ],
  },
  plugins: [
    new rspack.DefinePlugin({
      CONTEXT: JSON.stringify(__dirname),
    }),
  ],
};
