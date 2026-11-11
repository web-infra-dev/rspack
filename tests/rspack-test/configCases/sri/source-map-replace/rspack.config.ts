import { defineConfig } from '@rspack/cli';
import { type Compiler, SubresourceIntegrityPlugin } from '@rspack/core';

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: 'source-map',
  externals: {
    crypto: "require('crypto')",
    fs: "require('fs')",
    path: "require('path')",
  },
  node: { __dirname: false },
  output: {
    chunkFilename: '[name].[contenthash:8].js',
    sourceMapFilename: '[file].[contenthash:8].map',
    crossOriginLoading: 'anonymous',
  },
  optimization: { minimize: false, realContentHash: true },
  plugins: [
    new SubresourceIntegrityPlugin({ hashFuncNames: ['sha256', 'sha384'] }),
    {
      apply(compiler: Compiler) {
        compiler.hooks.thisCompilation.tap('CheckMapAssets', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'CheckMapAssets',
              stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
            },
            () => {
              for (const asset of compilation.getAssets()) {
                if (!asset.name.endsWith('.map')) continue;
                // RCH has edited the map's file field. The outer edit wrapper
                // must still expose plain JSON with no mappings of its own.
                expect(asset.source.map()).toBeNull();
                const map = JSON.parse(asset.source.source().toString());
                expect(compilation.getAsset(map.file)).toBeDefined();
              }
            },
          );
        });
      },
    },
  ],
});
