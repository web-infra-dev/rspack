/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'none',
  target: 'node',
  output: {
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
    ],
  },
};
