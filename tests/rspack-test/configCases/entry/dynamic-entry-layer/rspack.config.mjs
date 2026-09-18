/** @type {import("@rspack/core").Configuration} */
export default {
  entry() {
    return Promise.resolve({
      bundle0: {
        import: './index.js',
        layer: 'client',
      },
    });
  },
};
