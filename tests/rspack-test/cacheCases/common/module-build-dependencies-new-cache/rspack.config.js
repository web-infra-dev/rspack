const path = require('path');

let compilerIndex = 0;
let builtModules = [];
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
        test: /input\.js$/,
        loader: './loader.js',
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleBuildDependenciesTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleBuildDependenciesTest',
              (module) => {
                if (
                  module.resource &&
                  path.basename(module.resource) === 'input.js'
                ) {
                  builtModules.push(path.basename(module.resource));
                }
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ModuleBuildDependenciesTest',
              (module) => {
                if (
                  module.resource &&
                  path.basename(module.resource) === 'input.js'
                ) {
                  stillValidModules.push(path.basename(module.resource));
                }
              },
            );
          },
        );
        compiler.hooks.done.tap('ModuleBuildDependenciesTest', () => {
          expect(builtModules).toEqual(['input.js']);
          expect(stillValidModules).toEqual([]);
          builtModules = [];
          stillValidModules = [];
          compilerIndex++;
        });
      },
    },
  ],
};
