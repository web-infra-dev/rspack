"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	experiments: {
		outputModule: true
	},
	output: {
		module: true,
		importMetaName: "custom"
	},
	node: {
		__filename: "node-module",
		__dirname: "node-module"
	}
};
