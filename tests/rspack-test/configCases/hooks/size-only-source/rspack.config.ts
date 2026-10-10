import { defineConfig, definePlugin } from '@rspack/cli';
import { createHash } from 'node:crypto';

export default defineConfig(
  [true, false].map((compareBeforeEmit) => ({
    cache: true,
    devtool: 'source-map',
    optimization: { minimize: false },
    output: { compareBeforeEmit },
    plugins: [
      definePlugin({
        apply(compiler) {
          const { RawSource, SizeOnlySource } = compiler.rspack.sources;
          const sizes = new Map<string, number>();
          const emitted = new Map<
            string,
            Parameters<Parameters<typeof compiler.hooks.assetEmitted.tap>[1]>[1]
          >();
          compiler.hooks.thisCompilation.tap(
            'SizeOnlySource',
            (compilation) => {
              compilation.hooks.processAssets.tap('SizeOnlySource', () => {
                compilation.emitAsset('text.txt', new RawSource('你好🌍'), {
                  custom: 'kept',
                });
                compilation.emitAsset(
                  'binary.bin',
                  new RawSource(Buffer.from([0, 255, 128, 10])),
                );
                compilation.emitAsset('empty.txt', new RawSource(''));
              });
            },
          );
          compiler.hooks.emit.tap('SizeOnlySource', (compilation) => {
            for (const { name, source } of compilation.getAssets()) {
              expect(source).not.toBeInstanceOf(SizeOnlySource);
              sizes.set(name, source.buffer().length);
            }
          });
          compiler.hooks.assetEmitted.tap('SizeOnlySource', (name, info) => {
            expect(info.compilation.getAsset(name)!.source).toBeInstanceOf(
              SizeOnlySource,
            );
            expect(info.compilation.getAsset(name)!.info.size).toBe(
              sizes.get(name),
            );
            // Keep this info without reading content until after all writes.
            emitted.set(name, info);
          });
          compiler.hooks.afterEmit.tap('SizeOnlySource', (compilation) => {
            for (const { name, source, info } of compilation.getAssets()) {
              expect(source).toBeInstanceOf(SizeOnlySource);
              expect(compilation.assets[name]).toBeInstanceOf(SizeOnlySource);
              expect(source.size()).toBe(sizes.get(name));
              expect(info.size).toBe(sizes.get(name));
              for (const read of [
                () => source.source(),
                () => source.buffer(),
                () => source.map(),
                () => source.sourceAndMap(),
                () => source.updateHash(createHash('sha256')),
              ]) {
                expect(read).toThrow(
                  'Content and Map of this Source is not available (only size() is supported)',
                );
              }
              const original = emitted.get(name)!;
              expect(original.source).not.toBeInstanceOf(SizeOnlySource);
              expect(original.content.length).toBe(sizes.get(name));
              expect(original.source).toBe(original.source);
            }
            expect(compilation.getAsset('text.txt')!.info.custom).toBe('kept');
            expect(emitted.get('text.txt')!.source.source()).toBe('你好🌍');
            expect(emitted.get('binary.bin')!.content).toEqual(
              Buffer.from([0, 255, 128, 10]),
            );
            // Size-only snapshots cannot be sent back to Rust as source content.
            const sizeOnly = compilation.getAsset('text.txt')!.source;
            expect(() =>
              compilation.emitAsset('size-only.txt', sizeOnly),
            ).toThrow(
              'Content and Map of this Source is not available (only size() is supported)',
            );
            expect(() => compilation.updateAsset('text.txt', sizeOnly)).toThrow(
              'Content and Map of this Source is not available (only size() is supported)',
            );
            expect(() => {
              compilation.__internal_getInner().emitAsset('numeric.txt', {
                // @ts-expect-error Numeric size-only snapshots are output-only.
                source: sizeOnly.size(),
              });
            }).toThrow();
            expect(compilation.getAsset('size-only.txt')).toBeUndefined();
            expect(compilation.getAsset('numeric.txt')).toBeUndefined();
            expect(compilation.getAsset('text.txt')!.source.size()).toBe(
              sizes.get('text.txt'),
            );
            expect(compilation.getAsset('text.txt')!.info.custom).toBe('kept');
            emitted.clear();
          });
          compiler.hooks.done.tap('SizeOnlySource', (stats) => {
            const assets = stats.toJson({
              all: false,
              assets: true,
              relatedAssets: true,
            }).assets!;
            for (const asset of assets) {
              if (asset.name) expect(asset.size).toBe(sizes.get(asset.name));
            }
          });
        },
      }),
    ],
  })),
);
