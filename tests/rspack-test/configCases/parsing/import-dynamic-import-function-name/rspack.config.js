/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
    importFunctionName: '__import__',
  },
  node: {
    __dirname: false,
  },
};
