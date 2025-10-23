"use strict";
let rspack = require("@rspack/core");



/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	experiments: {
		outputModule: true
	},
	output: {
		module: true,
		chunkFormat: "module",
		filename: "[name].mjs",
		chunkFilename: "[name].chunk.mjs"
	},
	devtool: false,
	plugins: [
		new rspack.HtmlRspackPlugin({ template: "./src/index.html" , scriptLoading: "module"}),
	],
	optimization: {
		minimize: false,
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				split: {
					test: /split/,
					name: "split",
					priority: 10,
					enforce: true
				}
			}
		}
	}
};
