import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

const config = (
  sideEffects: boolean,
  concatenateModules: boolean,
): Configuration => ({
  name: `side-effects-${sideEffects}-concatenation-${concatenateModules}`,
  mode: 'production',
  optimization: {
    sideEffects,
    providedExports: true,
    usedExports: true,
    concatenateModules,
    minimize: false,
  },
  plugins: [
    definePlugin((compiler) => {
      compiler.hooks.thisCompilation.tap('Test', (compilation) => {
        let modules: string[] = [];
        compilation.hooks.finishModules.tap('Test', (finished) => {
          modules = [];
          for (const module of finished) {
            const resource = (module as { resource?: string }).resource;
            if (resource) modules.push(path.basename(resource));
          }
        });
        compilation.hooks.processAssets.tap('Test', () => {
          compilation.emitAsset(
            `${compiler.options.name}.modules.json`,
            new compiler.webpack.sources.RawSource(
              JSON.stringify({ sideEffects, modules: modules.sort() }),
            ),
          );
        });
      });
    }),
  ],
});

export default defineConfig([
  config(true, false),
  config(true, true),
  config(false, false),
]);
