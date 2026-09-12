const warnings = require('../bundling-hints/warnings');

module.exports = {
  validate(stats) {
    for (const [index, child] of stats.stats.entries()) {
      const actual = child.toJson({ all: false, warnings: true }).warnings;
      expect(actual).toHaveLength(1);
      expect(actual[0].message).toMatch(warnings[index]);
    }
  },
};
