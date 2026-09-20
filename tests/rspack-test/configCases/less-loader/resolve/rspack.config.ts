import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.less$/,
        use: [
          {
            loader: 'less-loader',
            options: {
              lessOptions: {
                paths: [
                  'node_modules',
                  path.resolve(import.meta.dirname, 'node_modules'),
                ],
              },
            },
          },
        ],
        type: 'css',
      },
    ],
  },
});
