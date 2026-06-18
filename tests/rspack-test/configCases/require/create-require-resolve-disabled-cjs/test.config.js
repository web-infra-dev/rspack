const fs = require("fs");
const path = require("path");

let emittedSource = "";

module.exports = {
	findBundle(i, options) {
		// The preserved literal `import.meta.url` makes the CommonJS bundle invalid
		// to execute, so capture the emitted artifact and skip running it.
		emittedSource = fs.readFileSync(
			path.join(options.output.path, `bundle${i}.js`),
			"utf-8"
		);
		return [];
	},
	afterExecute() {
		// The createRequire(...) call is kept with import.meta.url preserved verbatim
		// in the CommonJS artifact (NOT baked to a file:// path) — which is exactly
		// what the warning is about.
		expect(emittedSource).toContain("createRequire(import.meta.url)");
		expect(emittedSource).not.toContain("file://");
	}
};
