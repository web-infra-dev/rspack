import { defineConfig } from '@rspack/cli';
import { CssExtractRspackPlugin } from '@rspack/core';

export default defineConfig({
  target: 'web',
  experiments: {
    css: false,
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    minimize: false,
    // Move the extracted css into a chunk named "0" that holds no JavaScript,
    // while the JavaScript chunk is named "00", which coerces to the same number.
    splitChunks: {
      chunks: 'async',
      cacheGroups: {
        styles: {
          type: 'css/mini-extract',
          name: '0',
          chunks: 'async',
          enforce: true,
        },
      },
    },
  },
  plugins: [new CssExtractRspackPlugin()],
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
});
