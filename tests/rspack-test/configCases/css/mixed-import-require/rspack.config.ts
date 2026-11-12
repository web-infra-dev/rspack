import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  entry: './index.js',
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  target: 'web',
});
