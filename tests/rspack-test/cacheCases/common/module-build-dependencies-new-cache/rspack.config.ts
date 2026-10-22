import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

let compilerIndex = 0;
let builtModules: string[] = [];

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
        test: /input\.js$/,
        loader: './loader.mjs',
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleBuildDependenciesTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleBuildDependenciesTest',
              (module) => {
                if (
                  module instanceof NormalModule &&
                  module.resource &&
                  path.basename(module.resource) === 'input.js'
                ) {
                  builtModules.push(path.basename(module.resource));
                }
              },
            );
          },
        );
        compiler.hooks.done.tap('ModuleBuildDependenciesTest', () => {
          expect(builtModules).toEqual(['input.js']);
          builtModules = [];
          compilerIndex++;
        });
      },
    }),
  ],
});
