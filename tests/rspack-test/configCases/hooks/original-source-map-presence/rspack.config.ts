import { defineConfig } from '@rspack/cli';
import { type Compiler, sources } from '@rspack/core';

const cases = ['empty', 'newlines', 'crlf', 'whitespace', 'unicode', 'mapped'];

export default ([false, 'source-map'] as const).map((devtool) =>
  defineConfig({
    devtool,
    module: {
      rules: [{ test: /input\.js$/, use: ['./loader.mjs'] }],
    },
    plugins: [
      {
        apply(compiler: Compiler) {
          const checked = new Set<string>();
          compiler.hooks.compilation.tap('MapPresence', (compilation) => {
            compilation.hooks.succeedModule.tap('MapPresence', (module) => {
              const name = module.identifier().split('?').pop()!;
              if (!cases.includes(name)) return;
              checked.add(name);
              const source = module.originalSource()!;
              const content = source.source() as string;
              const originalMap = new sources.OriginalSource(
                content,
                'input.js',
              ).map();
              const hasMap = Boolean(
                devtool && (name === 'mapped' || originalMap),
              );
              expect(Object.getPrototypeOf(source)).toBe(
                hasMap
                  ? sources.SourceMapSource.prototype
                  : sources.RawSource.prototype,
              );
              if (hasMap) {
                const map = source.map()!;
                expect(source.map()).toBe(map);
                expect(source.sourceAndMap().map).toBe(map);
                expect(map.mappings).toBe(
                  name === 'mapped' ? '' : originalMap!.mappings,
                );
                expect(map.sourcesContent).toEqual([content]);
                expect(module.originalSource()!.map()).toEqual(map);
              } else {
                expect(source.map()).toBeNull();
                expect(source.sourceAndMap()).toEqual({
                  source: content,
                  map: null,
                });
                const raw = module._originalSource()!;
                expect(raw).not.toBeInstanceOf(sources.SourceMapSource);
                expect(raw.map).toBeUndefined();
              }
            });
          });
          compiler.hooks.done.tap('MapPresence', () => {
            expect([...checked].sort()).toEqual([...cases].sort());
          });
        },
      },
    ],
  }),
);
