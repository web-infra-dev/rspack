import fs from "node:fs";
import path from "node:path";

export default {
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
				"createFakeNamespaceObject.call(rspackRequire,",
			);
			expect(source).toContain(
				"createFakeNamespaceObject.call(rspackRequire, /*require.resolve*/(",
			);
		} else {
			expect(source).toContain("__webpack_require__.t(");
			expect(source).not.toContain("__webpack_require__.t.call(");
		}
	},
};
