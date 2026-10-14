import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig(
  [false, true].flatMap((cache) =>
    [false, true].map((incremental) => ({
      mode: 'development',
      output: {
        filename: `bundle-${cache}-${incremental}.js`,
      },
      incremental,
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
            test: /input\.js$/,
            use: [
              {
                loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
                codeFilter: 'transform:marker',
                cache,
              },
            ],
          },
        ],
      },
    })),
  ),
);
