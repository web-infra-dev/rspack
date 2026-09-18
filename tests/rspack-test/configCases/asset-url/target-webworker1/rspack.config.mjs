/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  target: 'webworker',
  devtool: false,
  output: {
    filename: 'deep/path/[name].js',
    assetModuleFilename: '[path][name][ext]',
    publicPath: '',
  },
};
