/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
    chunkFilename: 'main.[contenthash:8].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
};
