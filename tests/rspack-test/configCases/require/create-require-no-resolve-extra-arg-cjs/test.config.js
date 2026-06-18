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
		// The non-deferred multi-argument createRequire baked its first argument, so no
		// literal `import.meta` (a syntax error here) is left in the CommonJS artifact.
		expect(emittedSource).not.toContain("import.meta");
		expect(emittedSource).toContain("createRequire(");
	}
};
