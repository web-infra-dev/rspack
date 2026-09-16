/** @type {import("../../../../").Configuration} */
export default {
  target: 'node14',
  entry: './index.js',
  optimization: {
    chunkIds: 'named',
  },
  output: {
    module: true,
    filename: 'bundle.mjs',
  },
};
