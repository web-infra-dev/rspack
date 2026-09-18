/** @type {import('@rspack/core').Configuration} */
export default {
  module: {
    rules: [
      {
        test: /resource\.js$/,
        use: [
          {
            loader: './loader.mjs',
            parallel: true,
            options: {},
          },
        ],
      },
    ],
  },
};
