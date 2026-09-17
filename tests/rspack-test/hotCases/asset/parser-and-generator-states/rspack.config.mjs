/** @type {import("@rspack/core").Configuration} */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.(svg|png)$/,
        type: 'asset',
      },
    ],
  },
};
