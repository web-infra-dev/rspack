/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index',
    second: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 1,
    },
  },
};
