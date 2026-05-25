const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const readAsset = file =>
			fs.readFileSync(path.join(options.output.path, file), "utf-8");

		const entry = readAsset("main.mjs");
		const entryImpl = readAsset("index_js.mjs");
		const dynamic = readAsset("dynamic.mjs");

		expect(entry).toContain('import "./index_js.mjs";');
		expect(entryImpl).toContain("Promise.all");
		expect(entryImpl).toContain('import("node:stream")');
		expect(entryImpl).toContain('import("./dynamic.mjs")');
		expect(entry).not.toContain("export { __webpack_require__");
		expect(entry).not.toContain("as __webpack_require__");
		expect(dynamic).not.toContain('from "./main.mjs"');
		expect(dynamic).toContain('import { __webpack_require__ } from "./index_js.mjs";');
	},
};
