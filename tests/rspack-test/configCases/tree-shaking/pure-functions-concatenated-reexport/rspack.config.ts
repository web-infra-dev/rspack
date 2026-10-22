import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig(
  [false, true].flatMap((keepBase) =>
    [true, false].map((inlineExports) => ({
      mode: 'production',
      target: 'node',
      optimization: {
        minimize: false,
        inlineExports,
      },
      plugins: [new DefinePlugin({ KEEP_BASE: keepBase })],
    })),
  ),
);
