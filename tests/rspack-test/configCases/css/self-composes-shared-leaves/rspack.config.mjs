export default ['link', 'text', 'css-style-sheet', 'style'].flatMap(
  (exportType) =>
    [false, true].map((concatenateModules) => ({
      target: 'web',
      mode: 'development',
      devtool: false,
      experiments: { css: true },
      optimization: { concatenateModules },
      module: {
        rules: [
          {
            test: /\.css$/,
            type: 'css/module',
            generator: { localIdentName: '[local]' },
          },
        ],
        parser: { css: { exportType } },
      },
    })),
);
