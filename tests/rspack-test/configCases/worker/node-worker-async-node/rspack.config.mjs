/** @type {import("../../../../").Configuration} */
export default {
  target: 'async-node14',
  entry: './index.js',
  optimization: {
    chunkIds: 'named',
  },
  output: {
    filename: 'bundle.js',
  },
};
