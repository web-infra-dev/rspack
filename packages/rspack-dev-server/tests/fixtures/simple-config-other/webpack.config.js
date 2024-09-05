"use strict";

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
	}
};
