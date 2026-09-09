module.exports = {
  validate(stats) {
    expect(stats.stats.map(child => child.compilation.warnings.length)).toEqual([
      1, 1, 0, 0, 0, 1, 1, 0,
    ]);
    expect(stats.stats[5].compilation.options.mode).toBeUndefined();
    expect(stats.stats[6].compilation.options.mode).toBeUndefined();
  },
};
