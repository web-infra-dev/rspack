/** @type {import("@rspack/core").Configuration} */
export default {
  resolve: {
    extensions: ['...', '.ts', '.tsx', '.jsx'],
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
          './loader.mjs',
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
