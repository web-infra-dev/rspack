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
		expect(dynamic).toContain("moduleFactories.add");
		expect(dynamic).toContain('rspackRequire(/*! ./shared */ "./shared.js")');
		expect(dynamic).toContain("compatGetDefaultExport(dynamic)");

		expect(entry).not.toContain("export { __rspack_context");
		expect(entry).toContain("var modules = {};");
		expect(entry).toContain("function rspackRequire(moduleId)");
		expect(entry).toContain(
			'createFakeNamespaceObject.bind(rspackRequire, /*! ./dynamic */ "./dynamic.js", 19)'
		);
	}
};
