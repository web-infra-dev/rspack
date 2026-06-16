/** @type {import("@rspack/core").Configuration} */
module.exports = {
  resolve: {
    extensions: ['...', '.ts'],
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          './assert-loader.js',
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
              collectTypeScriptInfo: {
                typeExports: true,
              },
            },
          },
        ],
        type: 'javascript/auto',
      },
    ],
  },
};
