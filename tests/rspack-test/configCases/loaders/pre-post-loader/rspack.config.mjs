/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: './loader1.mjs',
      },
      {
        test: /a\.js$/,
        use: './loader2.mjs',
        enforce: 'pre',
      },
      {
        test: /a\.js$/,
        use: './loader3.mjs',
        enforce: 'post',
      },
    ],
  },
};
