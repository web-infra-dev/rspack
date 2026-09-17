/** @type {import("@rspack/core").Configuration} */
export default {
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
