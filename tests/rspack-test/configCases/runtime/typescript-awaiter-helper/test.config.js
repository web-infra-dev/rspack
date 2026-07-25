const fs = require("fs");
const path = require("path");

module.exports = {
	afterExecute(options) {
		const source = fs.readFileSync(
			path.resolve(options.output.path, "bundle0.js"),
			"utf-8"
		);

		expect(source).not.toContain("this && this.__awaiter");
		expect(source).toMatch(
			/var __awaiter = (?:__webpack_require__|__rspack_context)\.N;/
		);
	}
};
