import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
