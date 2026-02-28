const path = require("path");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	target: "web",
	node: {
		__dirname: false,
		__filename: false
	},
	mode: "development",
	entry: "./index.js",
	output: {
		hashFunction: "xxhash64",
		cssFilename: "main.css"
	},
	module: {
		parser: {
			"css/auto": {
				namedExports: true
			}
		},
		generator: {
			"css/auto": {
				exportsConvention: "as-is",
				localIdentName: "[hash]-[local]"
			}
		},
		rules: [
			{
				test: /\.css/,
				type: "css/auto"
			},
			{
				include: path.resolve(__dirname, "legacy"),
				test: /\.css$/,
				type: "css/module",
				parser: {
					namedExports: false
				},
				generator: {
					exportsConvention: "camel-case",
					localIdentName: "[hash]-[local]"
				}
			},
		]
	},

};
