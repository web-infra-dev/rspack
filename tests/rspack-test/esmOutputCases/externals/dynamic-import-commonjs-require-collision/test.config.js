const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toContain("user require should not be called");
		expect(source).toContain("async function loadPlatform(require)");
		expect(source).toContain('import("node:module").then');
		expect(source).not.toContain('import.meta.__customImport__("node:module")');
		expect(source).toContain('module.createRequire(import.meta.url)("os")');
		expect(source).not.toContain('__webpack_require__.t(require("os"), 22)');
	}
};
