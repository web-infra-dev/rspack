"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = () => ({
	devtool: false,
	mode: "development",
	entry: {
		main: {
			import: "./index.js",
			dependOn: "shared"
		},
		shared: "./common.js"
	},
	output: {
		module: true,
		filename: "[name].mjs",
		library: {
			type: "module"
		}
	},
	target: ["web", "es2020"],
	experiments: {
		},
	optimization: {
		minimize: false,
		runtimeChunk: false,
		concatenateModules: true
	}
});
