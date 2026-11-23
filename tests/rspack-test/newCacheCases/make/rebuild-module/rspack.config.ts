import { defineConfig, definePlugin } from '@rspack/cli';
import { NormalModule } from '@rspack/core';

let compilerIndex = 0;
let rebuilding = false;
const loaderOptions = {
  buildCount: 0,
  value: () =>
    rebuilding ? `rebuilt-${compilerIndex}` : `initial-${compilerIndex}`,
};

export default defineConfig({
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [
          {
            loader: './loader.mjs',
            options: loaderOptions,
          },
        ],
      },
    ],
  },
  plugins: [
    definePlugin((compiler) => {
      let rebuilt = false;
      let stillValid = false;

      compiler.hooks.compilation.tap(
        'RebuildModuleCacheTest',
        (compilation) => {
          compilation.hooks.stillValidModule.tap(
            'RebuildModuleCacheTest',
            (module) => {
              if (
                module instanceof NormalModule &&
                module.resource.endsWith('value.js')
              ) {
                stillValid = true;
              }
            },
          );
        },
      );

      compiler.hooks.finishMake.tapAsync(
        'RebuildModuleCacheTest',
        (compilation, callback) => {
          if (rebuilt) {
            return callback();
          }
          rebuilt = true;
          const valueModule = Array.from(compilation.modules).find(
            (module): module is NormalModule =>
              module instanceof NormalModule &&
              module.resource.endsWith('value.js'),
          );
          if (!valueModule) {
            return callback(new Error('value.js module not found'));
          }
          rebuilding = true;
          compilation.rebuildModule(valueModule, (error) => {
            rebuilding = false;
            callback(error);
          });
        },
      );

      compiler.hooks.done.tap('RebuildModuleCacheTest', () => {
        if (compilerIndex === 0) {
          expect(stillValid).toBe(false);
          expect(loaderOptions.buildCount).toBe(2);
        } else {
          expect(stillValid).toBe(true);
          expect(loaderOptions.buildCount).toBe(1);
        }
        compilerIndex++;
        loaderOptions.buildCount = 0;
      });
    }),
  ],
});
