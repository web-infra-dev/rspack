/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    extensions: ['...', '.ts'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
};
