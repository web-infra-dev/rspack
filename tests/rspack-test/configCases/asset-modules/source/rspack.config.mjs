/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/source',
      },
    ],
  },
};
