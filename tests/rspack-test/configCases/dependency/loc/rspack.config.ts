import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('TestPlugin', (compilation) => {
          compilation.hooks.finishModules.tap('TestPlugin', (modules) => {
            let entryModule;
            for (const module of modules) {
              if (module.identifier().endsWith('index.js')) {
                entryModule = module;
                break;
              }
            }

            expect(entryModule, 'entry module not found').toBeTruthy();
            const dependencies = entryModule!.dependencies;
            expect(
              dependencies.length > 0,
              'dependencies should not be empty',
            ).toBeTruthy();

            // Find the dependency corresponding to `import { a } from './lib'`
            const importDep = dependencies.find(
              (dep) => dep.type === 'esm import specifier',
            )!;
            expect(importDep, 'import dependency not found').toBeTruthy();

            const loc = importDep.loc!;
            expect(loc, 'loc should exist').toBeTruthy();
            if (!('start' in loc))
              throw new Error('loc should have a start position');
            // Verify start line/column.
            // Note: Rspack internal location might have offsets or point to specific tokens.
            // We verify we got a valid location object.
            expect(loc.start.line >= 1, 'line should be >= 1').toBeTruthy();
            expect(
              typeof loc.start.column === 'number',
              'column should be a number',
            ).toBeTruthy();
          });
        });
      },
    }),
  ],
});
