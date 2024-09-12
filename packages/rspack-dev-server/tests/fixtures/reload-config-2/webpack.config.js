"use strict";

const HTMLGeneratorPlugin = require("../../helpers/html-generator-plugin");

module.exports = {
	mode: "development",
	context: __dirname,
	stats: "none",
	entry: "./foo.js",
	output: {
		path: "/"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [{ loader: "style-loader" }, { loader: "css-loader" }]
			}
		]
	},
	node: false,
	infrastructureLogging: {
		level: "info",
		stream: {
			write: () => {}
		}
	},
	plugins: [new HTMLGeneratorPlugin()]
};
