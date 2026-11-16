import { defineConfig } from '@rspack/cli';
import path from 'node:path';

const devtools = [
  false,
  'source-map',
  'hidden-source-map',
  'nosources-source-map',
  'hidden-nosources-source-map',
  'inline-source-map',
  'inline-nosources-source-map',
] as const;

export default defineConfig(
  devtools.flatMap((devtool) =>
    (['text', 'style', 'css-style-sheet'] as const).flatMap((exportType) =>
      [false, true].map((useLess) => ({
        mode: 'production',
        target: 'web',
        devtool,
        optimization: { minimize: false },
        resolve: {
          alias: {
            STYLE_UNDER_TEST$: path.resolve(
              import.meta.dirname,
              useLess ? 'style.less' : 'style.css',
            ),
          },
        },
        module: {
          rules: [
            {
              test: /\.(css|less)$/,
              type: 'css/auto',
              parser: { exportType },
              use: useLess
                ? [{ loader: 'less-loader', options: { sourceMap: true } }]
                : [],
            },
          ],
        },
      })),
    ),
  ),
);
