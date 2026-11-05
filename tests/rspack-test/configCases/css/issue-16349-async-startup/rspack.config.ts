import { container } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

const { ModuleFederationPlugin } = container;

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: false,
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
    cssFilename: '[name].[contenthash].css',
    publicPath: 'https://test.cases/path/',
  },
  module: {
    rules: [{ test: /\.css$/, type: 'css' }],
  },
  optimization: {
    chunkIds: 'named',
    minimize: false,
    runtimeChunk: 'single',
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        initial: {
          test: /initial\.css$/,
          name: 'initial-style',
          enforce: true,
        },
      },
    },
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'container',
      filename: 'remoteEntry.js',
      exposes: {
        './exposed': './exposed.js',
      },
      experiments: {
        asyncStartup: true,
      },
    }),
  ],
});
