import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
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
      // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
      FREE_VERSION: DefinePlugin.runtimeValue(
        (ctx: { module: { layer: string | null } }) =>
          ctx.module.layer === 'free',
      ),
    }),
  ],
});
