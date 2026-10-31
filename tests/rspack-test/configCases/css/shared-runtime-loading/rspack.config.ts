import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'web',
  mode: 'development',
  devtool: false,
  entry: {
    main: './index.js',
    other: './feature.js',
  },
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
    chunkFilename: '[name].js',
    cssFilename: '[name].css',
    cssChunkFilename: '[name].css',
    publicPath: '',
  },
  optimization: {
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        styles: {
          test: /\.css$/,
          name: 'shared-css',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
});
