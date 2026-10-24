import { defineConfig } from '@rspack/cli';
import { rspack, type RspackOptions } from '@rspack/core';

const configurations: RspackOptions[] = [
  { optimization: { runtimeChunk: false } },
  { optimization: { runtimeChunk: true } },
  {
    optimization: {
      runtimeChunk: { name: (entrypoint) => `runtime-${entrypoint.name}` },
    },
  },
  { optimization: { runtimeChunk: false, splitChunks: false } },
  { optimization: { runtimeChunk: 'single' } },
  { optimization: { runtimeChunk: false }, output: { asyncChunks: false } },
  { optimization: { runtimeChunk: false, minimize: true } },
  {
    optimization: {
      runtimeChunk: false,
      splitChunks: {
        cacheGroups: {
          shared: {
            test: /(?:shared\.cjs|marker\.js)$/,
            name: 'shared',
            enforce: true,
          },
        },
      },
    },
  },
];

export default configurations.map(({ optimization, output }, index) =>
  defineConfig({
    mode: 'production',
    target: 'node',
    entry: { main: './index.js', other: './other.js' },
    output: {
      filename: `[name]${index}.mjs`,
      chunkFilename: `[id]${index}.mjs`,
      library: { type: 'modern-module' },
      ...output,
    },
    optimization: { minimize: false, ...optimization },
    plugins: [
      new rspack.DefinePlugin({
        OTHER_BUNDLE: JSON.stringify(`./other${index}.mjs`),
        SINGLE_RUNTIME: JSON.stringify(optimization?.runtimeChunk === 'single'),
      }),
    ],
  }),
);
