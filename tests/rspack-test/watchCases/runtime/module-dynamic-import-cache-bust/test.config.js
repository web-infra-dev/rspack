const fs = require("fs");
const path = require("path");

let outputPath = "";

module.exports = {
	findBundle(i, config) {
		outputPath = config.output.path;
		return [];
	},
	checkStats() {
		const source = fs.readFileSync(path.join(outputPath, "runtime~main.mjs"), "utf-8");
		expect(source).toContain("?t=");
		expect(source).toContain("chunkId");
		expect(source).not.toContain("Date.now");
		return true;
	}
};
