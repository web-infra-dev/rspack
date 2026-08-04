const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8",
		);
		expect(source).not.toContain("__webpack_modules__");
		expect(source).not.toContain("__webpack_module_cache__");
		expect(source).not.toContain("__webpack_require__(id)");
		expect(source).toContain("function __rspack_context_load(id)");
		expect(source).toContain('case "./modules/a.js": return require_a();');
		expect(source).toContain("return __rspack_context_load(id)");
		expect(source).toContain("return map[req]");
	},
};
