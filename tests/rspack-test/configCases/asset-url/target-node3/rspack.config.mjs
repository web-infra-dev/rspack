/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'node',
  devtool: false,
  output: {
    assetModuleFilename: '[name][ext]',
    publicPath: 'https://example.com/',
  },
};
