export default {
  checkStats(step, stats, text) {
    if (step === '1') {
      expect(text).toMatch(/LOG from rspack\.incremental\.modulesHashes/);
      expect(text).toMatch(/LOG from rspack\.incremental\.modulesCodegen[\s\S]*?1 modules are affected, 3 in total/);
      expect(stats.modules.find(module => module.name === './stable.js').codeGenerated).toBe(false);
      expect(stats.warnings).not.toEqual(expect.arrayContaining([
        expect.objectContaining({ message: expect.stringMatching(/DeterministicChunkIdsPlugin.*fallback/) }),
      ]));
    }
    return true;
  },
};
