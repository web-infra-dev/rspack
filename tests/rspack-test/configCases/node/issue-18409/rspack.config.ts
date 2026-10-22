import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: false,
  mode: 'development',
  entry: {
    main: {
      import: './index.js',
      dependOn: 'shared',
    },
    shared: './common.js',
  },
  output: {
    filename: '[name].js',
  },
  target: ['node'],
  optimization: {
    minimize: false,
    runtimeChunk: 'single',

    splitChunks: {
      cacheGroups: {
        separate: {
          test: /separate/,
          chunks: 'all',
          enforce: true,
          filename: '[name].[contenthash].js',
        },
      },
    },
  },
});
