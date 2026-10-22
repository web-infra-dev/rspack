import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig(
  ([undefined, 'link', 'text', 'style', 'css-style-sheet'] as const).flatMap(
    (exportType) =>
      [false, true].map((concatenateModules) =>
        defineConfig({
          target: 'web',
          mode: 'development',
          devtool: false,
          experiments: { css: true },
          optimization: { concatenateModules, minimize: false },
          module: {
            parser: { css: { exportType } },
            rules: [
              {
                test: /\.css$/,
                type: 'css/module',
                generator: { localIdentName: '[local]' },
              },
              {
                test: /target.module.css$/,
                type: 'css/module',
                parser: { exportType: 'link' },
              },
            ],
          },
          plugins: [
            new DefinePlugin({
              EXPORT_TYPE: JSON.stringify(exportType ?? 'link'),
            }),
          ],
        }),
      ),
  ),
);
