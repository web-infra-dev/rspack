/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: {
      import: './index',
      runtime: false,
    },
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
};
