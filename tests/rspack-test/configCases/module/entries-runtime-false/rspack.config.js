"use strict";

const EntryPlugin = require("@rspack/core").EntryPlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = () => ({
	devtool: false,
	mode: "development",
	entry: {
		main: {
			import: "./index.js"
		}
	},
	output: {
		filename: "[name].mjs",
		library: {
			type: "module"
		}
	},
	target: ["web", "es2020"],
	experiments: {
		outputModule: true
	},
	optimization: {
		minimize: false,
		runtimeChunk: false,
		splitChunks: {
			cacheGroups: {
				separate: {
					test: /separate/,
					chunks: "all",
					filename: "separate.mjs",
					enforce: true
				},
				common: {
					test: /common/,
					chunks: "all",
					filename: "common.mjs",
					enforce: true
				}
			}
		}
	},
	plugins: [new EntryPlugin(__dirname, "./separate.js", "main")]
});
