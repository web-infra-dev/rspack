const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: path.join(__dirname, "a.js"),
				use: [{ loader: "./loader-2.js" }, { loader: "./loader-1.js" }]
			}
		]
	}
};
