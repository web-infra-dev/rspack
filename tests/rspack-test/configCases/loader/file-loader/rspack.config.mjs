/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        use: [{ loader: 'file-loader', options: { esModule: false } }],
        type: 'javascript/auto',
      },
    ],
  },
};
