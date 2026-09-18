/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.png/,
        type: 'asset/resource',
      },
    ],
  },
};
