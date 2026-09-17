/**@type {import("@rspack/core").Configuration}*/
export default {
  mode: 'development',
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
  output: {
    chunkFilename: '[name].js',
  },
  externalsPresets: {
    node: true,
  },
};
