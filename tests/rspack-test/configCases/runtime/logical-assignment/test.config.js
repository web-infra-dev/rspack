const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		for (const option of Array.isArray(options) ? options : [options]) {
			const source = fs.readFileSync(
				path.resolve(option.output.path, option.output.filename),
				"utf-8"
			);
			expect(source).toContain("getProto = Object.getPrototypeOf;");
			expect(source).not.toContain("obj.__proto__");
			if (option.output.environment.logicalAssignment) {
				expect(source).toContain("leafPrototypes ||= [");
				expect(source).toContain("priority ||= 0;");
			} else {
				expect(source).toContain("leafPrototypes = leafPrototypes || [");
				expect(source).toContain("priority = priority || 0;");
				expect(source).not.toContain("leafPrototypes ||= [");
				expect(source).not.toContain("priority ||= 0;");
			}
		}
	}
};
