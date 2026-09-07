const fs = require("node:fs");

let outputPath;
let initialHash;

module.exports = {
	findBundle(_, config) {
		outputPath = config.output.path;
		return [];
	},
	checkStats(step, stats) {
		if (step === "0") {
			initialHash = stats.hash;
		} else {
			expect(stats.hash).not.toBe(initialHash);
			expect(fs.readdirSync(outputPath)).toContain(`first.${stats.hash}.js`);
		}
		return true;
	}
};
