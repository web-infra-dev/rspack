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
		// The function-scoped createRequire was cleared (its only use is a bundled invoke); the
		// top-level `export const req` sharing the name must not have kept it. So no literal
		// `import.meta` (a syntax error here) is left in the CommonJS artifact.
		expect(emittedSource).toContain("/* createRequire() */ undefined");
		expect(emittedSource).not.toContain("import.meta");
	}
};
