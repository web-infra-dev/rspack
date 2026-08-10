/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  output: {
    publicPath: 'https://test.cases/path/',
    chunkFilename: '[name].js',
    cssChunkFilename: '[name].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        dependency: 'url',
        type: 'css',
      },
      {
        test: /target\.js$/,
        dependency: 'url',
        type: 'javascript/auto',
      },
    ],
  },
  node: {
    __dirname: false,
  },
};
