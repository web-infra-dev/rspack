import { DefinePlugin } from '@rspack/core';

export default ['link', 'text', 'css-style-sheet', 'style'].flatMap(
  (exportType) =>
    [false, true].flatMap((concatenateModules) =>
      [undefined, 'styles'].map((layer) => ({
        mode: 'development',
        target: 'web',
        devtool: false,
        experiments: { css: true },
        optimization: { concatenateModules },
        module: {
          rules: [{ test: /\.css$/, type: 'css/module', layer }],
          parser: { css: { exportType } },
        },
        plugins: [
          new DefinePlugin({
            'process.env.EXPORT_TYPE': JSON.stringify(exportType),
          }),
        ],
      })),
    ),
);
