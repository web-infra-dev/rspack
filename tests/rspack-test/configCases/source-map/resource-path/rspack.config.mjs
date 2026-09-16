/** @type {import("@rspack/core").Configuration} */
export default {
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
  entry: {
    main: {
      import: './index',
      layer: 'something',
    },
  },
  output: {
    devtoolModuleFilenameTemplate(info) {
      return info.absoluteResourcePath;
    },
  },
};
