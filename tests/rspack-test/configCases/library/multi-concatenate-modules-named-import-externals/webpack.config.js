"use strict";

/** @type {import("@rspack/coretypes").Configuration} */
module.exports = {
	cache: true,
	target: "node",
	mode: "none",
	entry: { main: "./index.js", test: "./other-entry.js" },
	output: {
		module: true,
		library: {
			type: "modern-module"
		},
		filename: "[name].mjs",
		chunkFormat: "module"
	},
	experiments: {
		outputModule: true
	},
	resolve: {
		extensions: [".js"]
	},
	externalsType: "module",
	externals: {
		"externals-1/foo": "fs",
		"externals-2/foo": "fs-extra"
	},
	optimization: {
		concatenateModules: true,
		usedExports: true
	}
};
