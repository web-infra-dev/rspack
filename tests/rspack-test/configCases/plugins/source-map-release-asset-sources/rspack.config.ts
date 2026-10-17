import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'node',
  plugins: [
    new rspack.SourceMapDevToolPlugin({
      filename: '[file].map',
    }),
    definePlugin((compiler) => {
      let expectedSource: string | undefined;
      let assetFilename: string | undefined;

      compiler.hooks.compilation.tap('PLUGIN', (compilation) => {
        compilation.hooks.afterProcessAssets.tap('PLUGIN', () => {
          assetFilename = compilation
            .getAssets()
            .map(asset => asset.name)
            .find(name => name.endsWith('.js') && !name.endsWith('.map'));
          expect(assetFilename).toBeDefined();
          const asset = compilation.getAsset(assetFilename!);
          expect(asset).toBeDefined();

          const source = `// prefix\n${asset!.source.source().toString()}`;
          compilation.updateAsset(
            assetFilename!,
            new rspack.sources.RawSource(source),
          );

          expectedSource = `${source}// suffix\n`;
          compilation.updateAsset(
            assetFilename!,
            new rspack.sources.RawSource(expectedSource),
          );
        });
      });

      const assertAsset = () => {
        expect(assetFilename).toBeDefined();
        const asset = compiler._lastCompilation!.getAsset(assetFilename!);
        expect(asset).toBeDefined();
        expect(asset!.source.source().toString()).toBe(expectedSource);
        expect(asset!.source.size()).toBe(
          Buffer.byteLength(expectedSource!),
        );
        expect(
          compiler._lastCompilation!.getAsset(`${assetFilename}.map`),
        ).toBeDefined();
      };

      compiler.hooks.done.tap('PLUGIN', assertAsset);
      compiler.hooks.afterDone.tap('PLUGIN', assertAsset);
    }),
  ],
});
