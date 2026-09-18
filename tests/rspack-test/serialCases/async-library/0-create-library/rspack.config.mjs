/** @type {import("../../../../types").Configuration} */
export default {
  entry: './a.js',
  output: {
    module: true,
    filename: 'lib.js',
    library: {
      type: 'module',
    },
  },
  target: 'node14',
  optimization: {
    minimize: true,
  },
};
