import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    publicPath: '/public/',
  },
  entry: './entry.js',
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
      },
    ],
  },
});
