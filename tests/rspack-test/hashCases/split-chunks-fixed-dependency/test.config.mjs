/** @type {import('@rspack/test-tools').THashCaseConfig} */
export default {
  validate(stats) {
    const [first, second] = stats.stats.map((child) => {
      expect(child.hasErrors()).toBe(false);
      const { compilation } = child;
      const group = compilation.namedChunkGroups.get('lazy');
      const sharedModuleCounts = group.chunks
        .filter((chunk) => !chunk.name)
        .map((chunk) => compilation.chunkGraph.getChunkModules(chunk).length);
      expect(sharedModuleCounts).toEqual([1, 1]);
      const chunks = group.chunks.map((chunk) => chunk.id);
      expect(chunks).toHaveLength(3);
      return { chunks, files: compilation.entrypoints.get('main').getFiles() };
    });
    expect(second).toEqual(first);
  },
};
