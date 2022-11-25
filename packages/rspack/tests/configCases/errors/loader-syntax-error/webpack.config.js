const path = require("path")
/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: path.join(__dirname, "lib.js"),
				use: [
					{
						loader: "./my-loader.js"
					}
				]
			}
		]
	}
};
