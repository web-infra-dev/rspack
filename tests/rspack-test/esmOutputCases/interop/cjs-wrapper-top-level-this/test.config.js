const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8",
		);

		expect(source).toContain("var __cjs = (factory, module)");
		expect(source).not.toContain("__commonJS");
		expect(source).not.toContain("module.id");
		expect(source).not.toContain("module.loaded");
		expect(source).not.toContain("module.isEntry");
		expect(source).not.toContain("execOptions");
	},
};
