/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.js$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            // SWC reads `null` as an unset target.
            jsc: { target: null },
          },
        },
      },
    ],
  },
};
