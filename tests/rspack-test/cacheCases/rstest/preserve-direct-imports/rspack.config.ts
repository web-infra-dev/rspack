import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

export function createConfig(moduleCache = false, source?: string) {
  let step = -1;
  let context = '';
  const loaderOptions: { builtModules: string[] } = { builtModules: [] };
  return {
    mode: 'development' as const,
    incremental: false,
    cache: { type: 'persistent' as const },
    experiments: { newCache: { module: moduleCache } },
    optimization: {
      sideEffects: true,
      usedExports: true,
      concatenateModules: false,
    },
    module: {
      rules: [
        {
          test: /untouched\.js$/,
          loader: path.join(source ?? import.meta.dirname, 'loader.mjs'),
          options: loaderOptions,
        },
      ],
    },
    plugins: [
      definePlugin((compiler) => {
        context = source ?? compiler.context;
        compiler.hooks.done.tap('PreservationCache', () => {
          if (moduleCache && step > 0) {
            expect(loaderOptions.builtModules).toEqual([]);
          }
          loaderOptions.builtModules = [];
        });
      }),
      new rspack.experiments.RstestPlugin({
        injectModulePathName: false,
        importMetaPathName: false,
        hoistMockModule: false,
        manualMockRoot: import.meta.dirname,
        preserveSideEffectImports: async () => {
          step++;
          return [path.join(context, step % 2 === 0 ? 'a.js' : 'b.js')];
        },
      }),
    ],
  };
}

export default defineConfig(createConfig());
