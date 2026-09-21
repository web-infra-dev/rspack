import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule, type Module } from '@rspack/core';

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
let builtModules: string[] = [];

const recordModule = (modules: string[], module: Module) => {
  if (!(module instanceof NormalModule) || !module.resource) return;
  const name = path.basename(module.resource);
  if (TRACKED_MODULES.has(name)) modules.push(name);
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
        test: /[\\/](?:a|b|c|d|e|f)\.js$/,
        loader: './no-cache-loader.mjs',
      },
      {
        test: /[\\/]cleared\.js$/,
        loader: './clear-loader.mjs',
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleCacheCacheabilityTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleCacheCacheabilityTest',
              (module) => recordModule(builtModules, module),
            );
          },
        );
        compiler.hooks.done.tap('ModuleCacheCacheabilityTest', () => {
          builtModules.sort();
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
          } else {
            expect(builtModules).toEqual(NOT_CACHEABLE_MODULES);
          }
          builtModules = [];
          compilerIndex++;
        });
      },
    }),
  ],
});
