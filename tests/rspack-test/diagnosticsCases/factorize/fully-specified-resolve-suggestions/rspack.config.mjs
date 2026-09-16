/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        test: /\.js$/,
        resolve: {
          fullySpecified: true,
        },
        type: 'javascript/esm',
      },
    ],
  },
};
