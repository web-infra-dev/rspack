/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    uniqueName: 'css-test',
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
};
