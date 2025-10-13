"use strict";

/** @type {import("@rspack/coretypes").Configuration} */
module.exports = {
	entry: { main: "./index.js" },
	output: {
		module: true,
		library: {
			type: "module"
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
	externals: ["fs", "path"],
	externalsType: "module",
	optimization: {
		concatenateModules: true,
		usedExports: true
	}
};
