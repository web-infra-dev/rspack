const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "bundle0.js"),
			"utf-8"
		);
		expect(source).toContain("leafPrototypes = leafPrototypes || [");
		expect(source).not.toContain("leafPrototypes ||= [");
	},
};
