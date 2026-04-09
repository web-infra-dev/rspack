/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
            },
            ident: 'builtin-swc-loader',
          },
        ],
      },
    ],
  },
};
