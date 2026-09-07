const fs = require("fs");
const path = require("path");

module.exports = {
	snapshotFileFilter() {
		return false;
	},
	afterExecute(options) {
		if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) return;

		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);
		expect(source).toContain("context: Object.create(rspackRequire)");
	}
};
