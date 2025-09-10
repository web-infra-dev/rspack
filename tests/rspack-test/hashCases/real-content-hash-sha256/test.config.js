// const crypto = require("node:crypto");

/** @type {import("@rspack/test-tools").THashCaseConfig} */
module.exports = {
	validate(stats) {
		const assets = stats.stats[0].toJson().assets.map(i => i.name);
		assets.sort();

		// `04c785ed39b3beedfad58de4438c4e905d1da406dc6a1b9ed043ebc574819baa` = sha256 of src/file.svg
		expect(assets).toEqual([
			"04c785ed39b3beedfad58de4438c4e905d1da406dc6a1b9ed043ebc574819baa.svg",
			"main.js"
		]);
	}
};
