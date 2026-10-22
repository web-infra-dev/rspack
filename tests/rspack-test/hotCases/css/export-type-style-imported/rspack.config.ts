import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /.css$/,
        type: 'css/module',
        parser: {
          exportType: 'style',
        },
      },
    ],
  },
  experiments: {
    css: true,
  },
});
