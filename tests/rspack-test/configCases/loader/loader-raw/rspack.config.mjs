/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        use: [{ loader: './loader.js' }],
        type: 'asset/resource',
      },
    ],
  },
};
