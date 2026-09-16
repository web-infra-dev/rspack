/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index',
  },
  target: 'node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
};
