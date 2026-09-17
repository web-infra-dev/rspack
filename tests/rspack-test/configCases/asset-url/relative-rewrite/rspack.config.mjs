/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext][query][fragment]',
    publicPath: 'public/',
  },
  module: {
    parser: {
      javascript: {
        url: 'relative',
      },
    },
  },
};
