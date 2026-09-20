import { defineConfig } from '@rspack/cli';

import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        vendor: {
          chunks: 'all',
          name: 'vendor',
          test: /vendor/,
          enforce: true,
        },
      },
    },
  },
  plugins: [new HotModuleReplacementPlugin()],
});
