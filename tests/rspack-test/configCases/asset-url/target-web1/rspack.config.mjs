/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'web',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        dependency: 'url',
        loader: 'url-loader',
      },
    ],
  },
};
