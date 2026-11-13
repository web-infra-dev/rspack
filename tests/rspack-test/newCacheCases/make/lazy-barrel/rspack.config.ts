import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const pluginName = 'LazyBarrel';
const barrelPath = path.resolve(import.meta.dirname, 'lib/index.js');
const fallbackPath = path.resolve(import.meta.dirname, 'lib/c.js');
let compilerIndex = 0;

export default defineConfig({
  context: import.meta.dirname,
  optimization: {
    // avoid analyze side effects that will change index.js dependencies at HMR
    sideEffects: 'flag',
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin((compiler) => {
      const run = compilerIndex++;
      // Keep the original HMR/restart steps, then change only factory metadata.
      const sideEffects = run === 3;
      compiler.options.module.rules = [
        { test: /[\\/]lib[\\/]index\.js$/, sideEffects },
      ];

      let firstCompilation = true;
      let barrelBuilt = 0;
      let barrelRestored = 0;
      const createdModules = new Set<string>();
      compiler.hooks.compilation.tap(
        pluginName,
        (compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.createModule.tap(pluginName, (data) => {
            createdModules.add(data.resourceResolveData.resource);
          });
          compilation.hooks.buildModule.tap(pluginName, (module) => {
            if (module.identifier().endsWith(barrelPath)) barrelBuilt++;
          });
          compilation.hooks.stillValidModule.tap(pluginName, (module) => {
            if (module.identifier().endsWith(barrelPath)) barrelRestored++;
          });
          compilation.hooks.finishModules.tap(pluginName, () => {
            const barrel = [...compilation.modules].find((module) =>
              module.identifier().endsWith(barrelPath),
            )!;
            expect(barrel.factoryMeta.sideEffectFree).toBe(!sideEffects);
            const fallbackDependencies = barrel.dependencies.filter(
              (dependency) => dependency.request === './c',
            );
            expect(fallbackDependencies.length).toBeGreaterThan(0);
            expect(
              fallbackDependencies.some(
                (dependency) =>
                  compilation.moduleGraph.getModule(dependency) !== null,
              ),
            ).toBe(sideEffects);
          });
        },
      );
      compiler.hooks.done.tap(pluginName, () => {
        expect(createdModules.has(fallbackPath)).toBe(sideEffects);
        if (firstCompilation) {
          expect({ barrelBuilt, barrelRestored }).toEqual({
            barrelBuilt: run === 0 ? 1 : 0,
            barrelRestored: run === 0 ? 0 : 1,
          });
          firstCompilation = false;
        }
      });
    }),
  ],
});
