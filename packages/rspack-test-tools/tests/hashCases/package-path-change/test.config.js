/** @type {import('../../..').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const version0 = stats.stats[0].toJson({ assets: true });
		const version0Copy = stats.stats[1].toJson({ assets: true });
		const version1 = stats.stats[2].toJson({ assets: true });

		expect(version0.assetsByChunkName.main).toEqual(
			version0Copy.assetsByChunkName.main
		);
		expect(version0.assetsByChunkName.runtime).toEqual(
			version0Copy.assetsByChunkName.runtime
		);
		expect(version0.assetsByChunkName.package).toEqual(
			version0Copy.assetsByChunkName.package
		);

		expect(version0.assetsByChunkName.main).not.toEqual(
			version1.assetsByChunkName.main
		);
		expect(version0.assetsByChunkName.runtime).not.toEqual(
			version1.assetsByChunkName.runtime
		);
		expect(version0.assetsByChunkName.package).not.toEqual(
			version1.assetsByChunkName.package
		);
	}
};
