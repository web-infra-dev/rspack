/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  devtool: false,
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  },
};
