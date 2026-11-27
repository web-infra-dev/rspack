import { defineConfig } from '@rspack/cli';

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.svg$/,
        type: 'asset/resource',
      },
    ],
  },
  output: {
    assetModuleFilename: '[name].[contenthash:8][ext]',
  },
  optimization: {
    chunkIds: 'named',
    moduleIds: 'named',
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: {
          test: /logo\.svg$/,
          name: 'shared',
          minChunks: 2,
          enforce: true,
        },
      },
    },
  },
});
