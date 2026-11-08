import { defineConfig } from '@rspack/cli';

export default defineConfig(
  (['compact-hashed', 'deterministic'] as const).map((chunkIds) =>
    defineConfig({
      name: chunkIds,
      output: {
        filename: `${chunkIds}/[name].mjs`,
        chunkFilename: `${chunkIds}/[name].mjs`,
      },
      optimization: {
        minimize: false,
        moduleIds: 'named',
        chunkIds,
        removeEmptyChunks: true,
        runtimeChunk: 'single',
        splitChunks: {
          cacheGroups: {
            workers: {
              test: /(worker|async)\.js/,
              name: 'workers',
              enforce: true,
            },
          },
        },
      },
    }),
  ),
);
