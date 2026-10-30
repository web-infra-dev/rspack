import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default ([false, true, 'single'] as const).map((runtimeChunk, index) =>
  defineConfig({
    mode: 'production',
    target: 'node',
    entry: { main: './index.js', other: './other.js' },
    output: {
      filename: `[name]${index}.mjs`,
      chunkFilename: `[id]${index}.mjs`,
      library: { type: 'modern-module' },
    },
    optimization: { runtimeChunk, minimize: false },
    plugins: [
      new rspack.DefinePlugin({
        OTHER_BUNDLE: JSON.stringify(`./other${index}.mjs`),
        SINGLE_RUNTIME: JSON.stringify(runtimeChunk === 'single'),
      }),
    ],
  }),
);
