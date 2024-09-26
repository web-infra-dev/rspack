/** @type {import('../../..').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const assets = stats.stats[0].toJson().assets.map(i => i.name);
		assets.sort();
		expect(assets).toEqual([
			"file1.afc10c70ed4ce2b33593.svg",
			"file2.afc10c70ed4ce2b33593.svg",
			"main.js"
		]);
	}
};
