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
				"Promise.resolve(__rspack_context_load(id)).then(value => createFakeNamespaceObject(value, (7) & ~1))",
			);
		} else {
			expect(source).toContain(
				"Promise.resolve(__rspack_context_load(id)).then(value => __webpack_require__.t(value, (7) & ~1))",
			);
		}
		expect(source).not.toMatch(/(?:__webpack_require__|rspackRequire)\([^.)]/);
	},
};
