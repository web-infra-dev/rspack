import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';

let firstChunkAsset: string | null = null;

class CheckAssetPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap('TestPlugin', (compilation) => {
      compilation.hooks.afterProcessAssets.tap('TestPlugin', (assets) => {
        const chunkAsset = Object.keys(assets).find((asset) =>
          asset.startsWith('chunk.'),
        );
        if (!chunkAsset) {
          throw new Error('chunk asset not found');
        }
        if (firstChunkAsset === null) {
          firstChunkAsset = chunkAsset;
        } else {
          expect(firstChunkAsset).not.toEqual(chunkAsset);
        }
      });
    });
  }
}

export default defineConfig([
  {
    mode: 'production',
    entry: './main.js',
    output: {
      chunkFilename: 'chunk.[chunkhash].js',
    },
    optimization: {
      concatenateModules: true,
      minimize: false,
    },
    plugins: [new CheckAssetPlugin()],
  },
  {
    mode: 'production',
    entry: './main.js',
    output: {
      chunkFilename: 'chunk.[chunkhash].js',
    },
    optimization: {
      concatenateModules: true,
      minimize: false,
    },
    plugins: [new CheckAssetPlugin()],
  },
]);
