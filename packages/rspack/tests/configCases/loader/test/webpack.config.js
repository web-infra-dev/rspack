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
				test: /lib\.js/,
				uses: [
					{
						loader: "./loader-2.js"
					}
				]
			},
			{
				test: resolve("lib.js"),
				uses: [
					{
						loader: "./loader-1.js"
					}
				]
			}
		]
	}
};
