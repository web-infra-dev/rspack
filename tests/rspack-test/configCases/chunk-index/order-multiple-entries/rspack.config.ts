import { defineConfig } from '@rspack/cli';
import type { Compilation, Module, Compiler } from '@rspack/core';

export default defineConfig({
  entry: {
    entry1: './entry1',
    entry2: './entry2',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
  },
  plugins: [
    function (this: Compiler) {
      const handler = (compilation: Compilation) => {
        const moduleGraph = compilation.moduleGraph;
        compilation.hooks.afterSeal.tap('testcase', () => {
          const data: Record<string, string> = {};
          for (const [name, group] of compilation.namedChunkGroups) {
            const modules = new Map<Module, number>();
            const modules2 = new Map<Module, number>();
            for (const chunk of group.chunks) {
              for (const module of compilation.chunkGraph.getChunkModulesIterable(
                chunk,
              )) {
                const preOrder = group.getModulePreOrderIndex(module);
                if (typeof preOrder === 'number') {
                  modules.set(module, preOrder);
                }
                const postOrder = group.getModulePostOrderIndex(module);
                if (typeof postOrder === 'number') {
                  modules2.set(module, postOrder);
                }
              }
            }
            const sortedModules = Array.from(modules).sort((a, b) => {
              return a[1] - b[1];
            });
            const sortedModules2 = Array.from(modules2).sort((a, b) => {
              return a[1] - b[1];
            });
            const text = sortedModules
              .map(
                ([m, index]) =>
                  `${index}: ${m.readableIdentifier(
                    // @ts-expect-error Rspack does not expose the webpack requestShortener API.
                    compilation.requestShortener,
                  )}`,
              )
              .join(', ');
            const text2 = sortedModules2
              .map(
                ([m, index]) =>
                  `${index}: ${m.readableIdentifier(
                    // @ts-expect-error Rspack does not expose the webpack requestShortener API.
                    compilation.requestShortener,
                  )}`,
              )
              .join(', ');
            data[name + 'Index'] = text;
            data[name + 'Index2'] = text2;
          }
          expect(data).toEqual({
            entry1Index:
              '0: ./entry1.js, 1: ./a.js, 2: ./shared.js, 3: ./b.js, 4: ./c.js',
            entry1Index2:
              '0: ./shared.js, 1: ./a.js, 2: ./b.js, 3: ./c.js, 4: ./entry1.js',
            entry2Index:
              '0: ./entry2.js, 1: ./c.js, 2: ./b.js, 3: ./shared.js, 4: ./a.js',
            entry2Index2:
              '0: ./c.js, 1: ./shared.js, 2: ./b.js, 3: ./a.js, 4: ./entry2.js',
            asyncIndex: '0: ./async.js',
            asyncIndex2: '0: ./async.js',
          });
          const indices = Array.from(compilation.modules)
            .map(
              (m) =>
                [
                  // @ts-expect-error Rspack does not expose module graph traversal indices yet.
                  moduleGraph.getPreOrderIndex(m),
                  m,
                ] as [number, Module],
            )
            .filter((p) => typeof p[0] === 'number')
            .sort((a, b) => a[0] - b[0])
            .map(
              ([i, m]) =>
                // @ts-expect-error Rspack does not expose the webpack requestShortener API.
                `${i}: ${m.readableIdentifier(compilation.requestShortener)}`,
            )
            .join(', ');
          const indices2 = Array.from(compilation.modules)
            .map(
              (m) =>
                [
                  // @ts-expect-error Rspack does not expose module graph traversal indices yet.
                  moduleGraph.getPostOrderIndex(m),
                  m,
                ] as [number, Module],
            )
            .filter((p) => typeof p[0] === 'number')
            .sort((a, b) => a[0] - b[0])
            .map(
              ([i, m]) =>
                // @ts-expect-error Rspack does not expose the webpack requestShortener API.
                `${i}: ${m.readableIdentifier(compilation.requestShortener)}`,
            )
            .join(', ');
          expect(indices).toEqual(
            '0: ./entry1.js, 1: ./a.js, 2: ./shared.js, 3: ./b.js, 4: ./c.js, 5: ./entry2.js, 6: ./async.js',
          );
          expect(indices2).toEqual(
            '0: ./shared.js, 1: ./a.js, 2: ./b.js, 3: ./c.js, 4: ./entry1.js, 5: ./entry2.js, 6: ./async.js',
          );
        });
      };
      this.hooks.compilation.tap('testcase', handler);
    },
  ],
});
