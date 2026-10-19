import { defineConfig } from '@rspack/cli';
import { rspack, type Compiler } from '@rspack/core';

const oldHash = 'aabbccdd';
const content = `😀:${oldHash}:end`;
const binary = Buffer.concat([Buffer.from([0xff, 0]), Buffer.from(oldHash)]);

function digest(compiler: Compiler, value: string | Buffer) {
  const hash = compiler.webpack.util.createHash(
    compiler.options.output.hashFunction ?? 'xxhash64',
  );
  hash.update(Buffer.isBuffer(value) ? value : Buffer.from(value));
  return hash.digest('hex').slice(0, 8);
}

class TestPlugin {
  private readonly overrideHash: boolean;

  constructor(overrideHash: boolean) {
    this.overrideHash = overrideHash;
  }

  apply(compiler: Compiler) {
    compiler.hooks.thisCompilation.tap('Test', (compilation) => {
      const { RawSource, SourceMapSource } = compiler.webpack.sources;
      let hookCalls = 0;
      if (this.overrideHash) {
        rspack.optimize.RealContentHashPlugin.getCompilationHooks(
          compilation,
        ).updateHash.tap('Test', (assets, hash) => {
          if (hash !== oldHash) return;
          hookCalls++;
          expect(assets).toHaveLength(1);
          expect(Buffer.isBuffer(assets[0])).toBe(true);
          expect(assets[0].toString()).toBe('😀::end');
          return 'abcdef01';
        });
      }
      compilation.hooks.processAssets.tap(
        {
          name: 'Test',
          stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
        },
        () => {
          const sourceMap = {
            version: 3,
            file: 'mapped.js',
            sources: ['original.js'],
            sourcesContent: [content],
            names: [],
            mappings: 'AAAA',
          };
          compilation.emitAsset(`one.${oldHash}.txt`, new RawSource(content), {
            contenthash: oldHash,
          });
          compilation.emitAsset(
            `two.${oldHash}.txt`,
            new SourceMapSource(content, 'mapped.js', sourceMap),
            { contenthash: oldHash },
          );
          compilation.emitAsset(
            'plain.txt',
            new SourceMapSource('unchanged', 'plain.js', sourceMap),
          );
          compilation.emitAsset('binary.deadbeef.bin', new RawSource(binary), {
            contenthash: 'deadbeef',
          });
        },
      );
      compilation.hooks.processAssets.tap(
        {
          name: 'Test',
          stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_REPORT,
        },
        () => {
          const hash = this.overrideHash
            ? 'abcdef01'
            : digest(compiler, '😀::end');
          for (const prefix of ['one', 'two']) {
            const asset = compilation.getAsset(`${prefix}.${hash}.txt`)!;
            expect(asset).toBeDefined();
            expect(asset.source.source()).toBe(`😀:${hash}:end`);
            expect(asset.source.map()).toBeNull();
          }
          expect(
            compilation.getAsset('plain.txt')!.source.map(),
          ).not.toBeNull();
          const binaryAsset = compilation.getAsset(
            `binary.${digest(compiler, binary)}.bin`,
          )!;
          expect(binaryAsset.source.buffer()).toEqual(binary);
          if (this.overrideHash) expect(hookCalls).toBe(1);
          for (const asset of compilation.getAssets()) {
            if (!asset.name.endsWith('.js')) continue;
            const mapName = asset.info.related?.sourceMap;
            if (mapName) {
              const mapAsset = compilation.getAsset(mapName as string)!;
              const map = JSON.parse(mapAsset.source.source().toString());
              expect(map.file).toBe(asset.name);
              expect(map.mappings.length).toBeGreaterThan(0);
            }
          }
        },
      );
    });
  }
}

export default defineConfig(
  (
    [
      { devtool: false, hashFunction: 'xxhash64', overrideHash: false },
      { devtool: 'source-map', hashFunction: 'sha256', overrideHash: false },
      { devtool: 'source-map', hashFunction: 'md4', overrideHash: true },
    ] as const
  ).map(({ devtool, hashFunction, overrideHash }) => ({
    mode: 'production',
    devtool: devtool as false | 'source-map',
    output: {
      filename: 'main.[contenthash:8].js',
      sourceMapFilename: '[file].[contenthash:8].map',
      hashFunction,
    },
    optimization: { minimize: false, realContentHash: true },
    plugins: [new TestPlugin(overrideHash)],
  })),
);
