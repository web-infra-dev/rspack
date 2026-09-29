export default ['link', 'text', 'css-style-sheet', 'style'].flatMap(
  (exportType) =>
    [false, true].map((concatenateModules) => ({
      target: 'web',
      mode: 'development',
      devtool: false,
      optimization: { concatenateModules },
      experiments: { css: true },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'css/module',
            generator: { localIdentName: '[local]' },
            oneOf: [
              {
                resourceQuery: /camel-case-only/,
                generator: { exportsConvention: 'camel-case-only' },
              },
              {
                resourceQuery: /camel-case/,
                generator: { exportsConvention: 'camel-case' },
              },
              { generator: { exportsConvention: 'as-is' } },
            ],
          },
        ],
        parser: { css: { exportType } },
      },
    })),
);
