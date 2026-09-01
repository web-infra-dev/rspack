module.exports = {
  snapshotFileFilter() {
    return false;
  },
  findBundle(_index, options) {
    return `${options.name}/main.mjs`;
  },
  validate(stats) {
    const json = stats.toJson({
      all: false,
      chunks: true,
      chunkModules: true,
      ids: true,
    });
    const compilations = json.children || [json];

    for (const compilation of compilations) {
      const facadeChunks = compilation.chunks.filter(
        chunk => chunk.entry && !chunk.initial && chunk.modules.length === 0,
      );
      const facadeIds = facadeChunks.map(chunk => chunk.id);
      const ordinaryEmptyChunks = compilation.chunks.filter(
        chunk => !chunk.entry && !chunk.initial && chunk.modules.length === 0,
      );

      expect(facadeChunks).toHaveLength(14);
      expect(facadeIds.every(id => id !== null)).toBe(true);
      expect(new Set(facadeIds).size).toBe(facadeIds.length);
      expect(ordinaryEmptyChunks).toHaveLength(0);
    }
  },
};
