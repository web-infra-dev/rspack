module.exports = {
	validate(stats) {
		const nonRealContentHash = stats.stats[0].toJson({ assets: true });
		for (let i = 1; i < 4; i += 1) {
			const json = stats.stats[i].toJson({ assets: true });
			expect(json.assetsByChunkName.main).toEqual(
				nonRealContentHash.assetsByChunkName.main
			);
		}
		const realContentHash = stats.stats[4].toJson({ assets: true });
		for (let i = 4; i < 8; i += 1) {
			const json = stats.stats[i].toJson({ assets: true });
			expect(json.assetsByChunkName.main).toEqual(
				realContentHash.assetsByChunkName.main
			);
		}
	}
};
