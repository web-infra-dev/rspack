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

		expect(source).toMatch(
			/const\s+external\s*=\s*require\s*\(\s*["']external["']\s*\)/
		);
		expect(source).toMatch(
			/module\.exports\s*=\s*require\s*\(\s*["']external["']\s*\)/
		);
		expect(source).not.toContain('external "external"');
		expect(source).not.toMatch(
			/(?:__webpack_require__|__rspack_context\.r)\s*\(\s*[^)]*["']external["']/
		);
	}
};
