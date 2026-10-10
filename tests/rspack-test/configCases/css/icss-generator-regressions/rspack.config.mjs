import { DefinePlugin } from '@rspack/core';

const modes = ['link', 'text', 'css-style-sheet', 'style'].map(
  (exportType) => ({
    exportType,
    exportsOnly: false,
  }),
);
modes.push({ exportType: 'link', exportsOnly: true });

export default modes.flatMap(({ exportType, exportsOnly }) =>
  [false, true].map((concatenateModules) => ({
    target: 'web',
    mode: 'development',
    devtool: false,
    optimization: { concatenateModules },
    experiments: { css: true },
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    node: { __dirname: false, __filename: false },
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/module',
          parser: { dashedIdents: true },
          generator: { localIdentName: '[local]', exportsOnly },
        },
      ],
      parser: { css: { exportType } },
    },
    plugins: [
      new DefinePlugin({
        'process.env.EXPORT_TYPE': JSON.stringify(exportType),
        'process.env.EXPORTS_ONLY': JSON.stringify(exportsOnly),
      }),
    ],
  })),
);
