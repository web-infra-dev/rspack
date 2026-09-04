const path = require('path');
const { RawSource } = require('webpack-sources');

let compilerIndex = 0;
const loaderOptions = {
  builtModules: [],
};

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
        options: loaderOptions,
      },
      {
        test: /data\.json$/,
        type: 'json',
        parser: {
          parse: JSON.parse,
        },
        loader: './loader.js',
        options: loaderOptions,
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.compilation.tap('ModuleCacheTest', (compilation) => {
          compilation.hooks.succeedModule.tap('ModuleCacheTest', (module) => {
            if (
              module.resource &&
              path.basename(module.resource) === 'stable.js'
            ) {
              module.emitFile(
                'from-succeed-module.txt',
                new RawSource('from succeedModule'),
              );
            }
          });
        });
        compiler.hooks.done.tap('ModuleCacheTest', (stats) => {
          expect(
            stats.compilation
              .getAsset('from-succeed-module.txt')
              .source.source(),
          ).toBe('from succeedModule');
          const builtModules = loaderOptions.builtModules
            .map((resource) => path.basename(resource))
            .sort();
          if (compilerIndex === 0) {
            expect(builtModules).toEqual([
              'changed.js',
              'data.json',
              'stable.js',
            ]);
          } else {
            expect(builtModules).toEqual(['changed.js']);
          }
          loaderOptions.builtModules = [];
          compilerIndex++;
        });
      },
    },
  ],
};
