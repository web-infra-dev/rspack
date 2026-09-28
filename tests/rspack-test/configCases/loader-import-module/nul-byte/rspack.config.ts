import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    rules: [
      {
        test: /a.js/,
        loader: './convert-loader.mjs',
      },
    ],
  },
});
