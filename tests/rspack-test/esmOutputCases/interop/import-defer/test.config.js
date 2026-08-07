const fs = require("fs");
const path = require("path");

module.exports = {
	snapshotContent(content) {
		return content.replace(/[ \t]+$/gm, "").replace(/ +\t/g, "\t");
	},
	afterExecute(options) {
		let hasDeferredExternalLoader = false;
		for (const file of fs.readdirSync(options.output.path)) {
			if (!file.endsWith(".mjs")) continue;
			const source = fs.readFileSync(path.join(options.output.path, file), "utf-8");
			expect(source).not.toContain("__rspack_module_relocation_");
			expect(source).not.toContain("__webpack_modules__");
			expect(source).not.toContain("__webpack_module_cache__");
			expect(source).not.toContain("function __webpack_require__");
			if (
				source.includes("deferred-external.cjs") &&
				source.includes("function() { return module.createRequire")
			) {
				hasDeferredExternalLoader = true;
			}
		}
		expect(hasDeferredExternalLoader).toBe(true);
	},
};
