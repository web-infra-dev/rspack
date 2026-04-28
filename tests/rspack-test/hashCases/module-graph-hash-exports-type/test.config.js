/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const first = stats.stats[0].toJson({ assets: true, hash: true });
		expect(first.assetsByChunkName.loose).toBeTruthy();
		expect(first.assetsByChunkName.strict).toBeTruthy();
		expect(first.assetsByChunkName.runtime).toBeTruthy();
		for (let i = 1; i < 8; i += 1) {
			const current = stats.stats[i].toJson({ assets: true, hash: true });
			expect(current.hash).toEqual(first.hash);
			expect(current.assetsByChunkName.loose).toEqual(first.assetsByChunkName.loose);
			expect(current.assetsByChunkName.runtime).toEqual(first.assetsByChunkName.runtime);
		}
	}
};
