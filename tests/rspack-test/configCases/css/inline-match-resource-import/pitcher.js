const path = require("path");

module.exports.pitch = function () {
	const request = this.utils.contextify(
		this.context,
		`${this.resourcePath}.css!=!-!${path.resolve(
			__dirname,
			"style-loader.js"
		)}!${this.resourcePath}`
	);

	return `@import ${JSON.stringify(request)};`;
};
