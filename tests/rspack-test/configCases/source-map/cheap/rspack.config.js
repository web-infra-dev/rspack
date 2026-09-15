/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.s[ac]ss$/i,
        use: [{ loader: 'sass-loader' }],
        type: 'css',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
  devtool: 'cheap-source-map',
  externals: [
    {
      fs: 'node-commonjs fs',
    },
    {
      'source-map': 'commonjs source-map',
    },
    'source-map',
  ],
  externalsType: 'commonjs',
};
