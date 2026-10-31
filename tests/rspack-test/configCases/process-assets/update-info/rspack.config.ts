import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

export default defineConfig({
  output: {
    assetModuleFilename: 'images/[name][ext]',
  },
  plugins: [
    {
      apply: (compiler: Compiler) => {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'TestPlugin',
              // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
              additionalAssets: true,
            },
            (assets) => {
              for (const asset of Object.keys(assets)) {
                switch (asset) {
                  case 'images/file.svg': {
                    compilation.updateAsset(asset, assets[asset], {
                      custom: true,
                      related: {
                        // @ts-expect-error Custom related asset keys are not exposed in Rspack types yet.
                        first: ['first'],
                      },
                    });
                    compilation.updateAsset(asset, assets[asset], (info) => ({
                      ...info,
                      related: {
                        ...info.related,
                        second: ['second'],
                      },
                      customFn: true,
                    }));
                    break;
                  }
                  case 'images/file1.svg': {
                    compilation.updateAsset(asset, assets[asset], {
                      custom: true,
                    });
                    compilation.updateAsset(
                      asset,
                      assets[asset],
                      () => undefined,
                    );
                    break;
                  }
                }
              }
            },
          );
        });
      },
    },
  ],
});
