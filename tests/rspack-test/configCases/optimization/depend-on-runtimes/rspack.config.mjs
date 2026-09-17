/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
  },
  entry: {
    a: './a',
    b: './b',
    c: {
      import: './c',
      runtime: 'runtime-c',
    },
  },
};
