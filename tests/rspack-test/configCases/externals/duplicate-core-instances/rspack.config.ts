import type { Compiler } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import assert from 'node:assert/strict';
import { rspack as core } from '@rspack/core';

const getExternalModuleCompilationHooks =
  core.ExternalModule.getCompilationHooks;

let duplicateCoreId = 0;
let duplicateGetCompilationHooks = getExternalModuleCompilationHooks;
let duplicateModuleSize = core.Module.prototype.size;

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
        assert.notStrictEqual(
          core.ExternalModule,
          duplicateCore.ExternalModule,
          'Module classes belong to their JavaScript core instance',
        );
        assert.strictEqual(
          Object.getPrototypeOf(duplicateCore.ExternalModule.prototype),
          duplicateCore.Module.prototype,
        );
        duplicateGetCompilationHooks =
          duplicateCore.ExternalModule.getCompilationHooks;
        duplicateModuleSize = duplicateCore.Module.prototype.size;
      },
    );
  }
}

class ExternalModuleChunkConditionPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'ExternalModuleChunkConditionPlugin',
      (compilation) => {
        assert.strictEqual(
          duplicateGetCompilationHooks(compilation),
          getExternalModuleCompilationHooks(compilation),
          'ExternalModule hooks should be shared across core instances',
        );
        compilation.hooks.finishModules.tap(
          'DuplicateModuleMethods',
          (modules) => {
            for (const module of modules) {
              assert.strictEqual(
                duplicateModuleSize.call(module),
                module.size(),
              );
            }
          },
        );
        duplicateGetCompilationHooks(compilation).chunkCondition.tap(
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
