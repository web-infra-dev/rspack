/** @type {import('@rspack/core').Configuration} */
const createConfig = (parallel) => ({
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [
          {
            loader: './loader.js',
            parallel,
            options: {},
          },
        ],
      },
      {
        test: /stats\.js$/,
        use: [{ loader: './stats-loader.js', parallel }],
      },
    ],
  },
});

module.exports = [createConfig(true), createConfig(false)];
