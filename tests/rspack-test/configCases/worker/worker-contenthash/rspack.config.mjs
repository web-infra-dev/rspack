/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index.js',
      filename: '[name].js',
    },
  },
  output: {
    filename: '[name]-[contenthash].js',
  },
};
