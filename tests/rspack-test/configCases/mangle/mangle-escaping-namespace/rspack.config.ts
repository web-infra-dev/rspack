import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig(
  [false, true].flatMap((concatenateModules) =>
    ([false, true, 'size'] as const).map((mangleExports) => ({
      mode: 'production',
      target: 'node',
      module: { parser: { javascript: { exportsPresence: false } } },
      optimization: {
        usedExports: true,
        providedExports: true,
        inlineExports: true,
        mangleExports,
        concatenateModules,
        minimize: false,
      },
      plugins: [
        new rspack.DefinePlugin({
          EXPECT_MANGLE: JSON.stringify(mangleExports !== false),
        }),
      ],
    })),
  ),
);
