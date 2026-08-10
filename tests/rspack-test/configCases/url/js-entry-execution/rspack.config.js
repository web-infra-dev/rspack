/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  target: 'web',
  output: {
    publicPath: 'https://test.cases/path/',
    chunkFilename: 'url-[name].js',
  },
  module: {
    rules: [
      {
        test: /target\.js$/,
        dependency: 'url',
        type: 'javascript/auto',
      },
    ],
  },
};
