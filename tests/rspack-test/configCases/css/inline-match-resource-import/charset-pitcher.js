const fs = require("fs");
const path = require("path");

module.exports.pitch = function () {
	const request = this.utils.contextify(
		this.context,
		`${this.resourcePath}.css!=!-!${path.resolve(
			__dirname,
			"charset-style-loader.js"
		)}!${this.resourcePath}`
	);
	const source = fs.readFileSync(this.resourcePath, "utf-8");

	return `@import ${JSON.stringify(request)};${source}`;
};
