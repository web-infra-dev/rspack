const { RuntimeModule } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  target: 'node',
  entry: './index.js',
  mode: 'development',
  devtool: false,
  output: {
    filename: '[name].js',
    chunkFilename: 'chunks/[name].[contenthash:8].js',
  },
  optimization: {
    minimize: false,
    sideEffects: false,
    concatenateModules: false,
    usedExports: false,
    innerGraph: false,
    providedExports: false,
    chunkIds: 'named',
    runtimeChunk: {
      name: 'runtime',
    },
  },
  plugins: [
    (compiler) => {
      const RuntimeGlobals = compiler.rspack.RuntimeGlobals;

      class DependentHashSourceReuseRuntimeModule extends RuntimeModule {
        constructor() {
          super('dependent-hash-source-reuse');
          this.dependentHash = true;
        }

        generate() {
          const chunkIdToName = this.chunk.getChunkMaps(false).name;
          const chunkNameToId = Object.fromEntries(
            Object.entries(chunkIdToName).map(([chunkId, chunkName]) => [
              chunkName,
              chunkId,
            ]),
          );

          return `${RuntimeGlobals.require}.dependentHashSourceReuse = function(chunkName) {
	return ${RuntimeGlobals.getChunkScriptFilename}((${JSON.stringify(
    chunkNameToId,
  )})[chunkName] || chunkName);
};`;
        }
      }

      compiler.hooks.thisCompilation.tap(
        'DependentHashSourceReuseRuntimeModulePlugin',
        (compilation) => {
          compilation.hooks.additionalTreeRuntimeRequirements.tap(
            'DependentHashSourceReuseRuntimeModulePlugin',
            (chunk, set) => {
              set.add(RuntimeGlobals.getChunkScriptFilename);
              compilation.addRuntimeModule(
                chunk,
                new DependentHashSourceReuseRuntimeModule(),
              );
            },
          );
        },
      );
    },
  ],
};
