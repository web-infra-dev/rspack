import { defineConfig, definePlugin } from '@rspack/cli';
import { ContextModule } from '@rspack/core';

let compilerIndex = 0;
let contextIdentifier: string | undefined;

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
      module: { timestamp: false, hash: true },
      contextModule: { timestamp: true, hash: false },
    },
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        const built: string[] = [];
        const restored: string[] = [];
        compiler.hooks.compilation.tap(
          'ContextModuleTimestampTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ContextModuleTimestampTest',
              (module) => {
                if (module instanceof ContextModule)
                  built.push(module.identifier());
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ContextModuleTimestampTest',
              (module) => {
                if (module instanceof ContextModule)
                  restored.push(module.identifier());
              },
            );
          },
        );
        compiler.hooks.done.tap('ContextModuleTimestampTest', () => {
          if (compilerIndex === 0) {
            expect(built).toHaveLength(1);
            contextIdentifier = built[0];
          }
          if (compilerIndex === 0 || compilerIndex === 2) {
            expect(built).toEqual([contextIdentifier]);
            expect(restored).toEqual([]);
          } else {
            expect(built).toEqual([]);
            expect(restored).toEqual([contextIdentifier]);
          }
          compilerIndex++;
        });
      },
    }),
  ],
});
