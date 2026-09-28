import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import assert from 'node:assert/strict';
import { rspack as core } from '@rspack/core';

const getExternalModuleCompilationHooks =
  core.ExternalModule.getCompilationHooks;

let duplicateCoreId = 0;

class DuplicateCoreInstancesPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.beforeRun.tapPromise(
      'DuplicateCoreInstancesPlugin',
      async () => {
        const duplicateCore = await import(
          `${import.meta.resolve('@rspack/core')}?duplicate-core-instance=${duplicateCoreId++}`
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
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'ExternalModuleChunkConditionPlugin',
      (compilation) => {
        getExternalModuleCompilationHooks(compilation).chunkCondition.tap(
          'ExternalModuleChunkConditionPlugin',
          (chunk, compilation) =>
            compilation.chunkGraph.getNumberOfEntryModules(chunk) > 0,
        );
      },
    );
  }
}

export default defineConfig({
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
});
