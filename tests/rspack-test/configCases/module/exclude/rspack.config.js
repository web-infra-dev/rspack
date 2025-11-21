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
				exclude: /lib\.js/,
				use: [
					{
						loader: "./loader.js"
					}
				]
			},
			{
				exclude: resolve("index.js"),
				use: [
					{
						loader: "./loader.js"
					}
				]
			}
		]
	}
};
