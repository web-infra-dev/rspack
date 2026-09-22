import { defineConfig, definePlugin } from '@rspack/cli';
import { ContextModule } from '@rspack/core';

let compilerIndex = 0;
let contextIdentifiers: string[] | undefined;

export default defineConfig({
  experiments: {
    newCache: {
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
      module: true,
    },
  },
  cache: {
    type: 'persistent',
    snapshot: {
      contextModule: { timestamp: false, hash: true },
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const built: string[] = [];
        const restored: string[] = [];
        compiler.hooks.compilation.tap(
          'ContextModuleCacheTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ContextModuleCacheTest',
              (module) => {
                if (module instanceof ContextModule)
                  built.push(module.identifier());
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ContextModuleCacheTest',
              (module) => {
                if (module instanceof ContextModule)
                  restored.push(module.identifier());
              },
            );
          },
        );
        compiler.hooks.done.tap('ContextModuleCacheTest', () => {
          if (compilerIndex === 0) {
            expect(built).toHaveLength(7);
            expect(restored).toEqual([]);
            contextIdentifiers = built.sort();
          } else {
            expect(built).toEqual([]);
            expect(restored.sort()).toEqual(contextIdentifiers);
          }
          compilerIndex++;
        });
      },
    }),
  ],
});
