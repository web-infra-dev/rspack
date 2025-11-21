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
				resource: /lib\.js/,
				use: [
					{
						loader: "./loader-2.js"
					}
				]
			},
			{
				resource: resolve("lib.js"),
				use: [
					{
						loader: "./loader-1.js"
					}
				]
			}
		]
	}
};
