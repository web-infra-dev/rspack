"use strict";

const HTMLGeneratorPlugin = require("../../helpers/html-generator-plugin");

module.exports = {
	mode: "development",
	context: __dirname,
	stats: "none",
	entry: "./foo.js",
	output: {
		publicPath: "/"
	},
	infrastructureLogging: {
		level: "warn"
	},
	plugins: [new HTMLGeneratorPlugin()]
};
