import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: {
    a: './index.js',
    b: './b.js',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        shared: {
          test: /[\\/]shared[\\/]/,
          name: 'shared',
          minChunks: 2,
          enforce: true,
        },
      },
    },
  },
});
