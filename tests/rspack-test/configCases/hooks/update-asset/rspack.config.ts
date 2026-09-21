import assert from 'node:assert/strict';
import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  output: {
    filename: '[name].[contenthash].js',
  },
  plugins: [
    definePlugin(function plugin(compiler) {
      compiler.hooks.compilation.tap('test', (compilation) => {
        compilation.hooks.processAssets.tap(
          {
            name: 'test',
            stage: compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
          },
          (assets) => {
            Object.entries(assets).forEach(([filename, asset]) => {
              const newContent = `// UPDATED\n${asset.source()}`;
              compilation.updateAsset(
                filename,
                new compiler.rspack.sources.RawSource(newContent),
              );
            });
          },
        );
        compilation.hooks.processAssets.tap(
          {
            name: 'test',
            stage:
              compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH,
          },
          () => {
            compilation.getAssets().forEach(({ info }) => {
              assert(info.contenthash);
              expect(info.contenthash.length).toBeGreaterThan(0);
            });
          },
        );
      });
    }),
  ],
});
