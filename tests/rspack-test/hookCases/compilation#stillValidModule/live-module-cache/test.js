const path = require('node:path');

let round = 0;
let built = 0;
let succeeded = 0;
let reused = 0;
let firstAssets;

/** @type {import('@rspack/test-tools').THookCaseConfig} */
module.exports = {
  description: 'retains module identity and mutations made on memory cache hits',
  snapshotFileFilter: () => false,
  options() {
    return {
      context: __dirname,
      entry: './index.js',
      cache: { type: 'memory' },
      incremental: false,
      experiments: { newCache: { module: true } },
      module: { rules: [{ test: /index\.js$/, loader: './loader.js' }] },
      optimization: { concatenateModules: false },
      plugins: [compiler => {
        compiler.hooks.compilation.tap('LiveModuleCache', compilation => {
          const isEntry = module => module.resource && path.basename(module.resource) === 'index.js';
          compilation.hooks.buildModule.tap('LiveModuleCache', module => {
            if (isEntry(module)) {
              built++;
              expect(module.identifier()).toContain('index.js');
            }
          });
          compilation.hooks.succeedModule.tap('LiveModuleCache', module => {
            if (!isEntry(module)) return;
            succeeded++;
            module.emitFile('late-0.txt', new compiler.rspack.sources.RawSource('0'));
          });
          compilation.hooks.stillValidModule.tap('LiveModuleCache', module => {
            if (!isEntry(module)) return;
            reused++;
            expect(module.buildInfo.assets).toBe(firstAssets);
            expect(Object.keys(module.buildInfo.assets)).toContain(`late-${round - 1}.txt`);
            // Re-enter the binding for reads and writes while the build-cache
            // dispatcher holds exclusive access to this module.
            module.emitFile(`late-${round}.txt`, new compiler.rspack.sources.RawSource(String(round)));
          });
          compilation.hooks.finishModules.tap('LiveModuleCache', modules => {
            const module = [...modules].find(isEntry);
            expect(module).toBeDefined();
            const assets = module.buildInfo.assets;
            if (!firstAssets) firstAssets = assets;
            expect(assets).toBe(firstAssets);
            expect(Object.keys(assets)).toContain(`late-${round}.txt`);
          });
        });
        compiler.hooks.done.tap('LiveModuleCache', stats => {
          expect(stats.hasErrors()).toBe(false);
          for (let i = 0; i <= round; i++) {
            expect(stats.compilation.getAsset(`late-${i}.txt`).source.source()).toBe(String(i));
          }
          const entry = stats.toJson({ all: false, modules: true, cachedModules: true }).modules.find(module => module.nameForCondition && path.basename(module.nameForCondition) === 'index.js');
          expect(entry).toBeDefined();
          expect(entry.built).toBe(round === 0);
          expect(built).toBe(1);
          expect(succeeded).toBe(1);
          expect(reused).toBe(round);
          round++;
        });
      }],
    };
  },
  async compiler(context, compiler) {
    for (let i = 0; i < 3; i++) {
      await new Promise((resolve, reject) => compiler.run((error, stats) => {
        if (error) reject(error);
        else if (stats.hasErrors()) reject(new Error(stats.toString()));
        else resolve();
      }));
    }
    // The existing hook runner performs the fourth run on this compiler.
  },
};
