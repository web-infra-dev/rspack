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
				use: [
					{
						loader: "./loader-2.js"
					}
				]
			},
			{
				test: resolve("lib.js"),
				use: [
					{
						loader: "./loader-1.js"
					}
				]
			},
			{
				test: /\.module\.less$/,
				type: "css/module"
			},
			{
				test: /(?<!module).less$/,
				type: "css"
			},
			{
				test: /\.svg$/i,
				type: "asset"
			}
		]
	}
};
