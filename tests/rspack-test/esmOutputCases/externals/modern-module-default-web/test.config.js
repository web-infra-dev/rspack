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

		expect(source).toContain('import { resolve } from "path";');
		expect(source).toContain('import("os")');
		expect(source).toContain('require("fs")');
		expect(source).not.toContain("createRequire");
	}
};
