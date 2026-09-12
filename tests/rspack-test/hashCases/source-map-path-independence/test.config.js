/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
  validate(stats) {
    expect(stats.hasErrors()).toBe(false);
    const [first, relocated, changed] = stats.stats.map((child) =>
      child.toJson({ assets: true }),
    );
    expect(first.assetsByChunkName.main).toBeTruthy();
    expect(relocated.assetsByChunkName.main).toEqual(first.assetsByChunkName.main);
    expect(changed.assetsByChunkName.main).not.toEqual(first.assetsByChunkName.main);
  },
};
