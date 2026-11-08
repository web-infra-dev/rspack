import { defineConfig } from '@rspack/cli';

export default defineConfig({
  externals: {
    path: 'node-commonjs path',
  },
  target: 'web',
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].chunk.js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendors: {
          name: 'vendors',
          test: /node_modules/,
          enforce: true,
        },
      },
    },
  },
});
