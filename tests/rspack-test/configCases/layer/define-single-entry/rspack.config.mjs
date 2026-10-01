import { DefinePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    paid: { import: './main.js', layer: 'paid' },
    free: { import: './main.js', layer: 'free' },
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        layerPaidCommon: {
          name: 'layer-paid-common',
          layer: 'paid',
          chunks: 'async',
          enforce: true,
          reuseExistingChunk: true,
        },
      },
    },
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new DefinePlugin({
      FREE_VERSION: DefinePlugin.runtimeValue(
        (ctx) => ctx.module.layer === 'free',
      ),
    }),
  ],
};
