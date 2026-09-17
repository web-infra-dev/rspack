/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  optimization: {
    splitChunks: {
      minSize: 0,
    },
  },
};
