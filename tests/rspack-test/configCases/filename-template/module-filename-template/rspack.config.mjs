/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    devtoolModuleFilenameTemplate: function (info) {
      return 'dummy:///' + info.resourcePath;
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'cheap-source-map',
};
