const assert = require('node:assert');

const normalizePath = (value) => value.replace(/\\/g, '/').replace(/\/+$/, '');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  plugins: [
    {
      apply(compiler) {
        const originalInputFileSystem = compiler.inputFileSystem;
        const inputFileSystem = Object.create(originalInputFileSystem);
        let visitedUnrelatedDirectory = false;

        inputFileSystem.readdir = (dirPath, callback) => {
          if (normalizePath(dirPath).endsWith('/src/unrelated')) {
            visitedUnrelatedDirectory = true;
          }
          return originalInputFileSystem.readdir(dirPath, callback);
        };

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
            assert.strictEqual(
              visitedUnrelatedDirectory,
              false,
              'case-insensitive glob should not scan unrelated directories',
            );
          },
        );
      },
    },
  ],
  experiments: {
    useInputFileSystem: [/import-meta-glob-case-insensitive-pruning/],
  },
};
