/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
    ],
    parser: {
      asset: {
        dataUrlCondition: {
          maxSize: 100 * 1024,
        },
      },
    },
  },
};
