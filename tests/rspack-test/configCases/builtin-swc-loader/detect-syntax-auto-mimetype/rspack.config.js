/** @type {import("@rspack/core").Configuration} */
module.exports = {
  mode: 'development',
  resolve: {
    extensions: ['...', '.ts', '.tsx'],
  },
  module: {
    rules: [
      {
        mimetype: {
          or: ['text/javascript', 'application/javascript'],
        },
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              jsc: {
                externalHelpers: true,
                parser: {
                  decorators: true,
                },
                transform: {
                  react: {
                    runtime: 'automatic',
                  },
                },
              },
            },
          },
        ],
      },
    ],
  },
};
