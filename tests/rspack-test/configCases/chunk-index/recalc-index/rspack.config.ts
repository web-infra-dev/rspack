import { defineConfig } from '@rspack/cli';
import type { Compilation, Module, Compiler } from '@rspack/core';

export default defineConfig({
  entry: {
    main: './index.js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  plugins: [
    function (this: Compiler) {
      const handler = (compilation: Compilation) => {
        compilation.hooks.afterSeal.tap('testcase', () => {
          const data: Record<string, string> = {};
          for (const [name, group] of compilation.namedChunkGroups) {
            const modules = new Map<Module, number>();
            for (const chunk of group.chunks) {
              for (const module of compilation.chunkGraph.getChunkModulesIterable(
                chunk,
              )) {
                const postOrder = group.getModulePostOrderIndex(module);
                if (typeof postOrder === 'number') {
                  modules.set(module, postOrder);
                }
              }
            }
            const sortedModules = Array.from(modules).sort(
              (a, b) => a[1] - b[1],
            );
            const text = sortedModules
              .map(
                ([m, index]) =>
                  `${index}: ${m.readableIdentifier(
                    // @ts-expect-error Rspack does not expose the webpack requestShortener API.
                    compilation.requestShortener,
                  )}`,
              )
              .join(', ');
            data[`${name}Index`] = text;
          }
          expect(data).toEqual({
            dynamicIndex: '0: css ./a.css, 1: css ./b.css',
            mainIndex: '0: ./index.js',
          });
        });
      };
      this.hooks.compilation.tap('testcase', handler);
    },
  ],
});
