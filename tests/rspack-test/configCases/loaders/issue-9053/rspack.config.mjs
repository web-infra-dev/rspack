/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /c\.js$/,
        use: ['loader2'],
      },
      {
        test: /d\.js$/,
        use: ['loader3'],
      },
    ],
  },
};
