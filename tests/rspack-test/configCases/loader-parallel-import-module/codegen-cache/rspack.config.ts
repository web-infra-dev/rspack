import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    rules: [
      {
        test: /app-proxy\.js/,
        use: [
          {
            loader: './loader.mjs',
            options: {},
            parallel: true,
          },
        ],
      },
    ],
  },
});
