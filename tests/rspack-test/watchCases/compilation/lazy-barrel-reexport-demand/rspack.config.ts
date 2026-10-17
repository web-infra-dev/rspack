import { defineConfig } from '@rspack/cli';

export default defineConfig(
  [true, false].flatMap((cache) =>
    [true, false].map((incremental) => ({
      mode: 'development' as const,
      cache,
      incremental,
      output: {
        filename: `bundle-${cache}-${incremental}.js`,
      },
      optimization: {
        usedExports: false,
        sideEffects: 'flag' as const,
        concatenateModules: false,
      },
    })),
  ),
);
