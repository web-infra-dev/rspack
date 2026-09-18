/** @type {import('@rspack/core').RspackOptions} */
export default {
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: ['./loader.mjs'],
      },
    ],
  },
};
