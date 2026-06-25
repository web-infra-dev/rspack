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
		expect(entry).toContain('import { __webpack_require__ } from "./runtime.mjs";');
		expect(entry).toContain("Promise.all");
		expect(entry).toContain(
			'__webpack_require__.t(module.createRequire(import.meta.url)("node:stream"), 22)'
		);
		expect(entry).toContain('import("./dynamic.mjs")');
		expect(runtime).toContain("export { __webpack_require__");
		expect(entry).not.toContain("export { __webpack_require__");
		expect(entry).not.toContain("as __webpack_require__");
		expect(dynamic).not.toContain('from "./main.mjs"');
		expect(dynamic).toContain('import { __webpack_require__ } from "./runtime.mjs";');
	},
};
