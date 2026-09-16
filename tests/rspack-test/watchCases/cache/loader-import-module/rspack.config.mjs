/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.generate-json\.js$/,
        use: './loader',
        type: 'json',
      },
    ],
  },
};
