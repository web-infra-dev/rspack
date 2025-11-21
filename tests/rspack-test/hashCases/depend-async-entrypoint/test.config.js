/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const a = stats.stats[0].toJson({ assets: true });
		const b = stats.stats[1].toJson({ assets: true });
		expect(a.assetsByChunkName.main).not.toEqual(b.assetsByChunkName.main);
		expect(a.assetsByChunkName.worker).not.toEqual(b.assetsByChunkName.worker);
	}
};
