/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  mode: 'development',
  externalsPresets: { web: false, webAsync: true },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
