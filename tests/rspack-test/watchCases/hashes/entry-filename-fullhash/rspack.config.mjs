/** @type {import("@rspack/core").Configuration} */
export default {
  incremental: true,
  entry: {
    first: {
      import: './first.js',
      filename: '[name].[fullhash].js',
    },
    second: './second.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
};
