import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { Compilation, sources } from '@rspack/core';

export default defineConfig({
  plugins: [
    (compiler: Compiler) => {
      compiler.hooks.compilation.tap(
        'MockWebpackManifestPlugin',
        (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'MockWebpackManifestPlugin',
              stage: Infinity,
            },
            () => {
              const manifest = {
                'main.js': '/main.js',
                'main.js.map': '/main.js.map',
              };
              compilation.emitAsset(
                'asset-manifest.json',
                new sources.RawSource(JSON.stringify(manifest, null, 2)),
              );
            },
          );
        },
      );
    },
    (compiler: Compiler) => {
      compiler.hooks.compilation.tap('MockCompressionPlugin', (compilation) => {
        compilation.hooks.processAssets.tapPromise(
          {
            name: 'MockCompressionPlugin',
            stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER,
            // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
            additionalAssets: true,
          },
          async (assets) => {
            for (const name of Object.keys(assets)) {
              const { info } = compilation.getAsset(name)!;
              if (info && name === 'asset-manifest.json') {
                compilation.deleteAsset(name);
              }
            }
          },
        );
      });
    },
    (compiler: Compiler) => {
      compiler.hooks.compilation.tap('MockTerserPlugin', (compilation) => {
        compilation.hooks.processAssets.tapPromise(
          {
            name: 'MockTerserPlugin',
            stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE,
            // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
            additionalAssets: true,
          },
          async (assets) => {
            for (const name of Object.keys(assets)) {
              const { info } = compilation.getAsset(name)!;
              if (info && name === 'asset-manifest.json') {
                compilation.deleteAsset(name);
              }
            }
          },
        );
      });
    },
  ],
});
