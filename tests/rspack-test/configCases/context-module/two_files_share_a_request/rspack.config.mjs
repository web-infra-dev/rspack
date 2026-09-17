/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    extensions: ['.ts', '...'],
  },
  optimization: {
    minimize: false,
    moduleIds: 'named',
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        type: 'javascript/auto',
      },
    ],
  },
};
