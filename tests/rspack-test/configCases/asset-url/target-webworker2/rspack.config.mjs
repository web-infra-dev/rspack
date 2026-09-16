/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'webworker',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: '/',
  },
};
