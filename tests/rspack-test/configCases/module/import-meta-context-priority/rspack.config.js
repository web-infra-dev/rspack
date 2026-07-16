/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  devtool: false,
  target: 'node',
  experiments: {
    outputModule: true,
  },
  output: {
    module: true,
    chunkFormat: 'module',
  },
  module: {
    rules: [
      {
        test: /context-field-enabled\.js$/,
        parser: {
          importMeta: {
            webpackContext: true,
          },
        },
      },
      {
        test: /context-field-disabled\.js$/,
        parser: {
          importMeta: {
            webpackContext: false,
          },
        },
      },
    ],
  },
};
