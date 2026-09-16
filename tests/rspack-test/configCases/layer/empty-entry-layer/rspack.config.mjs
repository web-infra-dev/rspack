/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        issuerLayer: 'dark',
        resolve: {
          conditionNames: ['dark', '...'],
        },
      },
    ],
  },
};
