/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'web',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: '/path2/',
  },
};
