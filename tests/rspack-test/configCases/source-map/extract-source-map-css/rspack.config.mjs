/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'css',
        extractSourceMap: true,
      },
    ],
  },
};
