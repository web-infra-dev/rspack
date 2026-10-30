import { defineConfig } from '@rspack/cli';
import { rspack, type RspackOptions } from '@rspack/core';

const configurations: RspackOptions[] = [
  { optimization: { runtimeChunk: false } },
  { optimization: { runtimeChunk: true } },
  { optimization: { runtimeChunk: 'single' } },
  { optimization: { runtimeChunk: false, minimize: true } },
];

export default configurations.map(({ optimization }, index) =>
  defineConfig({
    mode: 'production',
    target: 'node',
    entry: { main: './index.js', other: './other.js' },
    output: {
      filename: `[name]${index}.mjs`,
      chunkFilename: `[id]${index}.mjs`,
      library: { type: 'modern-module' },
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
