import { defineConfig } from '@rspack/cli';
import { type Compiler, rspack } from '@rspack/core';

const { RawSource } = rspack.sources;

export default defineConfig({
  plugins: [
    new (class {
      banner: string;
      name: string;

      constructor(banner: string) {
        this.banner = banner;
        this.name = 'BannerPlugin';
      }
      apply(compiler: Compiler) {
        const banner = this.banner;
        compiler.hooks.compilation.tap('BannerPlugin', (compilation) => {
          compilation.hooks.processAssets.tap(
            {
              name: 'BannerPlugin',
              // ProcessAssetsStageAdditions
              stage: -100,
            },
            (assets) => {
              for (const file of Object.keys(assets)) {
                compilation.updateAsset(file, (old) => {
                  const newContent = `${banner}\n${old.source().toString()}`;
                  return new RawSource(newContent);
                });
              }
            },
          );
        });
      }
    })('/** MMMMM */'),
  ],
});
