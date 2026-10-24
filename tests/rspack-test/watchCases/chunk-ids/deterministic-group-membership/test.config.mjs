export default {
  checkStats(step, stats) {
    if (step === '2') {
      expect(stats.modules.find(module => module.name === './p.js').codeGenerated).toBe(true);
    }
    return true;
  },
};
