const fs = require("fs");
const path = require("path");
let outputPath = "";

module.exports = {
	findBundle(i, config) {
		outputPath = config.output.path;
		return [];
	},
	checkStats(i, stats) {
		const assets = stats.assets;
		const main = assets.find(i => i.name.startsWith("main."));
		const runtimeMain = assets.find(i => i.name.startsWith("runtime~main."));
		const entry2 = assets.find(i => i.name.startsWith("entry2."));
		const runtimeEntry2 = assets.find(i =>
			i.name.startsWith("runtime~entry2.")
		);

		const mainContent = fs.readFileSync(
			path.join(outputPath, main.name),
			"utf-8"
		);
		const entry2Content = fs.readFileSync(
			path.join(outputPath, entry2.name),
			"utf-8"
		);

		expect(mainContent.includes(runtimeMain.name)).toBe(true);
		expect(entry2Content.includes(runtimeEntry2.name)).toBe(true);

		return true;
	}
};
