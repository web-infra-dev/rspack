const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle() {
		return ["main.mjs"];
	},
	afterExecute(options) {
		const files = fs
			.readdirSync(options.output.path)
			.filter(file => file.endsWith(".mjs"))
			.sort();
		expect(files).toEqual(["main.mjs", "worker-source.mjs"]);

		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);
		expect(source).toMatch(
			/new URL\(\s*["']\.\/worker-source\.mjs["']\s*,\s*import\.meta\.url\s*\)/
		);
		expect(source).not.toContain('from "./worker-source.mjs"');
	},
};
