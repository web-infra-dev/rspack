import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkFilename: '[id].[contenthash].js',
  },
  module: {
    rules: [
      {
        test: /\.css/,
        type: 'css/auto',
      },
    ],
  },
});
