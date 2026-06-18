const fs = require("fs");
const path = require("path");

let emittedSource = "";

module.exports = {
	findBundle(i, options) {
		emittedSource = fs.readFileSync(
			path.join(options.output.path, `bundle${i}.mjs`),
			"utf-8"
		);
		// `__custom_import_meta` is not defined in this environment, so don't execute.
		return [];
	},
	afterExecute() {
		// The kept createRequire honors the customized importMetaName — for the deferred
		// variable form and the non-deferred forms (multi-argument call, inline export value).
		expect(emittedSource).toContain("req = __rspack_createRequire(__custom_import_meta.url)");
		expect(emittedSource).toContain("createRequire(__custom_import_meta.url, (ran = true))");
		// No literal `import.meta` is left in the user's createRequire calls (a multi-argument
		// or an inline export-default call would otherwise keep it).
		expect(emittedSource).not.toContain("createRequire(import.meta.url,");
		expect(emittedSource).not.toContain("createRequire(import.meta.url))");
	}
};
