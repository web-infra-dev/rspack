import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new webpack.ContextReplacementPlugin(
      /context-replacement/,
      path.resolve(import.meta.dirname, 'modules'),
      {
        a: './module-b',
      },
    ),
  ],
});
