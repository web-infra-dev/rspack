const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		if (!globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
			return;
		}

		const readAsset = file =>
			fs.readFileSync(path.join(options.output.path, file), "utf-8");

		const entry = readAsset("index_js.mjs");
		const dynamic = readAsset("dynamic.mjs");

		expect(dynamic).not.toContain("__rspack_context");
		expect(dynamic).not.toContain("moduleFactories");
		expect(dynamic).not.toContain("rspackRequire(");
		expect(dynamic).toContain("require_shared()");
		expect(dynamic).toContain("compatGetDefaultExport(dynamic_0)");

		expect(entry).not.toContain("export { __rspack_context");
		expect(entry).not.toContain("moduleFactories");
		expect(entry).not.toContain("function rspackRequire(moduleId)");
		expect(entry).toContain("require_shared");
		expect(entry).toContain(
			'import("./dynamic.mjs").then(m => m.require_dynamic())'
		);
	}
};
