/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: './loader',
      },
    ],
  },
};
