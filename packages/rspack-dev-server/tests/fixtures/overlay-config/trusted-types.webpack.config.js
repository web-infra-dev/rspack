"use strict";

const HTMLGeneratorPlugin = require("../../helpers/trusted-types-html-generator-plugin");

module.exports = {
	mode: "development",
	context: __dirname,
	stats: "none",
	entry: "./foo.js",
	output: {
		path: "/",
		trustedTypes: { policyName: "webpack" }
	},
	infrastructureLogging: {
		level: "info",
		stream: {
			write: () => {}
		}
	},
	plugins: [new HTMLGeneratorPlugin()]
};
