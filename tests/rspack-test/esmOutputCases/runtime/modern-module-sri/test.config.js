const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle: () => [],
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		const source = fs.readFileSync(
			path.resolve(config.output.path, "runtime.mjs"),
			"utf-8"
		);
		const requireName =
			config.experiments?.runtimeMode === "rspack"
				? "rspackRequire"
				: "__webpack_require__";
		expect(source).toContain(`${requireName}.sriHashes`);
		expect(source).not.toContain("__rspack_context.sriHashes");
	}
};
