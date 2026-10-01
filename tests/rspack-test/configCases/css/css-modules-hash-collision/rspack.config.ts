import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.module\.css$/i,
        type: 'css/module',
      },
    ],
  },
  experiments: {
    css: true,
  },
});
