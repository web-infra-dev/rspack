import { defineConfig } from '@rspack/cli';
import { type Compiler, ExternalModule } from '@rspack/core';

class ExternalModuleChunkConditionPlugin {
  apply(compiler: Compiler) {
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
  plugins: [new ExternalModuleChunkConditionPlugin()],
});
