const fs = require("fs");
const path = require("path");

let emittedSource = "";

module.exports = {
	findBundle(i, options) {
		const file = `bundle${i}.js`;
		emittedSource = fs.readFileSync(path.join(options.output.path, file), "utf-8");
		return file;
	},
	afterExecute() {
		// The `new URL(import.meta.url)` argument is not the deferrable bare form, so it is
		// baked to a build-time path instead of being left verbatim. No `import.meta` (which
		// would be a syntax error here) is leaked into the CommonJS artifact.
		expect(emittedSource).not.toContain("import.meta");
		expect(emittedSource).not.toContain("new URL");
		expect(emittedSource).toContain("createRequire(");
	}
};
