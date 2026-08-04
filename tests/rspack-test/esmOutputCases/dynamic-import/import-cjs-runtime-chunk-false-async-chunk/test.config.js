const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const readAsset = file =>
			fs.readFileSync(path.join(options.output.path, file), "utf-8");

		const entry = readAsset("main.mjs");
		const dynamic = readAsset("dynamic.mjs");
		const runtime = readAsset("runtime.mjs");

		expect(fs.existsSync(path.join(options.output.path, "index_js.mjs"))).toBe(
			false,
		);
		if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
			expect(entry).toContain(
				'import { createFakeNamespaceObject } from "./runtime.mjs";'
			);
			expect(entry).toContain(
				'createFakeNamespaceObject(module.createRequire(import.meta.url)("node:stream"), 22)'
			);
			expect(runtime).not.toContain("rspackRequire");
			expect(runtime).toContain("createFakeNamespaceObject");
			expect(runtime).not.toContain("moduleFactories");
			expect(entry).not.toContain("export { rspackRequire");
			expect(entry).not.toContain("as rspackRequire");
			expect(dynamic).toContain(
				'import { compatGetDefaultExport } from "./runtime.mjs";'
			);
			expect(dynamic).toContain("require_shared()");
		} else {
			expect(entry).toContain(
				'import { __webpack_require__ } from "./runtime.mjs";'
			);
			expect(entry).toContain(
				'__webpack_require__.t(module.createRequire(import.meta.url)("node:stream"), 22)'
			);
			expect(runtime).toContain("export { __webpack_require__");
			expect(entry).not.toContain("export { __webpack_require__");
			expect(entry).not.toContain("as __webpack_require__");
			expect(dynamic).toContain(
				'import { __webpack_require__ } from "./runtime.mjs";'
			);
			expect(runtime).not.toContain("__webpack_modules__");
			expect(runtime).not.toContain("__webpack_module_cache__");
		}
		expect(entry).toContain("Promise.all");
		expect(entry).toContain('import("./dynamic.mjs")');
		expect(dynamic).not.toContain('from "./main.mjs"');
	},
};
