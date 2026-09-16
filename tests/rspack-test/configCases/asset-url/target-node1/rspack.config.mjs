/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'node',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: 'public/',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        dependency: ['esm', 'commonjs'],
        loader: 'url-loader',
      },
    ],
  },
};
