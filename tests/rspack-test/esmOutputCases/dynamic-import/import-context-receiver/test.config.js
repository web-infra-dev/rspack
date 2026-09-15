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
			expect(source).toContain(
				"createFakeNamespaceObject.call(rspackRequire, id, 7 | 16)",
			);
		} else {
			expect(source).toContain("__webpack_require__.t(id, 7 | 16)");
		}
	},
};
