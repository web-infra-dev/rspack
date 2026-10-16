import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  target: 'web',
  node: {
    __dirname: false,
  },
  externals: {
    fs: 'node-commonjs fs',
  },
  output: {
    publicPath: () => '../',
    assetModuleFilename: 'assets/[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [new CssExtractRspackPlugin({ filename: 'style.css' })],
});
