const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "bundle0.js"),
			"utf-8"
		);

		expect(source).toContain('var uniqueName = "runtime-review:";');
		expect(source).toContain('var cssLoadingUniqueName = "runtime-review";');
		expect(source).toContain("uniqueName + key");
		expect(source).toContain('cssLoadingUniqueName + ":" + key');
		expect(source).not.toContain('var uniqueName = "runtime-review";');
	}
};
