/** @type {import('../../..').THashCaseConfig} */
module.exports = {
	validate(stats) {
		for (let i = 0; i < 4; i += 2) {
			const a = stats.stats[i + 0].toJson({
				assets: true
			});
			const b = stats.stats[i + 1].toJson({
				assets: true
			});
			expect(Object.keys(a.assetsByChunkName).length).toBe(5);
			expect(a.assetsByChunkName.main).toEqual(b.assetsByChunkName.main);
			expect(a.assetsByChunkName.lazy).toEqual(b.assetsByChunkName.lazy);
			expect(a.assetsByChunkName.a).toEqual(b.assetsByChunkName.a);
			expect(a.assetsByChunkName.b).toEqual(b.assetsByChunkName.b);
			// expect(a.assetsByChunkName.a).toEqual(a.assetsByChunkName.b);
		}
	}
};
