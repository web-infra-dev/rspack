import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';
export default defineConfig(
  [false, true].flatMap((cache) =>
    [false, true].flatMap((parallel) =>
      [false, true].map((mixed) => ({
        mode: 'development',
        output: {
          filename: `bundle-${cache}-${parallel}-${mixed}.js`,
        },
        incremental: false,
        cache: cache ? { type: 'memory' } : false,
        experiments: {
          newCache: {
            codeGeneration: false,
            loader: cache,
            minimize: false,
          },
        },
        module: {
          rules: [
            {
              test: /input\.txt$/,
              type: 'javascript/auto',
              use: [
                {
                  loader: fileURLToPath(
                    import.meta.resolve('./consumer-loader.mjs'),
                  ),
                  options: { name: `${cache}-${parallel}-${mixed}` },
                  cache,
                  parallel: parallel ? { maxWorkers: 1 } : false,
                },
                ...(mixed
                  ? [{ loader: 'builtin:test-passthrough-loader', cache }]
                  : []),
                {
                  loader: fileURLToPath(
                    import.meta.resolve('./producer-loader.mjs'),
                  ),
                },
              ],
            },
          ],
        },
        plugins: [new rspack.DefinePlugin({ LOADER_CACHE_ENABLED: cache })],
      })),
    ),
  ),
);
