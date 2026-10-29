import { defineConfig } from '@rspack/cli';
import type { LessLoaderOptions } from '@rspack/core';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          {
            loader: 'builtin:less-loader',
            options: {
              additionalData: '@injected: 3px;',
              lessOptions: {
                math: 'always',
                paths: ['./includes'],
                globalVars: { scale: 4 },
                modifyVars: { color: 'green' },
              },
            } satisfies LessLoaderOptions,
          },
        ],
      },
    ],
  },
});
