class Plugin {
  /**
   * @param {"number" | "string"} expectedType
   * @param {string | undefined} expectedId
   */
  constructor(expectedType, expectedId) {
    this.expectedType = expectedType;
    this.expectedId = expectedId;
  }

  /**
   * @param {import("@rspack/core").Compiler} compiler
   */
  apply(compiler) {
    compiler.hooks.compilation.tap('Test', (compilation) => {
      compilation.hooks.processAssets.tap('Test', () => {
        const [chunk] = compilation.chunks;
        expect(typeof chunk.id).toBe(this.expectedType);
        if (this.expectedId !== undefined) {
          expect(chunk.id).toBe(this.expectedId);
        }
        expect(chunk.ids).toStrictEqual([chunk.id]);

        const stats = compilation.getStats().toJson({
          chunks: true,
          ids: true,
          modules: true,
        });
        expect(stats.chunks[0].id).toBe(chunk.id);
        expect(stats.modules[0].chunks).toStrictEqual([chunk.id]);
      });
    });
  }
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    name: 'named-numeric',
    entry: {
      123: './index.js',
    },
    optimization: {
      chunkIds: 'named',
    },
    plugins: [new Plugin('string', '123')],
  },
  {
    name: 'deterministic-numeric',
    entry: {
      main: './index.js',
    },
    optimization: {
      chunkIds: 'deterministic',
    },
    plugins: [new Plugin('number')],
  },
];
