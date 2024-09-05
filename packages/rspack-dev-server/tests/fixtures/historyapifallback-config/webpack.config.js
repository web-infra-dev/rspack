"use strict";

const moduleRuleForHTML = {
	test: /\.html$/,
	type: "asset/resource",
	generator: {
		filename: "[name][ext]"
	}
};

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
				...moduleRuleForHTML
			}
		]
	},
	infrastructureLogging: {
		level: "warn"
	}
};
