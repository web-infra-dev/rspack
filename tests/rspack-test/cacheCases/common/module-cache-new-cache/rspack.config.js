const path = require('path');

let compilerIndex = 0;
let stillValidModules = [];

/** @type {import("@rspack/core").Configuration} */
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
        test: /(?:changed|stable)\.js$/,
        loader: './loader.js',
        options: {
          builtModules: [],
        },
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('ModuleCacheTest', (compilation) => {
          compilation.hooks.stillValidModule.tap(
            'ModuleCacheTest',
            (module) => {
              if (module.resource) {
                stillValidModules.push(path.basename(module.resource));
              } else {
                stillValidModules.push('context');
              }
            },
          );
        });
        compiler.hooks.done.tap('ModuleCacheTest', () => {
          const options = compiler.options.module.rules[0].options;
          const builtModules = options.builtModules
            .map((resource) => path.basename(resource))
            .sort();
          if (compilerIndex === 0) {
            expect(builtModules).toEqual(['changed.js', 'stable.js']);
            expect(stillValidModules).toEqual([]);
          } else {
            expect(builtModules).toEqual(['changed.js']);
            expect(stillValidModules.sort()).toEqual([
              'index.js',
              'stable.js',
              'value.js',
            ]);
          }
          options.builtModules = [];
          stillValidModules = [];
          compilerIndex++;
        });
      },
    },
  ],
};
