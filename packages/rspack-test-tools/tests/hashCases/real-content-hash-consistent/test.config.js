module.exports = {
	validate(stats) {
		const a = stats.stats[0].toJson({
			assets: true
		});
		const b = stats.stats[1].toJson({
			assets: true
		});
		expect(a.assetsByChunkName.main).toEqual(b.assetsByChunkName.main);
	}
};
