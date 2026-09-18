/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  resolve: {
    alias: {
      './ignored-module': false,
    },
  },
  devtool: 'source-map',
};
