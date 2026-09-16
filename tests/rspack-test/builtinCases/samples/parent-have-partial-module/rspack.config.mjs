/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  optimization: {
    removeAvailableModules: true,
    providedExports: true,
    usedExports: 'global',
  },
};
