/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: './loader',
      },
    ],
  },
};
