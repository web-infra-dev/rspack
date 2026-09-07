const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toContain("file://");
		expect(source).toMatch(/createRequire\)?\(['"]file:\/\//);
		expect(source).toContain("/* createRequire() */ undefined");
		expect(source).toContain("__webpack_require__(");
		expect(source).toContain("/*require.resolve*/");
	}
};
