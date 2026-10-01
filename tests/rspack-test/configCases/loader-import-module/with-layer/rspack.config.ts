import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.js/,
        loader: './loader.mjs',
        issuerLayer: 'main',
        options: {},
      },
    ],
  },
});
