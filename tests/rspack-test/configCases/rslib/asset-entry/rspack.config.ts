import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  entry: './index.png',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset/resource',
      },
    ],
  },
  plugins: [
    new rspack.experiments.RslibPlugin(),
    definePlugin((compiler) => {
      compiler.hooks.done.tap('test case', (stats) => {
        const asset = stats.compilation.getAsset('bundle0.js');
        expect(asset).toBeDefined();
      });
    }),
  ],
});
