const fs = require("fs");
const path = require("path");

function readOutput(options) {
	return fs
		.readdirSync(options.output.path)
		.filter(file => file.endsWith(".mjs"))
		.map(file => fs.readFileSync(path.join(options.output.path, file), "utf-8"))
		.join("\n");
}

module.exports = {
	findBundle() {
		return [];
	},
	snapshotFileFilter() {
		return false;
	},
	afterExecute(options) {
		const source = readOutput(options);

		expect(source).toMatch(/import\s*\{\s*resolve\s*\}\s*from\s*["']path["']/);
		expect(source).toMatch(/import\s*\(\s*["']os["']\s*\)/);
		expect(source).toMatch(/require\s*\(\s*["']fs["']\s*\)/);
		expect(source).not.toContain("createRequire");
	}
};
