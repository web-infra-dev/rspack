import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: {
    base: './base.js',
    parent: { import: './parent.js', dependOn: 'base' },
    sibling: { import: './sibling.js', dependOn: 'base' },
    main: { import: './index.js', dependOn: ['parent', 'sibling'] },
    other: './other.js',
  },
  output: { filename: '[name].js' },
  optimization: {
    runtimeChunk: 'single',
    chunkIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: { test: /shared\.css$/, name: 'shared', minChunks: 2 },
      },
    },
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'javascript/auto',
        use: [rspack.CssExtractRspackPlugin.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: '[name].[contenthash].css',
    }),
  ],
});
