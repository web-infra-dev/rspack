/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    a: './index.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
  incremental: {
    buildChunkGraph: false,
  },
};
