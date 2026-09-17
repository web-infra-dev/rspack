/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './example.js',
  optimization: {
    usedExports: true,
    providedExports: true,
  },
  stats: {
    assets: true,
    modules: true,
    usedExports: true,
    providedExports: true,
  },
};
