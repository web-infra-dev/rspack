"use strict";

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
	}
};
