/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    a: './a.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[runtime].[contenthash].[name].js',
  },
};
