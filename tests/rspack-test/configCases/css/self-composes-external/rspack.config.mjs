import { DefinePlugin } from '@rspack/core';

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
          },
          { test: /\.js$/, type: 'javascript/auto' },
        ],
        parser: { css: { exportType } },
      },
      plugins: [
        new DefinePlugin({
          'process.env.EXPORT_TYPE': JSON.stringify(exportType),
        }),
      ],
    })),
);
