'use strict';

const assert = require('node:assert/strict');
const { pathToFileURL } = require('node:url');
const core = require('@rspack/core');

let duplicateCoreId = 0;

class DuplicateCoreInstancesPlugin {
  apply(compiler) {
    compiler.hooks.beforeRun.tapPromise(
      'DuplicateCoreInstancesPlugin',
      async () => {
        const duplicateCore = await import(
          `${pathToFileURL(require.resolve('@rspack/core')).href}?duplicate-core-instance=${duplicateCoreId++}`
        );

        assert.notStrictEqual(
          core,
          duplicateCore,
          'JavaScript core should be evaluated twice',
        );
        assert.notStrictEqual(
          core.Compilation,
          duplicateCore.Compilation,
          'Compilation should come from different JavaScript core instances',
        );
        assert.strictEqual(
          core.ExternalModule,
          duplicateCore.ExternalModule,
          'ExternalModule should come from the same native binding',
        );
      },
    );
  }
}

class ExternalModuleChunkConditionPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(
      'ExternalModuleChunkConditionPlugin',
      (compilation) => {
        core.ExternalModule.getCompilationHooks(compilation).chunkCondition.tap(
          'ExternalModuleChunkConditionPlugin',
          (chunk, compilation) =>
            compilation.chunkGraph.getNumberOfEntryModules(chunk) > 0,
        );
      },
    );
  }
}

module.exports = {
  externals: { external: 'fs' },
  externalsType: 'module-import',
  output: {
    module: true,
    chunkFormat: 'module',
    chunkFilename: '[name].mjs',
  },
  optimization: {
    moduleIds: 'named',
    concatenateModules: false,
  },
  plugins: [
    new DuplicateCoreInstancesPlugin(),
    new ExternalModuleChunkConditionPlugin(),
  ],
};
