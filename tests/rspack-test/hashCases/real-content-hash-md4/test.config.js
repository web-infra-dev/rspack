// const crypto = require("node:crypto");

/** @type {import('@rspack/test-tools').THashCaseConfig} */
module.exports = {
	validate(stats) {
		const assets = stats.stats[0].toJson().assets.map(i => i.name);
		assets.sort();
		// `afc10c70ed4ce2b33593` = md4 of src/file.svg
		expect(assets).toEqual(["afc10c70ed4ce2b33593.svg", "main.js"]);
	}
};
