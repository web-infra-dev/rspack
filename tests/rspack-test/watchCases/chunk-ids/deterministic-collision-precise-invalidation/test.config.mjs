export default {
  checkStats(step, stats) {
    if (step === '1') {
      expect(stats.modules.find(module => module.name === './stable.js').codeGenerated).toBe(false);
      expect(stats.modules.find(module => module.name === './index.js').codeGenerated).toBe(true);
    }
    return true;
  },
};
