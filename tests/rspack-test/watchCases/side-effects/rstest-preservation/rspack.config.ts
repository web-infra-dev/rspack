import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';
import path from 'node:path';

const source = path.resolve(
  import.meta.dirname,
  '../../../cacheCases/rstest/preserve-direct-imports',
);

let step = -1;
export default defineConfig({
  mode: 'development' as const,
  incremental: true,
  cache: false,
  resolve: {
    alias: {
      'importer-a': path.join(source, 'a.js'),
      'importer-b': path.join(source, 'b.js'),
      untouched: path.join(source, 'untouched.js'),
      'preservation-state': path.join(source, 'state.js'),
    },
  },
  optimization: {
    sideEffects: true,
    usedExports: true,
    concatenateModules: false,
  },
  plugins: [
    new rspack.experiments.RstestPlugin({
      injectModulePathName: false,
      importMetaPathName: false,
      hoistMockModule: false,
      manualMockRoot: import.meta.dirname,
      preserveSideEffectImports: async () => {
        step++;
        return [path.join(source, step % 2 === 0 ? 'a.js' : 'b.js')];
      },
    }),
    definePlugin((compiler) => {
      let built: string[] = [];
      compiler.hooks.compilation.tap('PreservationWatch', (compilation) => {
        built = [];
        compilation.hooks.buildModule.tap('PreservationWatch', (module) => {
          built.push(module.identifier());
        });
      });
      compiler.hooks.done.tap('PreservationWatch', () => {
        if (step > 0) {
          expect(built.some((id) => id.endsWith('untouched.js'))).toBe(false);
        }
      });
    }),
  ],
});
