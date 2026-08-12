'use strict';

const { pathToFileURL } = require('node:url');
const { ExternalModule } = require('@rspack/core');

class LoadDuplicateCorePlugin {
  apply(compiler) {
    compiler.hooks.beforeRun.tapPromise('LoadDuplicateCorePlugin', async () => {
      await import(
        `${pathToFileURL(require.resolve('@rspack/core')).href}?duplicate-core-instance`
      );
    });
  }
}

class ExternalModuleChunkConditionPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(
      'ExternalModuleChunkConditionPlugin',
      (compilation) => {
        ExternalModule.getCompilationHooks(compilation).chunkCondition.tap(
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
  experiments: {
    outputModule: true,
  },
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
    new LoadDuplicateCorePlugin(),
    new ExternalModuleChunkConditionPlugin(),
  ],
};
