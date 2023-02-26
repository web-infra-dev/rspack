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
				use: "./loader0.js"
			},
			{
				exclude: [/index\.js/],
				use: "./loader1.js",
				issuer: {
					not: [/index\.js/]
				}
			},
			{
				exclude: [/index\.js/],
				use: "./loader2.js",
				issuer: {
					and: [/1\.js/, path.resolve(__dirname, "lib")]
				}
			},
			{
				exclude: [/index\.js/],
				use: "./loader3.js",
				issuer: {
					or: [/1\.js/, /2\.js/]
				}
			}
		]
	}
};
