/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    assetModuleFilename: 'file[ext]',
    publicPath: 'assets/',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
        generator: {
          publicPath: '',
        },
      },
    ],
  },
};
