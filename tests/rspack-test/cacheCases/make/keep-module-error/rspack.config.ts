import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';
import path from 'node:path';

let index = 0;
let builtErrorModules: string[] = [];

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
  optimization: {
    minimize: false,
  },
  cache: {
    type: 'persistent',
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.compilation.tap(
          'ModuleCacheErrorTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ModuleCacheErrorTest',
              (module) => {
                if (
                  module instanceof NormalModule &&
                  module.resource &&
                  path.basename(module.resource) === 'file.js'
                ) {
                  builtErrorModules.push(path.basename(module.resource));
                }
              },
            );
          },
        );
        compiler.hooks.done.tapPromise('PLUGIN', async (stats) => {
          const { errors } = stats.toJson({ errors: true });
          expect(builtErrorModules).toEqual(['file.js']);
          expect(errors).toHaveLength(1);
          expect(errors?.[0].message).toMatch('LoaderError');
          builtErrorModules = [];
          index++;
        });
      },
    }),
  ],
});
