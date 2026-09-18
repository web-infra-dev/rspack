/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index',
    misc: './second',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 0,
    },
  },
};
