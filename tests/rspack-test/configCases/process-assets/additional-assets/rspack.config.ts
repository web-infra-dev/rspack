import { defineConfig } from '@rspack/cli';
import type { sources, Compiler } from '@rspack/core';

const newName = 'file.txt';

export default defineConfig({
  output: {
    assetModuleFilename: 'images/[name][ext]',
  },
  plugins: [
    {
      apply: (compiler: Compiler) => {
        const PLUGIN_NAME = 'TestPlugin';

        compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: PLUGIN_NAME,
              // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
              additionalAssets: (assets: Record<string, sources.Source>) => {
                for (const name of Object.keys(assets)) {
                  if (newName !== name) {
                    continue;
                  }
                  compilation.updateAsset(name, assets[name], {
                    additional: true,
                  });
                }
              },
            },
            () => {
              compilation.emitAsset(
                newName,
                new compiler.rspack.sources.RawSource('text'),
                {
                  new: true,
                },
              );
            },
          );
        });
      },
    },
    {
      apply: (compiler: Compiler) => {
        const PLUGIN_NAME = 'TestPlugin1';

        compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: PLUGIN_NAME,
              // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
              additionalAssets: false,
            },
            () => {
              compilation.emitAsset(
                'file1.txt',
                new compiler.rspack.sources.RawSource('text'),
                {
                  new: true,
                },
              );
            },
          );
        });
      },
    },
    {
      apply: (compiler: Compiler) => {
        const PLUGIN_NAME = 'TestPlugin2';

        compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: PLUGIN_NAME,
              // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
              additionalAssets: true,
            },
            (assets) => {
              compilation.updateAsset(newName, assets[newName], {
                additionalAgain: true,
              });
            },
          );
        });
      },
    },
  ],
});
