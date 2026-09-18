/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'node',
  entry: {
    index: './index.js',
    sut: './sut.js',
  },
  output: {
    pathinfo: true,
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    concatenateModules: false,
  },
};
