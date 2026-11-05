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
          expect(content).toEqual(Buffer.from('abc'));
          for (const method of ['source', 'map']) {
            Object.defineProperty(buffer, method, {
              value: () => {
                throw new Error(`Native binary source serialized ${method}()`);
              },
            });
          }
          compilation.updateAsset('buffer.bin', buffer);

          for (const [name, source] of [
            ['single-buffer.txt', new ConcatSource(buffer)],
            ['mixed-buffer.txt', new ConcatSource('prefix:', buffer)],
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
          for (const method of ['map', 'sourceAndMap', 'buffer'] as const) {
            const name = `${method}.txt`;
            compilation.emitAsset(name, new SourceMapSource('body', name, map));
            const source = compilation.getAsset(name)!.source;
            source[method]();
            for (const read of ['source', 'map']) {
              Object.defineProperty(source, read, {
                value: () => {
                  throw new Error(
                    `Reading ${method}() invalidated native source`,
                  );
                },
              });
            }
            compilation.updateAsset(name, source);
            compilation.emitAsset(
              `${method}-result.json`,
              new RawSource(
                JSON.stringify(compilation.getAsset(name)!.source.map()),
              ),
            );

            const replacement = new ReplaceSource(source);
            replacement.insert(0, 'prefix:');
            compilation.emitAsset(`${method}-replace.txt`, replacement);
            compilation.emitAsset(
              `${method}-replace-result.json`,
              new RawSource(
                JSON.stringify(
                  compilation.getAsset(`${method}-replace.txt`)!.source.map(),
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
          const text = compilation.getAsset('original.txt')!.source;
          const binaryConcat = new ConcatSource(
            text,
            new RawSource(Buffer.from('!')),
          );
          for (const source of [text, binaryConcat]) {
            for (const method of ['source', 'map']) {
              Object.defineProperty(source, method, {
                value: () => {
                  throw new Error(
                    `Buffer concatenation serialized ${method}()`,
                  );
                },
              });
            }
          }
          compilation.emitAsset('buffer-child.txt', binaryConcat);
          compilation.emitAsset(
            'overlap-original.txt',
            new SourceMapSource('body', 'overlap-original.txt', map),
          );
          const overlapping = new ReplaceSource(
            compilation.getAsset('overlap-original.txt')!.source,
          );
          overlapping.replace(0, 2, 'chan', 'replacement');
          overlapping.replace(1, 3, 'ged', 'replacement');
          const sibling = compilation.getAsset('original.txt')!.source;
          const combined = new ConcatSource(
            'prefix:',
            sibling,
            ':',
            overlapping,
          );
          const outer = new ReplaceSource(combined);
          outer.insert(0, 'outer:');
          // Only the overlapping replacements may serialize. Their parents and sibling
          // must keep their native/structured representation.
          for (const source of [sibling, combined, outer]) {
            for (const method of ['source', 'map']) {
              Object.defineProperty(source, method, {
                value: () => {
                  throw new Error(
                    `Parent or sibling used fallback ${method}()`,
                  );
                },
              });
            }
          }
          compilation.emitAsset('overlap-fallback.txt', outer);
          compilation.emitAsset(
            'overlap-map.json',
            new RawSource(
              JSON.stringify(
                compilation.getAsset('overlap-fallback.txt')!.source.map(),
              ),
            ),
          );
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
