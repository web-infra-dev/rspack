/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    bundle0: './index.js',
    bundle1: './other.js',
  },
  module: {
    parser: {
      javascript: {
        commonjsMagicComments: true,
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
