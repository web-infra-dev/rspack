import { defineConfig, definePlugin } from '@rspack/cli';
import type { InputFileSystem } from '@rspack/core';

const normalizePath = (value: string) =>
  value.replace(/\\/g, '/').replace(/\/+$/, '');

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        // A persistent-cache snapshot also stats entries under the context.
        const cache = compiler.options.cache;
        if (typeof cache === 'object' && cache.type === 'persistent') return;
        const originalInputFileSystem = compiler.inputFileSystem!;
        const inputFileSystem: InputFileSystem = Object.create(
          originalInputFileSystem,
        );
        let statUnrelatedDirectory = false;

        inputFileSystem.stat = ((
          ...args: Parameters<InputFileSystem['stat']>
        ) => {
          const [dirPath] = args;
          if (normalizePath(dirPath.toString()).endsWith('/src/unrelated')) {
            statUnrelatedDirectory = true;
          }
          return originalInputFileSystem.stat(...args);
        }) as InputFileSystem['stat'];

        compiler.inputFileSystem = inputFileSystem;
        compiler.hooks.beforeCompile.tap('ImportMetaGlobDfaPruning', () => {
          compiler.inputFileSystem = inputFileSystem;
        });
        compiler.hooks.afterCompile.tap('ImportMetaGlobDfaPruning', () => {
          expect(
            statUnrelatedDirectory,
            'a negative-only branch should be pruned before stat',
          ).toBe(false);
        });
      },
    }),
  ],
  experiments: {
    useInputFileSystem: [/import-meta-glob-dfa-pruning/],
  },
});
