import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule, rspack } from '@rspack/core';

const { RawSource } = rspack.sources;

let compilerIndex = 0;
const loaderOptions: { builtModules: string[] } = {
  builtModules: [],
};

export default defineConfig({
  context: import.meta.dirname,
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
        loader: './loader.mjs',
        options: loaderOptions,
      },
      {
        test: /data\.json$/,
        type: 'json',
        parser: {
          parse: JSON.parse,
        },
        loader: './loader.mjs',
        options: loaderOptions,
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap('ModuleCacheTest', (compilation) => {
          compilation.hooks.succeedModule.tap('ModuleCacheTest', (module) => {
            if (
              module instanceof NormalModule &&
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
              ?.source.source(),
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
            // Whole-module persistence skips the JSON module's custom parser callback.
            expect(builtModules).toEqual(['changed.js', 'data.json']);
          }
          loaderOptions.builtModules = [];
          compilerIndex++;
        });
      },
    }),
  ],
});
