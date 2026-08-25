const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "bundle0.js"),
			"utf-8"
		);
		expect(source).toContain("priority ||= 0");
		expect(source).toContain("leafPrototypes ||= [");
		expect(source).toContain('chunkId + ".bundle0.js"');
		expect(source).not.toContain('\"\" + chunkId');
	},
};
