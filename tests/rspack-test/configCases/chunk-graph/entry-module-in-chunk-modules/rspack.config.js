class Plugin {
  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const chunk = Array.from(compilation.chunks).find(
          (chunk) => chunk.name === 'main',
        );
        const entryModules = [
          ...compilation.chunkGraph.getChunkEntryModulesIterable(chunk),
        ];
        const modules = new Set(compilation.chunkGraph.getChunkModules(chunk));

        expect(entryModules).toHaveLength(1);
        expect(modules.has(entryModules[0])).toBe(true);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  node: false,
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    sideEffects: false,
  },
  plugins: [new Plugin()],
};
