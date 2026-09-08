module.exports = {
  validate(stats) {
    expect(stats.stats.map(child => child.compilation.warnings.length)).toEqual([
      1, 1, 0, 0, 0,
    ]);
  },
};
