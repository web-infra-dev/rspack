/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    assetModuleFilename: 'images/file[ext]',
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
