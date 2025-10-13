/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const fullhash = stats.hash;
		const asset = stats.toJson({ assets: true }).children[0].assets[0];
		expect(asset.name).toBe(`${fullhash}.js`);
	}
};
