import { defineConfig, definePlugin } from '@rspack/cli';
import type { InputFileSystem } from '@rspack/core';

const normalizePath = (value: string) =>
  value.replace(/\\/g, '/').replace(/\/+$/, '');

export default defineConfig({
  plugins: [
    definePlugin({
      apply(compiler) {
        const originalInputFileSystem = compiler.inputFileSystem!;
        const inputFileSystem: InputFileSystem = Object.create(
          originalInputFileSystem,
        );
        let visitedUnrelatedDirectory = false;

        inputFileSystem.readdir = ((
          ...args: Parameters<InputFileSystem['readdir']>
        ) => {
          const [dirPath] = args;
          if (normalizePath(dirPath.toString()).endsWith('/src/unrelated')) {
            visitedUnrelatedDirectory = true;
          }
          return originalInputFileSystem.readdir(...args);
        }) as InputFileSystem['readdir'];

        compiler.inputFileSystem = inputFileSystem;
        compiler.hooks.beforeCompile.tap(
          'ImportMetaGlobCaseInsensitivePruning',
          () => {
            compiler.inputFileSystem = inputFileSystem;
          },
        );
        compiler.hooks.afterCompile.tap(
          'ImportMetaGlobCaseInsensitivePruning',
          () => {
            expect(
              visitedUnrelatedDirectory,
              'case-insensitive glob should not scan unrelated directories',
            ).toBe(false);
          },
        );
      },
    }),
  ],
  experiments: {
    useInputFileSystem: [/import-meta-glob-case-insensitive-pruning/],
  },
});
