/** @type {import("@rspack/core").Configuration} */
export default {
  entry: './index.js',
  module: {
    rules: [
      {
        issuer: /dark/,
        resolve: {
          conditionNames: ['dark', '...'],
        },
      },
    ],
  },
};
