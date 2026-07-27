const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "main.js"),
			"utf-8"
		);
		expect(source.match(/__rspack_context\.ctx\(/g) || []).toHaveLength(8);
		expect(source.match(/__rspack_context\.ctx\s*=/g) || []).toHaveLength(1);
		expect(source).not.toContain(
			"function __rspack_context_module_resolve(req)"
		);
	}
};
