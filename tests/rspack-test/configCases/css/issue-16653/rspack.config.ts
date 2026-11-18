import { defineConfig } from '@rspack/cli';

export default defineConfig(
  ([undefined, 'link', 'style', 'text', 'css-style-sheet'] as const).flatMap(
    (exportType) =>
      [false, true].flatMap((concatenateModules) =>
        (['asset/inline', 'asset/resource'] as const).map((type) => ({
          mode: 'production',
          target: 'web',
          devtool: false,
          output: {
            publicPath: '',
            assetModuleFilename: '[name][ext]',
            cssFilename: `${exportType ?? 'default'}-${concatenateModules}-${type.replace('/', '-')}.css`,
          },
          optimization: { concatenateModules, minimize: false },
          module: {
            rules: [
              {
                test: /\.css$/,
                type: 'css',
                parser: { exportType },
              },
              { test: /\.svg$/, type },
            ],
          },
        })),
      ),
  ),
);
