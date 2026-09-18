/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    vendor: ['./a'],
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      minSize: 1,
      name: 'vendor',
    },
  },
};
