/** @type {import('@rspack/core').RspackOptions} */
module.exports = {
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: ['./loader.js'],
      },
    ],
  },
};
