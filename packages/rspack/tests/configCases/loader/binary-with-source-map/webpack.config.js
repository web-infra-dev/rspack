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
				use: [{ loader: function () {} }],
				type: "asset/resource"
			}
		]
	}
};
