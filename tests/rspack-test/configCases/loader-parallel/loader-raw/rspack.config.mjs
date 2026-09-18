/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        use: [{ loader: './loader.mjs', parallel: true, options: {} }],
        type: 'asset/resource',
      },
    ],
  },
};
