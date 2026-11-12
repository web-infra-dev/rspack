import { defineConfig } from '@rspack/cli';
import { HotModuleReplacementPlugin } from '@rspack/core';

export default defineConfig({
  entry: {
    first: ['./shared', './first'],
    second: ['./shared', './second'],
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
          minChunks: 2,
          enforce: true,
        },
      },
    },
  },
  plugins: [new HotModuleReplacementPlugin()],
});
