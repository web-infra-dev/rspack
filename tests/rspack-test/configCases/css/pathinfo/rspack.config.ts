import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  output: {
    pathinfo: true,
    cssChunkFilename: '[name].[chunkhash].css',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
});
