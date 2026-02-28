const path = require("path");
const resolve = filename => path.resolve(__dirname, filename);

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				include: value => value === resolve("lib.js"),
				use: "./loader-3.js"
			},
			{
				include: /lib\.js/,
				use: [
					{
						loader: "./loader-2.js"
					}
				]
			},
			{
				include: resolve("lib.js"),
				use: [
					{
						loader: "./loader-1.js"
					}
				]
			}
		]
	}
};
