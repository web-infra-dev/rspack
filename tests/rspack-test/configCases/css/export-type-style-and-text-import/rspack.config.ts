import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  output: { publicPath: 'auto' },
  module: {
    rules: [
      {
        test: /wrapper-style\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'style',
        },
      },
      {
        test: /base\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'text',
        },
      },
    ],
  },
  experiments: {
    css: true,
  },
});
