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
            loader: "builtin:swc-loader",
            options: {
              jsc: {
                parser: {
                  syntax: "typescript",
                }
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
    concatenateModules: true,
  },
  experiments: {
    inlineEnum: true,
  }
}
