const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.join(options.output.path, "main.mjs"),
			"utf-8"
		);

		expect(source).toContain('const __rspack_createRequire = "local-createRequire";');
		expect(source).toContain(
			'const __rspack_createRequire_require = "local-require";'
		);
		expect(source).not.toContain(
			"import { createRequire as __rspack_createRequire }"
		);
		expect(source).toContain('import("node:module").then');
		expect(source).toContain('module.createRequire(import.meta.url)("fs")');
	}
};
