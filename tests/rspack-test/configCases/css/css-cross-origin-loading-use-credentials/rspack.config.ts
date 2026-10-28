import { defineConfig } from '@rspack/cli';

export default defineConfig({
  output: {
    crossOriginLoading: 'use-credentials',
  },
  entry: './index.js',
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/module',
      },
    ],
  },
});
