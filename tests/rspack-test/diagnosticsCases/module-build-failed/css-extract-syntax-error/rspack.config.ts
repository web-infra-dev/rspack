import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
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
