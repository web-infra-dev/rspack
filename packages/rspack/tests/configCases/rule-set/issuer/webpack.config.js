const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				use: [
					{
						loader: "./loader.js"
					}
				],
				issuer: {
					not: [/index\.css/]
				}
			},
			{
				use: [
					{
						loader: "./loader1.js"
					}
				],
				issuer: {
					not: [/index\.js/]
				}
			}
		]
	}
};
