const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				exclude: [/index\.js/],
				use: [
					{
						loader: "./loader.js"
					}
				]
			},
			{
				exclude: [/index\.js/],
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
