import { ContextModule } from '@rspack/core';

let compilerIndex = 0;
let contextIdentifiers;

/** @type {import('@rspack/core').Configuration} */
export default {
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
      module: { timestamp: false, hash: true },
      contextModule: { timestamp: false, hash: true },
    },
  },
  plugins: [
    {
      apply(compiler) {
        const built = [];
        const restored = [];
        compiler.hooks.compilation.tap(
          'ContextModuleInvalidationTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ContextModuleInvalidationTest',
              (module) => {
                if (module instanceof ContextModule)
                  built.push(module.identifier());
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ContextModuleInvalidationTest',
              (module) => {
                if (module instanceof ContextModule)
                  restored.push(module.identifier());
              },
            );
          },
        );
        compiler.hooks.done.tap('ContextModuleInvalidationTest', () => {
          if (compilerIndex === 0) {
            expect(built).toHaveLength(2);
            expect(restored).toEqual([]);
            contextIdentifiers = built.sort();
          } else if (compilerIndex === 1 || compilerIndex === 5) {
            expect(built).toEqual([]);
            expect(restored.sort()).toEqual(contextIdentifiers);
          } else {
            expect(built).toEqual(
              contextIdentifiers.filter((id) => id.includes('|lazy|')),
            );
            expect(restored).toEqual(
              contextIdentifiers.filter((id) => id.includes('|sync|')),
            );
          }
          compilerIndex++;
        });
      },
    },
  ],
};
