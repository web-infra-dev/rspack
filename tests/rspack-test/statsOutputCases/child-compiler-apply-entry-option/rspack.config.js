"use strict";

var TestApplyEntryOptionPlugin = require("./TestApplyEntryOptionPlugin");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		parent: "./parent"
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new TestApplyEntryOptionPlugin({
			entry: {
				child: "./child"
			}
		})
	],
	stats: {
		assets: true,
		modules: true,
		children: true,
		entrypoints: true
	}
};
