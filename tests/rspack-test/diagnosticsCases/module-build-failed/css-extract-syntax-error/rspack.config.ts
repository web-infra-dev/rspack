import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  experiments: {
    css: false,
  },
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
