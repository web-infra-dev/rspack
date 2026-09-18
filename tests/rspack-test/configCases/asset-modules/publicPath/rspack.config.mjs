/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    publicPath: 'assets/',
    assetModuleFilename: 'file[ext]',
    environment: {
      templateLiteral: true,
    },
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
    ],
  },
};
