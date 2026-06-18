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
		// No `.resolve` use -> the whole createRequire(import.meta.url) call is cleared to
		// `undefined` (and its `module` import dropped), so nothing leaks into the (valid,
		// executed) CommonJS artifact.
		expect(emittedSource).toContain("/* createRequire() */ undefined");
		expect(emittedSource).not.toContain("createRequire(import.meta.url");
	}
};
