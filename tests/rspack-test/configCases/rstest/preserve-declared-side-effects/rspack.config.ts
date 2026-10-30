import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';
import { join } from 'node:path';

export default defineConfig(
  [false, true].flatMap((preserve) =>
    [false, true].map((concatenateModules) => ({
      mode: 'production' as const,
      experiments: { pureFunctions: true },
      module: {
        rules: [
          { test: /importer\.js$/, sideEffects: true },
          { test: /rule-free\.js$/, sideEffects: false },
        ],
      },
      optimization: {
        sideEffects: true,
        usedExports: true,
        concatenateModules,
        minimize: false,
      },
      plugins: [
        new rspack.DefinePlugin({ PRESERVE: JSON.stringify(preserve) }),
        new rspack.experiments.VirtualModulesPlugin({
          [join(
            import.meta.dirname,
            'node_modules/registered-package/package.json',
          )]:
            '{"name":"registered-package","sideEffects":false,"main":"index.js"}',
          [join(
            import.meta.dirname,
            'node_modules/registered-package/index.js',
          )]: 'import { trace } from "../../state.js"; trace.push("package");',
        }),
        new rspack.experiments.RstestPlugin({
          injectModulePathName: false,
          importMetaPathName: false,
          hoistMockModule: false,
          manualMockRoot: import.meta.dirname,
          preserveSideEffectImports: async () =>
            preserve ? [join(import.meta.dirname, 'importer.js')] : [],
        }),
        definePlugin((compiler) => {
          compiler.hooks.done.tap('PreserveDeclaredSideEffects', (stats) => {
            const compilation = stats.compilation;
            let pureModuleChunks: number | undefined;
            for (const module of compilation.modules) {
              if (
                module.nameForCondition() ===
                join(import.meta.dirname, 'pure.js')
              ) {
                pureModuleChunks = Array.from(
                  compilation.chunkGraph.getModuleChunksIterable(module),
                ).length;
              }
            }
            expect(pureModuleChunks).toBe(0);
          });
        }),
      ],
    })),
  ),
);
