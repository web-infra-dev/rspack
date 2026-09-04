const path = require('path');

const PLUGIN_NAME = 'FinishModulesArtifacts';
const runtime = 'finish-modules-test';

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  optimization: {
    concatenateModules: false,
    providedExports: true,
    usedExports: true,
    sideEffects: true,
  },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
        const moduleGraph = compilation.moduleGraph;
        let syncModule;
        let exportsInfo;

        const check = () => {
          const asyncModule = [...compilation.modules].find(
            (module) => module.resource === path.join(__dirname, 'async.js'),
          );
          expect(compilation.moduleGraph).toBe(moduleGraph);
          expect(moduleGraph.getProvidedExports(syncModule).sort()).toEqual([
            'unused',
            'value',
          ]);
          expect(moduleGraph.getUsedExports(syncModule, 'main')).toBeNull();
          expect(moduleGraph.getUsedExports(syncModule, ['main'])).toBeNull();
          expect(exportsInfo.isUsed('main')).toBe(true);
          expect(exportsInfo.isModuleUsed('main')).toBe(true);
          expect(exportsInfo.getUsed('value', 'main')).toBe(2);
          expect(exportsInfo.getUsed(['value'], new Set(['main']))).toBe(2);
          expect(moduleGraph.isAsync(syncModule)).toBe(false);
          expect(moduleGraph.isAsync(asyncModule)).toBe(true);

          // A side-effect-free import is inactive even before usage analysis.
          const states = moduleGraph
            .getIncomingConnections(syncModule)
            .map((connection) => connection.getActiveState('main'));
          expect(states).toContain(false);
        };

        compilation.hooks.finishModules.tap(
          { name: PLUGIN_NAME, stage: 20 },
          (modules) => {
            syncModule = [...modules].find(
              (module) => module.resource === path.join(__dirname, 'sync.js'),
            );
            exportsInfo = moduleGraph.getExportsInfo(syncModule);
            check();
            expect(exportsInfo.setUsedInUnknownWay(runtime)).toBe(true);
          },
        );

        compilation.hooks.finishModules.tapPromise(
          { name: PLUGIN_NAME, stage: 30 },
          async () => {
            check();
            await new Promise((resolve) => setImmediate(resolve));
            check();
            // The mutation must reach the same artifact across taps and await.
            expect(exportsInfo.setUsedInUnknownWay(runtime)).toBe(false);
            expect(
              moduleGraph
                .getExportsInfo(syncModule)
                .setUsedInUnknownWay(runtime),
            ).toBe(false);
          },
        );

        compilation.hooks.seal.tap(PLUGIN_NAME, () => {
          check();
          expect(exportsInfo.setUsedInUnknownWay(runtime)).toBe(false);
        });
      });
    },
  ],
};
