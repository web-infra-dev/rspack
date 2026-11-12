import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.js/,
        use: [
          {
            loader: './loader.mjs',
            options: {},
            parallel: true,
          },
        ],
        issuerLayer: 'main',
      },
    ],
  },
});
