import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

class Plugin {
  apply(compiler: Compiler) {
    compiler.hooks.afterEmit.tap('Test', (compilation) => {
      const assets = compilation.getAssets();
      for (const asset of assets) {
        const sourceMap = asset.info.related?.sourceMap;
        if (sourceMap) {
          expect(sourceMap).toBe(`${asset.name}.map`);
        }
      }
    });
  }
}

export default defineConfig({
  context: import.meta.dirname,
  output: {
    filename: '[name].[contenthash].js',
  },
  devtool: 'source-map',
  plugins: [new Plugin()],
  module: {
    generator: {
      asset: {
        filename: 'assets/[name].[contenthash][ext]',
      },
    },
    rules: [
      {
        test: /file\.txt$/,
        type: 'asset/resource',
      },
    ],
  },
});
