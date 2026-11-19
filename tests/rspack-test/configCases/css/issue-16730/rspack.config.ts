import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  optimization: {
    chunkIds: 'named',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
        parser: {
          exportType: 'style',
        },
      },
    ],
  },
});
