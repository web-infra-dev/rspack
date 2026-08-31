const rspack = require('@rspack/core');
const path = require('path');

let index = 0;
let builtErrorModules = [];
let validErrorModules = [];

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
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleCacheErrorTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleCacheErrorTest',
              (module) => {
                if (
                  module.resource &&
                  path.basename(module.resource) === 'file.js'
                ) {
                  builtErrorModules.push(path.basename(module.resource));
                }
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ModuleCacheErrorTest',
              (module) => {
                if (
                  module.resource &&
                  path.basename(module.resource) === 'file.js'
                ) {
                  validErrorModules.push(path.basename(module.resource));
                }
              },
            );
          },
        );
        compiler.hooks.done.tapPromise('PLUGIN', async (stats) => {
          const { errors } = stats.toJson({ errors: true });
          expect(builtErrorModules).toEqual(['file.js']);
          expect(validErrorModules).toEqual([]);
          expect(errors).toHaveLength(1);
          expect(errors[0].message).toMatch('LoaderError');
          builtErrorModules = [];
          validErrorModules = [];
          index++;
        });
      },
    },
  ],
};
