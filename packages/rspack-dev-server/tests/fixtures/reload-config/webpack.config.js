"use strict";

const HTMLGeneratorPlugin = require("../../helpers/html-generator-plugin");

module.exports = {
	mode: "development",
	context: __dirname,
	entry: "./foo.js",
	stats: "none",
	output: {
		path: "/"
	},
	experiments: {
		css: true
	},
	module: {
		// rules: [
		//   {
		//     test: /\.css$/,
		//     use: [{ loader: "style-loader" }, { loader: "css-loader" }],
		//   },
		// ],
	},
	infrastructureLogging: {
		level: "info",
		stream: {
			write: () => {}
		}
	},
	plugins: [new HTMLGeneratorPlugin()]
};
