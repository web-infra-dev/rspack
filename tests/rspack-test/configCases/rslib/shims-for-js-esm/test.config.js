const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle: () => [],
	validate(stats, stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const content = fs.readFileSync(
			path.resolve(config.output.path, "bundle.mjs"),
			"utf-8"
		);
		expect(content).toContain("__rspack_fileURLToPath");
		expect(content).toContain("__rspack_import_meta_dirname__");
	}
};
