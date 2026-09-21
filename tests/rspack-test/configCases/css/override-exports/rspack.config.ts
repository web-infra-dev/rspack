import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  target: 'web',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
