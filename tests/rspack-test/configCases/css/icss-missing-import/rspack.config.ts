import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['development', 'production'] as const).flatMap((mode) =>
    [false, true].map((exportsOnly) =>
      defineConfig({
        mode,
        target: 'web',
        module: {
          rules: [
            { test: /\.css$/, type: 'css/module', generator: { exportsOnly } },
          ],
        },
        optimization: { minimize: false },
        experiments: { css: true },
      }),
    ),
  ),
);
