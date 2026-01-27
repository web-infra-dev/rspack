/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const assets = stats.stats[0].toJson().assets.map(i => i.name);
		assets.sort();
		expect(assets).toEqual([
			"file1.c30068f3cc748ce3.svg",
			"file2.c30068f3cc748ce3.svg",
			"main.js"
		]);
	}
};
