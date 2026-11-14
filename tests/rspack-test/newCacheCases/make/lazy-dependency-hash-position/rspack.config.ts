import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';

const pluginName = 'LazyDependencyHashPosition';
let compilerIndex = 0;

export default defineConfig({
  mode: 'development',
  optimization: {
    sideEffects: 'flag',
  },
  plugins: [
    definePlugin((compiler) => {
      const index = compilerIndex++;
      const importerPath = path.join(compiler.context, 'package/index.js');
      const activity = { built: 0, restored: 0 };
      compiler.hooks.compilation.tap(pluginName, (compilation) => {
        compilation.hooks.buildModule.tap(pluginName, (module) => {
          if (module.identifier().endsWith(importerPath)) activity.built++;
        });
        compilation.hooks.stillValidModule.tap(pluginName, (module) => {
          if (module.identifier().endsWith(importerPath)) activity.restored++;
        });
        compilation.hooks.finishModules.tap(pluginName, () => {
          const importer = [...compilation.modules].find((module) =>
            module.identifier().endsWith(importerPath),
          )!;
          const dependencies = importer.dependencies.filter(
            (dependency) => dependency.type === 'esm import',
          );
          expect(dependencies.map((dependency) => dependency.request)).toEqual([
            './a',
            './b',
          ]);
          expect(
            dependencies.map((dependency) =>
              compilation.moduleGraph.getModule(dependency),
            ),
          ).toEqual([null, null]);
        });
      });
      compiler.hooks.done.tap(pluginName, () => {
        expect(activity).toEqual({
          built: index === 0 ? 1 : 0,
          restored: index === 0 ? 0 : 1,
        });
      });
    }),
  ],
});
