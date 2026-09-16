/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        resourceQuery: /^\?loader/,
        use: './loader?query',
      },
    ],
  },
};
