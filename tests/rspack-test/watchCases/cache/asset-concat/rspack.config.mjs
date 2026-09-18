/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  cache: true,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/inline',
      },
      {
        test: /\.jpg$/,
        type: 'asset/resource',
      },
      {
        test: /\.svg$/,
        type: 'asset',
      },
    ],
  },
  optimization: {
    concatenateModules: true,
  },
};
