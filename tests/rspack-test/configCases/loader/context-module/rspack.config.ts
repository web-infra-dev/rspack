import { defineConfig, definePlugin } from '@rspack/cli';

import path from 'node:path';
import assert from 'node:assert';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'a.js'),
        sideEffects: false,
        use: [
          {
            loader: './my-loader.mjs',
          },
        ],
      },
    ],
  },
  plugins: [
    definePlugin({
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('MyPlugin', (compilation) => {
          compilation.hooks.processAssets.tap('MyPlugin', () => {
            let hasModule = false;
            for (const chunk of compilation.chunks) {
              const modules = compilation.chunkGraph.getChunkModules(chunk);
              for (const module of modules) {
                if (module.identifier().endsWith('a.js')) {
                  hasModule = true;
                  assert(module.buildInfo.LOADER_ACCESS === true);
                  assert(module.buildMeta.LOADER_ACCESS === true);
                  assert(module.factoryMeta.sideEffectFree === true);
                }
              }
            }
            assert(hasModule);
          });
        });
      },
    }),
  ],
});
