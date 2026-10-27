import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.thisCompilation.tap('SourceFallback', (compilation) => {
        compilation.hooks.processAssets.tap('SourceFallback', () => {
          const { RawSource, SourceMapSource, ConcatSource, ReplaceSource } =
            compiler.rspack.sources;
          compilation.emitAsset(
            'buffer.bin',
            new RawSource(Buffer.from('abc')),
          );
          const buffer = compilation.getAsset('buffer.bin')!.source;
          const content = buffer.source();
          if (!Buffer.isBuffer(content)) throw new Error('Expected a Buffer');
          content[0] = 0x41;
          compilation.updateAsset('buffer.bin', buffer);

          for (const [name, source] of [
            [
              'single-buffer.txt',
              new ConcatSource(new RawSource(Buffer.from('body'))),
            ],
            [
              'mixed-buffer.txt',
              new ConcatSource('prefix:', new RawSource(Buffer.from('body'))),
            ],
          ] as const) {
            compilation.emitAsset(name, source);
            expect(typeof compilation.getAsset(name)!.source.source()).toBe(
              'string',
            );
          }

          const map = {
            version: 3,
            file: 'mapped.txt',
            sources: ['original.txt'],
            sourcesContent: ['body'],
            names: [],
            mappings: 'AAAA',
          };
          for (const readMap of ['map', 'sourceAndMap'] as const) {
            const name = `${readMap}.txt`;
            compilation.emitAsset(name, new SourceMapSource('body', name, map));
            const source = compilation.getAsset(name)!.source;
            const exposedMap =
              readMap === 'map' ? source.map() : source.sourceAndMap().map;
            if (!exposedMap) throw new Error('Expected a source map');
            exposedMap.sources[0] = 'changed.txt';
            compilation.updateAsset(name, source);
            compilation.emitAsset(
              `${readMap}-result.json`,
              new RawSource(
                JSON.stringify(compilation.getAsset(name)!.source.map()),
              ),
            );

            const replacementOriginal = compilation.getAsset(name)!.source;
            const replacementMap =
              readMap === 'map'
                ? replacementOriginal.map()
                : replacementOriginal.sourceAndMap().map;
            if (!replacementMap) throw new Error('Expected a source map');
            replacementMap.sources[0] = 'replacement.txt';
            const replacement = new ReplaceSource(replacementOriginal);
            replacement.insert(0, 'prefix:');
            compilation.emitAsset(`${readMap}-replace.txt`, replacement);
            compilation.emitAsset(
              `${readMap}-replace-result.json`,
              new RawSource(
                JSON.stringify(
                  compilation.getAsset(`${readMap}-replace.txt`)!.source.map(),
                ),
              ),
            );
          }

          class CustomConcatSource extends ConcatSource {
            override source() {
              return `prefix:${super.source()}`;
            }
          }
          class CustomReplaceSource extends ReplaceSource {
            override source() {
              return `prefix:${super.source()}`;
            }
          }
          compilation.emitAsset('original.txt', new RawSource('body'));
          compilation.emitAsset(
            'subclass-concat.txt',
            new CustomConcatSource('body'),
          );
          compilation.emitAsset(
            'subclass-replace.txt',
            new CustomReplaceSource(
              compilation.getAsset('original.txt')!.source,
            ),
          );
        });
      });
    }),
  ],
});
