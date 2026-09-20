import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  plugins: [new rspack.CssExtractRspackPlugin()],
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/inline',
        generator: {
          dataUrl() {
            return 'data:image/png;base64,custom-content';
          },
        },
      },
      {
        test: /\.css/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  experiments: {
    css: false,
  },
});
