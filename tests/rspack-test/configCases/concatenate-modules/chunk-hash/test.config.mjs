export default {
  findBundle(index) {
    return ["bundle0.js", "bundle1.js"][index];
  },
  validate(stats) {
    const chunkAssets = stats.stats.map(({ compilation }) => {
      const assets = compilation.getAssets().filter(({ name }) =>
        name.startsWith("chunk.")
      );
      expect(assets).toHaveLength(1);
      return assets[0].name;
    });
    expect(chunkAssets).toHaveLength(2);
    expect(chunkAssets[0]).not.toBe(chunkAssets[1]);
  },
};
