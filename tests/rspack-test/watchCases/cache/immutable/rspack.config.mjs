/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  cache: true,
  output: {
    filename: 'bundle.js?[contenthash]',
  },
};
