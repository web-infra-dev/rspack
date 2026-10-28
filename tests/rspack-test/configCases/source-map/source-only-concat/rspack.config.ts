import { defineConfig } from '@rspack/cli';
import type { Compiler, Configuration } from '@rspack/core';

const content = 'globalThis.sourceOnly = "😀尾巴";\n';

export default defineConfig(
  ['web', 'node'].flatMap((target) =>
    [
      'source-map',
      'hidden-source-map',
      'inline-source-map',
      'nosources-source-map',
    ].map(
      (devtool) =>
        ({
          target,
          devtool,
          optimization: { minimize: false },
          plugins: [
            {
              apply(compiler: Compiler) {
                compiler.hooks.thisCompilation.tap(
                  'SourceOnlyConcat',
                  (compilation) => {
                    compilation.hooks.processAssets.tap(
                      {
                        name: 'SourceOnlyConcat',
                        stage:
                          compiler.webpack.Compilation
                            .PROCESS_ASSETS_STAGE_ADDITIONS,
                      },
                      () => {
                        const { ConcatSource, OriginalSource, RawSource } =
                          compiler.webpack.sources;
                        compilation.emitAsset(
                          'mapped.js',
                          new ConcatSource(
                            new OriginalSource(content, 'original.js'),
                            new RawSource('// generated suffix\n'),
                          ),
                        );
                      },
                    );
                  },
                );
                compiler.hooks.afterEmit.tap(
                  'SourceOnlyConcatCheck',
                  (compilation) => {
                    const source = compilation.getAsset('mapped.js')!.source;
                    const text = source.source().toString();
                    expect(text.startsWith(content)).toBe(true);
                    expect(source.map()).toBeNull();
                    expect(source.size()).toBe(Buffer.byteLength(text));
                    expect(source.buffer()).toEqual(Buffer.from(text));
                    let map;
                    if (devtool === 'inline-source-map') {
                      const encoded = text.match(/base64,([^\s]+)/)![1];
                      map = JSON.parse(
                        Buffer.from(encoded, 'base64').toString(),
                      );
                    } else {
                      map = JSON.parse(
                        compilation
                          .getAsset('mapped.js.map')!
                          .source.source()
                          .toString(),
                      );
                      expect(text.includes('sourceMappingURL=')).toBe(
                        devtool !== 'hidden-source-map',
                      );
                    }
                    expect(map.sources).toEqual([
                      expect.stringMatching(/:\/\/\/original\.js$/),
                    ]);
                    if (devtool === 'nosources-source-map') {
                      expect(map.sourcesContent).toBeUndefined();
                    } else {
                      expect(map.sourcesContent).toContain(content);
                    }
                    expect(map.mappings.length).toBeGreaterThan(0);
                  },
                );
              },
            },
          ],
        }) as Configuration,
    ),
  ),
);
