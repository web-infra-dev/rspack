import { ExternalModule } from '@rspack/core';

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

export default {
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
  plugins: [new ExternalModuleChunkConditionPlugin()],
};
