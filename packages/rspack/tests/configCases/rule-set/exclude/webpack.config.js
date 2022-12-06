const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				exclude: [/lib\.js/, /index\.js/],
				use: [
					{
						loader: "./loader.js"
					}
				]
			}
		]
	}
};
