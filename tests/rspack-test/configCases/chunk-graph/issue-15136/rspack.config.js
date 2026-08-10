class ReadChunkGraphDuringFinishModulesPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(
      'ReadChunkGraphDuringFinishModulesPlugin',
      (compilation) => {
        compilation.hooks.finishModules.tap(
          'ReadChunkGraphDuringFinishModulesPlugin',
          () => {
            const module = [...compilation.modules].find((module) =>
              module.identifier().includes('lodash-es/startsWith.js'),
            );
            expect(module).toBeDefined();
            expect(compilation.chunkGraph.getModuleChunks(module)).toEqual([]);
          },
        );
      },
    );
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  mode: 'production',
  module: {
    rules: [
      {
        test: /lodash-es[\\/].*\.js$/,
        loader: 'builtin:swc-loader',
        type: 'javascript/esm',
      },
    ],
  },
  optimization: {
    minimize: false,
  },
  plugins: [new ReadChunkGraphDuringFinishModulesPlugin()],
};
