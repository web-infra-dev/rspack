/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const v0 = stats.stats[0].toJson({ assets: true, hash: true });
		const v1 = stats.stats[1].toJson({ assets: true, hash: true });

		const v0Css = v0.assets.find(a => a.name.endsWith(".css"))?.name;
		const v1Css = v1.assets.find(a => a.name.endsWith(".css"))?.name;

		expect(v0Css).toBeDefined();
		expect(v1Css).toBeDefined();
		expect(v0Css).not.toBe(v1Css);
		expect(v0.hash).not.toBe(v1.hash);
	}
};
