/** @type {import("@rspack/core").Configuration} */
module.exports = {
  resolve: {
    extensions: ['.ts', '...'],
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
              jsc: {
                target: 'esnext',
              },
              collectTypeScriptInfo: {
                exportedEnum: true,
              },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    moduleIds: 'named',
    concatenateModules: false,
    inlineExports: true,
  },
};
