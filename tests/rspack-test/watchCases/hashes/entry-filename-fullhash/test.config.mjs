import fs from "node:fs";

let outputPath;
let initialHash;

export default {
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
