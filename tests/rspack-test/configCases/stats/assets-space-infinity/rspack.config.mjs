/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  entry: './index.js',
  stats: {
    assetsSpace: Infinity,
  },
};
