const fs = require("fs");
const path = require("path");

let outputPath;

module.exports = {
	findBundle(_index, options) {
		outputPath = options.output.path;
		return [];
	},
	checkStats(step, stats) {
		const filename = step === "0" ? "first.js" : "deep/nested/first.js";
		expect(stats.assets.some(asset => asset.name === filename)).toBe(true);

		const source = fs.readFileSync(path.join(outputPath, filename), "utf-8");
		const assetUrlMatch = source.match(
			/new\s+URL\s*\(\s*(["'])([^"']+\.txt)\1/
		);
		expect(
			assetUrlMatch,
			`expected ${filename} to contain a rendered .txt asset URL`
		).not.toBeNull();
		const assetUrl = assetUrlMatch[2];
		expect(assetUrl).toMatch(step === "0" ? /^\.\// : /^\.\.\/\.\.\//);
		return true;
	}
};
