/**@type {import("@rspack/core").Configuration}*/
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
      {
        test: /\.css/,
        type: 'css/auto',
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
