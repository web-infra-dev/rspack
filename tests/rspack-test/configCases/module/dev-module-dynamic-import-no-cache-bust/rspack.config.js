"use strict";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		main: "./index.js"
	},
	output: {
		module: true,
		library: { type: "module" },
		chunkFormat: "module",
		chunkLoading: "import",
		filename: "[name].mjs",
		chunkFilename: "[name].chunk.mjs"
	},
	optimization: {
		runtimeChunk: true,
		minimize: false
	}
};
