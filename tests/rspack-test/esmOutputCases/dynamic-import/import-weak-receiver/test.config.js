const fs = require("fs");
const path = require("path");

module.exports = {
	snapshotFileFilter() {
		return false;
	},
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8",
		);

		if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
			expect(source).toContain("createFakeNamespaceObject(m, 22)");
			expect(source).toContain("createFakeNamespaceObject(((() => require_value())");
		} else {
			expect(source).toContain("__webpack_require__.t(m, 22)");
			expect(source).toContain("__webpack_require__.t(((() => require_value())");
		}
		expect(source).toContain("(1) & ~1");
		expect(source).not.toContain(".call(rspackRequire");
		expect(source).not.toContain("moduleFactories");
		expect(source).not.toContain("__webpack_require__.m[");
	},
};
