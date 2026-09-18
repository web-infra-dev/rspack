/**
 * @type {import('@rspack/core').Configuration}
 */
export default {
  context: import.meta.dirname,
  output: {
    publicPath: '/',
    assetModuleFilename: '[path][name][ext][query]',
  },
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
};
