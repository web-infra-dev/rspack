/** @type {import("@rspack/core").Configuration} */
module.exports = {
  externals: {
    path: 'node-commonjs path',
  },
  entry: './src/index.js',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
        generator: {
          localIdentName: '[path][name]__[local]',
        },
      },
    ],
  },
};
