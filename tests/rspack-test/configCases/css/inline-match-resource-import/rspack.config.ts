import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  output: {
    pathinfo: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css',
      },
    ],
  },
});
