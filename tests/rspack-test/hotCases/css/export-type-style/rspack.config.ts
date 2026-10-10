import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  entry: ['./public-path.js', './index.js'],
  output: { assetModuleFilename: 'assets/[name][ext]' },
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      { test: /\.svg$/, type: 'asset/resource' },
      {
        test: /\.css$/,
        type: 'css/module',
        parser: {
          exportType: 'style',
          runtimePublicPath: true,
        },
      },
    ],
  },
  experiments: {
    css: true,
  },
});
