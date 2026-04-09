module.exports = {
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.ts/,
        sideEffects: false,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              detectSyntax: 'auto',
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
    concatenateModules: true,
    inlineExports: true,
  },
};
