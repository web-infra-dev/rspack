import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule, rspack } from '@rspack/core';
import path from 'node:path';

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
  optimization: {
    concatenateModules: false,
    sideEffects: true,
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
          const { modules } = stats.toJson({
            all: false,
            modules: true,
            cachedModules: true,
            orphanModules: true,
            optimizationBailout: true,
          });
          const sideEffectsBailouts = (name: string) =>
            modules
              ?.find((module) => module.name === `./${name}`)
              ?.optimizationBailout?.filter((reason) =>
                reason.includes('with side_effects in source code'),
              );
          expect(sideEffectsBailouts('stable.js')).toEqual([
            expect.stringContaining('ExportDefaultExpr with side_effects'),
          ]);
          expect(sideEffectsBailouts('changed.js')).toEqual(
            compilerIndex === 0
              ? [expect.stringContaining('Statement with side_effects')]
              : [],
          );
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
