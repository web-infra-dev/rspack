/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const first = stats.stats[0].toJson({ assets: true, hash: true });
		expect(first.assetsByChunkName.main).toBeTruthy();
		for (let i = 1; i < 4; i += 1) {
			const current = stats.stats[i].toJson({ assets: true, hash: true });
			expect(current.hash).toEqual(first.hash);
			expect(current.assetsByChunkName.main).toEqual(first.assetsByChunkName.main);
		}
	}
};
