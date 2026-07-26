/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'production',
  optimization: {
    minimize: false,
  },
  module: {
    rules: [
      {
        test: /loaders\.js$/,
        loader: 'builtin:swc-loader',
        options: {
          jsc: {
            externalHelpers: true,
            target: 'es2015',
          },
        },
        type: 'javascript/auto',
      },
    ],
  },
};
