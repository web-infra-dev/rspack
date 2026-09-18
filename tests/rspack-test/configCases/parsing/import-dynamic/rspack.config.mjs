/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: './index.js',
    test: './test.js',
  },
  module: {
    parser: {
      javascript: {
        importDynamic: false,
      },
    },
  },
  output: {
    filename: '[name].js',
  },
  node: {
    __dirname: false,
  },
};
