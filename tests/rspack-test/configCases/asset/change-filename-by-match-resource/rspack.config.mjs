/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    assetModuleFilename: 'assets/[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        use: './loader.js',
        type: 'asset/resource',
      },
    ],
  },
};
