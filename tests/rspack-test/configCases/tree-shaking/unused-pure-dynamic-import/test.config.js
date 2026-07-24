const fs = require("fs");
const path = require("path");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
	validate(_stats, _stderr, options) {
		const config = Array.isArray(options) ? options[0] : options;
		expect(
			fs.existsSync(path.join(config.output.path, "used-dynamic-import.js"))
		).toBe(true);
		expect(
			fs.existsSync(
				path.join(config.output.path, "unused-dynamic-import.js")
			)
		).toBe(false);
	}
};
