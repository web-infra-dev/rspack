/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  devtool: 'cheap-source-map',
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  },
};
