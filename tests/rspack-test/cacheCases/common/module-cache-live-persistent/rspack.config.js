const path = require('node:path');

let round = 0;
let built = 0;
let reused = 0;

module.exports = {
  context: __dirname,
  cache: { type: 'persistent', maxMemoryGenerations: 0 },
  incremental: false,
  experiments: { newCache: { module: true } },
  optimization: { concatenateModules: false },
  plugins: [
    (compiler) => {
      compiler.hooks.compilation.tap('PersistentLiveModule', (compilation) => {
        const isStable = (module) =>
          module.resource && path.basename(module.resource) === 'stable.js';
        const emit = (module) =>
          module.emitFile(
            `hit-${round}.txt`,
            new compiler.rspack.sources.RawSource(String(round)),
          );
        compilation.hooks.succeedModule.tap(
          'PersistentLiveModule',
          (module) => {
            if (!isStable(module)) return;
            built++;
            emit(module);
          },
        );
        compilation.hooks.stillValidModule.tap(
          'PersistentLiveModule',
          (module) => {
            if (!isStable(module)) return;
            reused++;
            for (let i = 0; i < round; i++) {
              expect(Object.keys(module.buildInfo.assets)).toContain(
                `hit-${i}.txt`,
              );
            }
            emit(module);
          },
        );
      });
      compiler.hooks.done.tap('PersistentLiveModule', (stats) => {
        expect(stats.hasErrors()).toBe(false);
        expect(built).toBe(1);
        expect(reused).toBe(round);
        for (let i = 0; i <= round; i++) {
          expect(
            stats.compilation.getAsset(`hit-${i}.txt`).source.source(),
          ).toBe(String(i));
        }
        round++;
      });
    },
  ],
};
