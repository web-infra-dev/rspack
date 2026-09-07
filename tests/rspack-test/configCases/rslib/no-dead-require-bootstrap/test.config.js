const fs = require("fs");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(options.output.path + "/bundle0.js", "utf-8");

		expect(source).not.toContain("function __rspack_require");
		expect(source).not.toContain("var __rspack_module_cache");
		expect(source).not.toContain("__rspack_modules[moduleId]");
	}
};
