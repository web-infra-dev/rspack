import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  optimization: {
    realContentHash: true,
  },
  module: {
    generator: {
      asset: {
        filename: 'assets/[name].[contenthash][ext]',
      },
    },
    rules: [
      {
        test: /file\.text$/,
        type: 'asset/resource',
      },
    ],
  },
});
