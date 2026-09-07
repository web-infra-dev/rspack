/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  optimization: {
    minimize: false,
  },
  output: {
    chunkFilename: '[name].js',
  },
  module: {
    rules: [
      {
        test: /lazy\.js$/,
        parser: {
          dynamicImportMode: 'lazy',
        },
      },
      {
        test: /lazy-once\.js$/,
        parser: {
          dynamicImportMode: 'lazy-once',
        },
      },
      {
        test: /eager\.js$/,
        parser: {
          dynamicImportMode: 'eager',
        },
      },
      {
        test: /weak\.js$/,
        parser: {
          dynamicImportMode: 'weak',
        },
      },
    ],
  },
};
