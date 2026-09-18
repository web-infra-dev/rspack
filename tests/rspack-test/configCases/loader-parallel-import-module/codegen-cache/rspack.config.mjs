/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    rules: [
      {
        test: /app-proxy\.js/,
        use: [
          {
            loader: './loader.mjs',
            options: {},
            parallel: true,
          },
        ],
      },
    ],
  },
};
