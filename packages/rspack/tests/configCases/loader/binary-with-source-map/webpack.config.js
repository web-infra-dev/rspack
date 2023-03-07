const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	devtool: "source-map",
	module: {
		rules: [
			{
				test: path.join(__dirname, "logo.png"),
				use: ["./empty-loader.js"],
				type: "asset/resource"
			}
		]
	}
};
