/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
