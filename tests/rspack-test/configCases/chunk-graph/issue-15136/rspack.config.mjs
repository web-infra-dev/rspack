/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/esm',
        use: 'builtin:swc-loader',
      },
    ],
  },
};
