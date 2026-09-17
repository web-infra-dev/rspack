/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  output: {
    assetModuleFilename: 'images/file[ext]',
  },
  module: {
    generator: {
      'asset/resource': {
        emit: false,
      },
    },
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
    ],
  },
};
