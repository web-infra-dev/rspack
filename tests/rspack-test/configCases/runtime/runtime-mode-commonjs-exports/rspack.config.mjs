/** @type {import("@rspack/core").Configuration} */
export default {
  experiments: {
    runtimeMode: 'rspack',
  },
  optimization: {
    usedExports: true,
  },
};
