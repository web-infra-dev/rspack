/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    environment: {
      templateLiteral: false,
    },
  },
  module: {
    rules: [
      {
        test: /\.(png|svg|jpg)$/,
        type: 'asset/resource',
      },
    ],
  },
};
