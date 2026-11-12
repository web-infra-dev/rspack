import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.js$/,
        loader: 'builtin:swc-loader',
        options: {
          transformImport: [
            {
              libraryName: './lib',
              customName: './lib/{{ member }',
            },
          ],
        },
      },
    ],
  },
});
