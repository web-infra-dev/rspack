/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: {
    entry: './entry',
  },
  stats: {
    assets: true,
    modules: true,
  },
};
