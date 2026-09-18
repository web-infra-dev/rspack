/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.module\.css$/i,
        type: 'css/module',
      },
    ],
  },
  experiments: {
    css: true,
  },
};
