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

		expect(source).toMatch(/import\s*\{\s*resolve\s*\}\s*from\s*["']path["']/);
		expect(source).toMatch(/import\s*\(\s*["']os["']\s*\)/);
		expect(source).toMatch(/from\s*["']fs["']/);
		expect(source).not.toContain("createRequire");
	}
};
