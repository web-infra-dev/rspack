import { defineConfig } from '@rspack/cli';
import { rspack, type Compiler, type RspackOptions } from '@rspack/core';

const configurations: RspackOptions[] = [
  { optimization: { runtimeChunk: false } },
  { optimization: { runtimeChunk: true } },
  { optimization: { runtimeChunk: 'single' } },
];

export default configurations
  .flatMap((config) =>
    ([false, 'eval', 'eval-source-map'] as const).flatMap((devtool) =>
      [false, true].map((minimize) => ({
        devtool,
        optimization: { ...config.optimization, minimize },
      })),
    ),
  )
  .map(({ optimization, devtool }, index) =>
    defineConfig({
      mode: 'production',
      devtool,
      target: 'node',
      entry: { main: './index.js', other: './other.js' },
      output: {
        filename: `[name]${index}.mjs`,
        chunkFilename: `[id]${index}.mjs`,
        library: { type: 'modern-module' },
      },
      optimization,
      plugins: [
        {
          apply(compiler: Compiler) {
            compiler.hooks.afterPlugins.tap('AssertDevtoolFallback', () => {
              expect(compiler.options.devtool).toBe(
                devtool === 'eval'
                  ? false
                  : devtool === 'eval-source-map'
                    ? 'source-map'
                    : devtool,
              );
            });
          },
        },
        new rspack.DefinePlugin({
          OTHER_BUNDLE: JSON.stringify(`./other${index}.mjs`),
          SINGLE_RUNTIME: JSON.stringify(
            optimization?.runtimeChunk === 'single',
          ),
        }),
      ],
    }),
  );
