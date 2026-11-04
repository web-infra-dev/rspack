import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

let context = '';
let step = 0;

export default defineConfig({
  mode: 'development',
  incremental: true,
  cache: false,
  experiments: { pureFunctions: true },
  optimization: {
    sideEffects: true,
    usedExports: true,
    concatenateModules: false,
    minimize: false,
  },
  plugins: [
    definePlugin((compiler) => {
      context = compiler.context;
      compiler.hooks.done.tap('PreserveTargetDeclarations', (stats) => {
        const compilation = stats.compilation;
        let pureModuleChunks: number | undefined;
        let effectModuleChunks: number | undefined;
        for (const module of compilation.modules) {
          const resource = module.nameForCondition();
          if (resource && path.basename(resource) === 'pure.js') {
            pureModuleChunks = Array.from(
              compilation.chunkGraph.getModuleChunksIterable(module),
            ).length;
          }
          if (resource && path.basename(resource) === 'effect.js') {
            effectModuleChunks = Array.from(
              compilation.chunkGraph.getModuleChunksIterable(module),
            ).length;
          }
        }
        expect(pureModuleChunks).toBe(step === 2 ? 0 : 1);
        expect(effectModuleChunks).toBe(1);
        step++;
      });
    }),
    new rspack.experiments.RstestPlugin({
      injectModulePathName: false,
      importMetaPathName: false,
      hoistMockModule: false,
      manualMockRoot: import.meta.dirname,
      preserveSideEffectImports: async () => [
        path.join(context, 'importer.js'),
      ],
    }),
  ],
});
