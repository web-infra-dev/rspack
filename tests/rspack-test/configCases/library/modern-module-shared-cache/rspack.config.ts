import { defineConfig } from '@rspack/cli';
import { rspack, type RspackOptions } from '@rspack/core';

const configurations: RspackOptions[] = [
  { optimization: { runtimeChunk: false } },
  { optimization: { runtimeChunk: true } },
  { optimization: { runtimeChunk: 'single' } },
];

export default configurations
  .flatMap((config) =>
    [false, true].map((minimize) => ({
      optimization: { ...config.optimization, minimize },
    })),
  )
  .map(({ optimization }, index) =>
    defineConfig({
      mode: 'production',
      target: 'node',
      entry: { main: './index.js', other: './other.js' },
      output: {
        filename: `[name]${index}.mjs`,
        chunkFilename: `[id]${index}.mjs`,
        library: { type: 'modern-module' },
      },
      optimization,
      plugins: [
        new rspack.DefinePlugin({
          OTHER_BUNDLE: JSON.stringify(`./other${index}.mjs`),
          SINGLE_RUNTIME: JSON.stringify(
            optimization?.runtimeChunk === 'single',
          ),
        }),
      ],
    }),
  );
