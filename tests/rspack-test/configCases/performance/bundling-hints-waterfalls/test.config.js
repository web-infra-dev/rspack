module.exports = {
  validate(stats) {
    const warnings = stats.stats.map(child =>
      child.toJson({ all: false, warnings: true }).warnings,
    );
    expect(warnings.map(child => child.length)).toEqual([1, 1, 0]);
    const chains = warnings.slice(0, 2).map(child =>
      child[0].message.split('\n').filter(line => line.includes(' -> ')),
    );
    expect(chains.map(child => child.length)).toEqual([1, 1]);
    expect(chains[0][0]).toMatch(/a -> b -> shared \(/);
    expect(chains[1][0]).toMatch(/a -> b -> shared -> leaf \(/);
  },
};
