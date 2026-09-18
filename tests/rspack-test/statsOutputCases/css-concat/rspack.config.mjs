/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  stats: {
    entrypoints: true,
    assets: true,
    modules: true,
  },
};
