import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'production',
  entry: './src/index.js',
  devtool: false,
  output: {
    filename: 'main.js',
    assetModuleFilename: '[name].[contenthash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(svg)$/,
        type: 'asset/resource',
      },
    ],
  },
  optimization: {
    realContentHash: true,
  },
});
