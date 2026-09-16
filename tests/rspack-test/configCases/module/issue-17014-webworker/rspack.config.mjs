/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    module: true,
  },
  target: ['web', 'es2020'],
  optimization: {
    splitChunks: {
      minSize: 1,
      maxSize: 1,
    },
  },
};
