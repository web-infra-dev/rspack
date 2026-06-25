/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'web',
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: 'postcss-loader',
            options: {
              postcssOptions: {
                plugins: [require.resolve('postcss-pxtorem')],
              },
            },
          },
        ],
        type: 'css/auto',
        generator: {
          exportsOnly: false,
        },
      },
    ],
  },
};
