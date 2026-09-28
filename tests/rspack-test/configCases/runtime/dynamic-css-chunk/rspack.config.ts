import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    chunkFilename: '[id].[hash].js',
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
