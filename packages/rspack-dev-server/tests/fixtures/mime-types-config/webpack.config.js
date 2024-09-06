"use strict";

const moduleRuleForCustom = {
	test: /\.custom$/,
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
	node: false,
	infrastructureLogging: {
		level: "warn"
	},
	module: {
		rules: [
			{
				...moduleRuleForCustom
			}
		]
	}
};
