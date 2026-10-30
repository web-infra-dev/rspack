import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';

const oldHash = 'aabbccdd';
const content = 'console.log("same content");';

export default defineConfig({
  mode: 'production',
  target: 'web',
  devtool: 'hidden-source-map',
  optimization: { minimize: false, realContentHash: true },
  plugins: [
    {
      apply(compiler: Compiler) {
        compiler.hooks.thisCompilation.tap('SourceOnlyDedup', (compilation) => {
          const { RawSource, SourceMapSource } = compiler.webpack.sources;
          compilation.hooks.processAssets.tap(
            {
              name: 'SourceOnlyDedup',
              stage:
                compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
            },
            () => {
              // Devtool used to flatten both mapped graphs to identical raw text.
              for (const name of ['one', 'two']) {
                compilation.emitAsset(
                  `${name}.${oldHash}.js`,
                  new SourceMapSource(content, `${name}.js`, {
                    version: 3,
                    file: `${name}.js`,
                    sources: [`${name}.js`],
                    sourcesContent: [content],
                    names: [],
                    mappings: 'AAAA',
                  }),
                  { contenthash: oldHash },
                );
              }
              compilation.emitAsset(
                `raw.${oldHash}.js`,
                new RawSource(content),
                {
                  contenthash: oldHash,
                },
              );
            },
          );
          compilation.hooks.processAssets.tap(
            {
              name: 'SourceOnlyDedupCheck',
              stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
            },
            () => {
              const hash = compiler.webpack.util.createHash(
                compiler.options.output.hashFunction ?? 'xxhash64',
              );
              hash.update(Buffer.from(content));
              const expected = hash.digest('hex').slice(0, oldHash.length);
              for (const name of ['one', 'two', 'raw']) {
                expect(
                  compilation.getAssets().map((asset) => asset.name),
                ).toContain(`${name}.${expected}.js`);
                const asset = compilation.getAsset(`${name}.${expected}.js`);
                expect(asset).toBeDefined();
                expect(asset!.source.source().toString()).toBe(content);
                expect(asset!.source.map()).toBeNull();
              }
            },
          );
        });
      },
    },
  ],
});
