/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    index: {
      import: ['./index.js'],
    },
  },
  optimization: {
    providedExports: true,
    usedExports: 'global',
  },
};
