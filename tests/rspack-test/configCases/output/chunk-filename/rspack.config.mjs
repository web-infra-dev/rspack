/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  target: 'node',
  output: {
    filename: '[name].js',
    chunkFilename: 'chunks/async-[name].[chunkhash:8].js',
  },
};
