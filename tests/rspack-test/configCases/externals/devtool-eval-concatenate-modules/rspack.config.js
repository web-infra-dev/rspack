"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "eval",
	module: {
		parser: {
			javascript: {
				importMeta: false
			}
		}
	},
	entry: {
		main: "./index.js",
		imported: {
			import: "./imported.js",
			library: {
				type: "module"
			}
		}
	},
	target: "node14",
	output: {
		module: true,
		filename: "[name].mjs",
		module: true,
		library: {
			type: "module"
		}
	},
	externals: "./imported.mjs",
	externalsType: "module",
	experiments: {
		},
	optimization: {
		concatenateModules: true
	}
};
