import fs from "node:fs";
import path from "node:path";

function readOutput(options) {
	return fs
		.readdirSync(options.output.path)
		.filter(file => file.endsWith(".mjs"))
		.map(file => fs.readFileSync(path.join(options.output.path, file), "utf-8"))
		.join("\n");
}

export default {
	findBundle() {
		return [];
	},
	snapshotFileFilter() {
		return false;
	},
	afterExecute(options) {
		const source = readOutput(options);

		expect(source).not.toContain('export * from "fs"');
		expect(source).toContain('require("fs")');
	}
};
