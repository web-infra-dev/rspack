import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  cache: false,
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  node: { __dirname: false },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [new CssExtractRspackPlugin()],
});
