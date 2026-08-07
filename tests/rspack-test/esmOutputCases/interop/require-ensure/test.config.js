const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8",
		);
		expect(source).toContain(
			".bind(null, function __rspack_static_require__(request)",
		);
		expect(source).not.toContain("__rspack_module_relocation_");
		expect(source).not.toMatch(
			/(?:__webpack_require__|rspackRequire)\("\.\/value\.js"\)/,
		);
	},
};
