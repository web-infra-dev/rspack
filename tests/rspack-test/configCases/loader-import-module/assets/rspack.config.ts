import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
      {
        test: /index\.js/,
        loader: './loader.mjs',
      },
    ],
  },
});
