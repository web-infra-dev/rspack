// const crypto = require("node:crypto");

/** @type {import('../../../dist').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const assets = stats.stats[0].toJson().assets.map(i => i.name);
		assets.sort();

		// `c30068f3cc748ce3` = xxhash64 of src/file.svg
		expect(assets).toEqual(['c30068f3cc748ce3.svg', 'main.js']);
	}
};
