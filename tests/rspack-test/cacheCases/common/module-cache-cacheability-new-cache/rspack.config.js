const path = require('path');

const TRACKED_MODULES = new Set([
  'a.js',
  'b.js',
  'c.js',
  'cleared.js',
  'd.js',
  'e.js',
  'f.js',
  'index.js',
  'stable.js',
]);
const NOT_CACHEABLE_MODULES = ['a.js', 'b.js', 'c.js', 'd.js', 'e.js', 'f.js'];

let compilerIndex = 0;
let builtModules = [];
let stillValidModules = [];

const recordModule = (modules, module) => {
  if (!module.resource) return;
  const name = path.basename(module.resource);
  if (TRACKED_MODULES.has(name)) modules.push(name);
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
  module: {
    rules: [
      {
        test: /[\\/](?:a|b|c|d|e|f)\.js$/,
        loader: './no-cache-loader.js',
      },
      {
        test: /[\\/]cleared\.js$/,
        loader: './clear-loader.js',
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleCacheCacheabilityTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleCacheCacheabilityTest',
              (module) => recordModule(builtModules, module),
            );
            compilation.hooks.stillValidModule.tap(
              'ModuleCacheCacheabilityTest',
              (module) => recordModule(stillValidModules, module),
            );
          },
        );
        compiler.hooks.done.tap('ModuleCacheCacheabilityTest', () => {
          builtModules.sort();
          stillValidModules.sort();
          if (compilerIndex === 0) {
            expect(builtModules).toEqual([
              'a.js',
              'b.js',
              'c.js',
              'cleared.js',
              'd.js',
              'e.js',
              'f.js',
              'index.js',
              'stable.js',
            ]);
            expect(stillValidModules).toEqual([]);
          } else {
            expect(builtModules).toEqual(NOT_CACHEABLE_MODULES);
            expect(stillValidModules).toEqual([
              'cleared.js',
              'index.js',
              'stable.js',
            ]);
          }
          builtModules = [];
          stillValidModules = [];
          compilerIndex++;
        });
      },
    },
  ],
};
