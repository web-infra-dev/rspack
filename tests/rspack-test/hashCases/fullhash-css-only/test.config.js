/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const version0 = stats.stats[0].toJson({ assets: true });
		const version1 = stats.stats[1].toJson({ assets: true });
		const js0 = version0.assets.find(asset => asset.name.endsWith(".js"));
		const js1 = version1.assets.find(asset => asset.name.endsWith(".js"));
		const css0 = version0.assets.find(asset => asset.name.endsWith(".css"));
		const css1 = version1.assets.find(asset => asset.name.endsWith(".css"));

		expect(js0.name).toBe(`main.${version0.hash}.js`);
		expect(js1.name).toBe(`main.${version1.hash}.js`);
		expect(css0.name).not.toBe(css1.name);
		expect(version0.hash).not.toBe(version1.hash);
	}
};
