const path = require('path');

const TRACKED_MODULES = new Set(['a.js', 'b.js', 'index.js', 'stable.js']);

let compilerIndex = 0;
let builtModules = [];
let stillValidModules = [];
let succeededModules = [];

const getTrackedModuleName = (module) => {
  if (!module.resource) return;
  const name = path.basename(module.resource);
  if (TRACKED_MODULES.has(name)) return name;
};

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  context: __dirname,
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
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleCacheSucceedModuleTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleCacheSucceedModuleTest',
              (module) => {
                const name = getTrackedModuleName(module);
                if (name) builtModules.push(name);
              },
            );
            compilation.hooks.succeedModule.tap(
              'ModuleCacheSucceedModuleTest',
              (module) => {
                const name = getTrackedModuleName(module);
                if (!name) return;
                succeededModules.push(name);
                if (name === 'a.js' || name === 'b.js') {
                  module.buildInfo.cacheable = false;
                }
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ModuleCacheSucceedModuleTest',
              (module) => {
                const name = getTrackedModuleName(module);
                if (name) stillValidModules.push(name);
              },
            );
          },
        );
        compiler.hooks.done.tap('ModuleCacheSucceedModuleTest', () => {
          builtModules.sort();
          stillValidModules.sort();
          succeededModules.sort();
          if (compilerIndex === 0) {
            expect(builtModules).toEqual([
              'a.js',
              'b.js',
              'index.js',
              'stable.js',
            ]);
            expect(succeededModules).toEqual(builtModules);
            expect(stillValidModules).toEqual([]);
          } else {
            expect(builtModules).toEqual(['a.js', 'b.js']);
            expect(succeededModules).toEqual(['a.js', 'b.js']);
            expect(stillValidModules).toEqual(['index.js', 'stable.js']);
          }
          builtModules = [];
          stillValidModules = [];
          succeededModules = [];
          compilerIndex++;
        });
      },
    },
  ],
};
