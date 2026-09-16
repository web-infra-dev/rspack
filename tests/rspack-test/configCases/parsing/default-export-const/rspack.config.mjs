/** @type {import("@rspack/core").Configuration} */
export default [
  {
    entry: './not-supports-const.js',
    output: {
      environment: {
        const: false,
      },
    },
    optimization: {
      // Avoid the default export being inlined
      inlineExports: false,
    },
  },
  {
    entry: './supports-const.js',
    output: {
      environment: {
        const: true,
      },
    },
    optimization: {
      // Avoid the default export being inlined
      inlineExports: false,
    },
  },
];
