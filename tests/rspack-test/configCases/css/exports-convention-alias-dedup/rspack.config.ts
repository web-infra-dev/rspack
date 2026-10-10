import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['camel-case', 'camel-case-only'] as const).flatMap((exportsConvention) =>
    [false, true].map((concatenateModules) =>
      defineConfig({
        mode: 'production',
        target: 'node',
        optimization: { concatenateModules, minimize: false },
        module: {
          rules: [
            {
              test: /\.module\.css$/,
              type: 'css/module',
              generator: {
                exportsConvention,
                exportsOnly: true,
                localIdentName: '[local]',
              },
            },
          ],
        },
        experiments: { css: true },
      }),
    ),
  ),
);
