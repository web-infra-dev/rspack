/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    sideEffects: true,
  },
  externalsPresets: {
    node: true,
  },
};
